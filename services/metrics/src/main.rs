//! PistonProtection Metrics Collector Service
//!
//! Collects and aggregates metrics from worker nodes and provides
//! APIs for querying metrics data, real-time streaming, and alert management.

mod aggregator;
mod alerts;
mod handlers;
mod storage;
mod streams;

use aggregator::{AggregatorConfig, MetricsAggregator};
use alerts::{AlertConfig, AlertManager};
use handlers::MetricsGrpcService;
use pistonprotection_common::{config::Config, geoip::GeoIpService, redis::CacheService, telemetry};
use pistonprotection_proto::metrics::metrics_service_server::MetricsServiceServer;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use storage::{RetentionConfig, TimeSeriesStorage};
use streams::MetricsStreamer;
use tokio::signal;
use tonic::transport::Server;
use tonic_health::server::health_reporter;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::{error, info, warn};

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::Serialize;

const SERVICE_NAME: &str = "metrics";

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub aggregator: Arc<MetricsAggregator>,
    pub storage: Arc<TimeSeriesStorage>,
    pub alerts: Arc<AlertManager>,
    pub streamer: Arc<MetricsStreamer>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let config = Config::load(SERVICE_NAME)?;

    // Initialize telemetry
    telemetry::init(SERVICE_NAME, &config.telemetry)?;

    info!(
        "Starting {} service v{}",
        SERVICE_NAME,
        env!("CARGO_PKG_VERSION")
    );

    // Initialize database connection
    let db_pool = if let Some(ref db_config) = config.database {
        match pistonprotection_common::db::create_pool(db_config).await {
            Ok(pool) => {
                info!("Database connection established");
                Some(pool)
            }
            Err(e) => {
                warn!("Failed to connect to database: {}. Running without persistence.", e);
                None
            }
        }
    } else {
        info!("No database configuration, running without persistence");
        None
    };

    // Initialize Redis connection
    let redis_pool = if let Some(ref redis_config) = config.redis {
        match pistonprotection_common::redis::create_pool(redis_config).await {
            Ok(pool) => {
                info!("Redis connection established");
                Some(pool)
            }
            Err(e) => {
                warn!("Failed to connect to Redis: {}. Running without cache.", e);
                None
            }
        }
    } else {
        info!("No Redis configuration, running without cache");
        None
    };

    // Create cache service if Redis is available
    let cache = redis_pool
        .as_ref()
        .map(|pool| CacheService::new(pool.clone(), "metrics"));

    // Initialize GeoIP service
    let geoip = Arc::new(
        GeoIpService::new(
            std::env::var("GEOIP_CITY_DB").ok().as_deref(),
            std::env::var("GEOIP_ASN_DB").ok().as_deref(),
        )
        .unwrap_or_else(|_| {
            warn!("Failed to load GeoIP databases, using dummy service");
            GeoIpService::dummy()
        }),
    );

    // Create time-series storage
    let retention_config = RetentionConfig {
        raw_retention: Duration::from_secs(
            std::env::var("RETENTION_RAW_HOURS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(24)
                * 3600,
        ),
        five_min_retention: Duration::from_secs(
            std::env::var("RETENTION_5MIN_DAYS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(7)
                * 86400,
        ),
        hourly_retention: Duration::from_secs(
            std::env::var("RETENTION_HOURLY_DAYS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(30)
                * 86400,
        ),
        daily_retention: Duration::from_secs(
            std::env::var("RETENTION_DAILY_DAYS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(365)
                * 86400,
        ),
    };

    let storage = Arc::new(TimeSeriesStorage::new(
        db_pool.clone(),
        redis_pool.clone(),
        "piston:metrics",
        retention_config,
    ));

    // Create metrics aggregator
    let aggregator_config = AggregatorConfig {
        cache_ttl: Duration::from_secs(5),
        stale_threshold: Duration::from_secs(10),
        attack_threshold_multiplier: 3.0,
        min_baseline_samples: 30,
        baseline_window_size: 60,
    };

    let aggregator = Arc::new(MetricsAggregator::new(
        storage.clone(),
        cache,
        geoip,
        aggregator_config,
    ));

    // Create alert manager
    let alert_config = AlertConfig {
        eval_interval: Duration::from_secs(10),
        min_repeat_interval: Duration::from_secs(300),
        notification_retries: 3,
        notification_timeout: Duration::from_secs(10),
    };

    let alerts = AlertManager::new(db_pool.clone(), alert_config);

    // Load alerts from database
    if let Err(e) = alerts.load_alerts().await {
        warn!("Failed to load alerts from database: {}", e);
    }

    // Create metrics streamer
    let streamer = Arc::new(MetricsStreamer::new(aggregator.clone()));

    // Create application state
    let app_state = AppState {
        aggregator: aggregator.clone(),
        storage: storage.clone(),
        alerts: alerts.clone(),
        streamer: streamer.clone(),
    };

    // Start background tasks
    let aggregator_for_flush = aggregator.clone();
    let storage_for_cleanup = storage.clone();

    // Periodic flush task
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            if let Err(e) = aggregator_for_flush.flush_to_storage().await {
                error!("Failed to flush metrics to storage: {}", e);
            }
            aggregator_for_flush.reset_periodic_counters();
        }
    });

    // Periodic cleanup task
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(3600)); // Hourly
        loop {
            interval.tick().await;
            if let Err(e) = storage_for_cleanup.cleanup_old_data().await {
                error!("Failed to cleanup old data: {}", e);
            }
        }
    });

    // Create gRPC server
    let grpc_addr: SocketAddr = config.grpc_addr().parse()?;
    let http_addr: SocketAddr = config.http_addr().parse()?;

    // Setup health reporter
    let (mut health_reporter, health_service) = health_reporter();
    health_reporter
        .set_serving::<MetricsServiceServer<MetricsGrpcService>>()
        .await;

    // Create gRPC service
    let metrics_service = MetricsGrpcService::new(
        aggregator.clone(),
        storage.clone(),
        alerts.clone(),
        streamer.clone(),
    );

    // Create HTTP router for health checks and Prometheus metrics
    let http_router = create_http_router(app_state);

    // Spawn HTTP server
    let http_handle = tokio::spawn(async move {
        info!("HTTP server listening on {}", http_addr);
        let listener = tokio::net::TcpListener::bind(http_addr)
            .await
            .expect("Failed to bind HTTP listener");
        axum::serve(listener, http_router)
            .await
            .expect("HTTP server error");
    });

    // Spawn gRPC server
    let grpc_handle = tokio::spawn(async move {
        info!("gRPC server listening on {}", grpc_addr);
        Server::builder()
            .add_service(health_service)
            .add_service(MetricsServiceServer::new(metrics_service))
            .serve(grpc_addr)
            .await
            .expect("gRPC server error");
    });

    info!("Metrics collector ready");
    info!("  gRPC: {}", grpc_addr);
    info!("  HTTP: {}", http_addr);

    // Wait for shutdown signal
    shutdown_signal().await;
    info!("Shutting down...");

    // Graceful shutdown
    // Note: In a production environment, you would want to properly
    // abort these handles and wait for them to complete
    http_handle.abort();
    grpc_handle.abort();

    // Final flush
    if let Err(e) = aggregator.flush_to_storage().await {
        error!("Failed to flush metrics during shutdown: {}", e);
    }

    telemetry::shutdown();

    info!("Shutdown complete");
    Ok(())
}

/// Create HTTP router for health checks and metrics endpoints
fn create_http_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/health", get(health_check))
        .route("/health/live", get(liveness_check))
        .route("/health/ready", get(readiness_check))
        .route("/metrics", get(prometheus_metrics))
        .route("/api/v1/status", get(service_status))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}

/// Health check response
#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    service: &'static str,
    version: &'static str,
}

/// Service status response
#[derive(Serialize)]
struct StatusResponse {
    status: &'static str,
    service: &'static str,
    version: &'static str,
    uptime_seconds: u64,
    backends_tracked: usize,
    workers_tracked: usize,
    alerts_active: usize,
}

async fn health_check(State(_state): State<AppState>) -> impl IntoResponse {
    Json(HealthResponse {
        status: "healthy",
        service: SERVICE_NAME,
        version: env!("CARGO_PKG_VERSION"),
    })
}

async fn liveness_check() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

async fn readiness_check(State(_state): State<AppState>) -> impl IntoResponse {
    // Could add more sophisticated readiness checks here
    (StatusCode::OK, "READY")
}

async fn prometheus_metrics() -> impl IntoResponse {
    let metrics = pistonprotection_common::metrics::encode_metrics();
    (
        StatusCode::OK,
        [("content-type", "text/plain; version=0.0.4")],
        metrics,
    )
}

async fn service_status(State(state): State<AppState>) -> impl IntoResponse {
    // Get counts from aggregator
    let (workers, _) = state
        .aggregator
        .list_worker_metrics(None)
        .await
        .unwrap_or_default();

    Json(StatusResponse {
        status: "healthy",
        service: SERVICE_NAME,
        version: env!("CARGO_PKG_VERSION"),
        uptime_seconds: 0, // Would need to track start time
        backends_tracked: 0, // Would need to expose this from aggregator
        workers_tracked: workers.len(),
        alerts_active: 0, // Would need to expose this from alert manager
    })
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
