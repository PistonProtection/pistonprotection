//! Service communication tests
//!
//! Tests that verify communication between services
//! using mock implementations.

use super::test_helpers::{generate_test_id, TestBackend, TestOrganization, TestUser};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

// ============================================================================
// Mock Service Infrastructure
// ============================================================================

/// Mock message bus for inter-service communication
struct MockMessageBus {
    messages: Arc<RwLock<Vec<ServiceMessage>>>,
    subscribers: Arc<RwLock<HashMap<String, Vec<Box<dyn Fn(&ServiceMessage) + Send + Sync>>>>>,
}

#[derive(Clone, Debug)]
struct ServiceMessage {
    id: String,
    source: String,
    destination: String,
    message_type: String,
    payload: String,
    timestamp: u64,
}

impl MockMessageBus {
    fn new() -> Self {
        Self {
            messages: Arc::new(RwLock::new(Vec::new())),
            subscribers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn publish(&self, message: ServiceMessage) {
        self.messages.write().await.push(message.clone());
    }

    async fn get_messages(&self) -> Vec<ServiceMessage> {
        self.messages.read().await.clone()
    }

    async fn get_messages_by_type(&self, message_type: &str) -> Vec<ServiceMessage> {
        self.messages
            .read()
            .await
            .iter()
            .filter(|m| m.message_type == message_type)
            .cloned()
            .collect()
    }

    async fn clear(&self) {
        self.messages.write().await.clear();
    }
}

/// Mock service registry
struct MockServiceRegistry {
    services: HashMap<String, MockServiceInfo>,
}

#[derive(Clone)]
struct MockServiceInfo {
    name: String,
    address: String,
    port: u16,
    healthy: bool,
    version: String,
}

impl MockServiceRegistry {
    fn new() -> Self {
        Self {
            services: HashMap::new(),
        }
    }

    fn register(&mut self, name: &str, address: &str, port: u16) {
        self.services.insert(
            name.to_string(),
            MockServiceInfo {
                name: name.to_string(),
                address: address.to_string(),
                port,
                healthy: true,
                version: "1.0.0".to_string(),
            },
        );
    }

    fn discover(&self, name: &str) -> Option<&MockServiceInfo> {
        self.services.get(name)
    }

    fn set_health(&mut self, name: &str, healthy: bool) {
        if let Some(service) = self.services.get_mut(name) {
            service.healthy = healthy;
        }
    }

    fn get_healthy_services(&self) -> Vec<&MockServiceInfo> {
        self.services.values().filter(|s| s.healthy).collect()
    }
}

// ============================================================================
// Mock Services
// ============================================================================

/// Mock Gateway Service
struct MockGatewayService {
    message_bus: Arc<MockMessageBus>,
    backends: HashMap<String, BackendState>,
}

#[derive(Clone)]
struct BackendState {
    id: String,
    name: String,
    address: String,
    status: String,
    connections: u32,
}

impl MockGatewayService {
    fn new(message_bus: Arc<MockMessageBus>) -> Self {
        Self {
            message_bus,
            backends: HashMap::new(),
        }
    }

    async fn register_backend(&mut self, backend: &TestBackend) -> String {
        let id = generate_test_id();
        self.backends.insert(
            id.clone(),
            BackendState {
                id: id.clone(),
                name: backend.name.clone(),
                address: backend.address.clone(),
                status: "active".to_string(),
                connections: 0,
            },
        );

        // Publish event
        self.message_bus
            .publish(ServiceMessage {
                id: generate_test_id(),
                source: "gateway".to_string(),
                destination: "metrics".to_string(),
                message_type: "backend.registered".to_string(),
                payload: serde_json::json!({
                    "backend_id": id,
                    "name": backend.name,
                }).to_string(),
                timestamp: chrono::Utc::now().timestamp_millis() as u64,
            })
            .await;

        id
    }

    async fn update_backend_status(&mut self, backend_id: &str, status: &str) {
        if let Some(backend) = self.backends.get_mut(backend_id) {
            backend.status = status.to_string();

            self.message_bus
                .publish(ServiceMessage {
                    id: generate_test_id(),
                    source: "gateway".to_string(),
                    destination: "metrics".to_string(),
                    message_type: "backend.status_changed".to_string(),
                    payload: serde_json::json!({
                        "backend_id": backend_id,
                        "status": status,
                    }).to_string(),
                    timestamp: chrono::Utc::now().timestamp_millis() as u64,
                })
                .await;
        }
    }

    async fn record_connection(&mut self, backend_id: &str) {
        if let Some(backend) = self.backends.get_mut(backend_id) {
            backend.connections += 1;

            self.message_bus
                .publish(ServiceMessage {
                    id: generate_test_id(),
                    source: "gateway".to_string(),
                    destination: "metrics".to_string(),
                    message_type: "connection.established".to_string(),
                    payload: serde_json::json!({
                        "backend_id": backend_id,
                        "connections": backend.connections,
                    }).to_string(),
                    timestamp: chrono::Utc::now().timestamp_millis() as u64,
                })
                .await;
        }
    }
}

/// Mock Auth Service
struct MockAuthService {
    message_bus: Arc<MockMessageBus>,
    sessions: HashMap<String, SessionInfo>,
}

#[derive(Clone)]
struct SessionInfo {
    token: String,
    user_id: String,
    org_id: Option<String>,
    created_at: u64,
}

impl MockAuthService {
    fn new(message_bus: Arc<MockMessageBus>) -> Self {
        Self {
            message_bus,
            sessions: HashMap::new(),
        }
    }

    async fn create_session(&mut self, user_id: &str, org_id: Option<&str>) -> String {
        let token = format!("token_{}", generate_test_id());
        self.sessions.insert(
            token.clone(),
            SessionInfo {
                token: token.clone(),
                user_id: user_id.to_string(),
                org_id: org_id.map(|s| s.to_string()),
                created_at: chrono::Utc::now().timestamp_millis() as u64,
            },
        );

        self.message_bus
            .publish(ServiceMessage {
                id: generate_test_id(),
                source: "auth".to_string(),
                destination: "*".to_string(),
                message_type: "session.created".to_string(),
                payload: serde_json::json!({
                    "user_id": user_id,
                }).to_string(),
                timestamp: chrono::Utc::now().timestamp_millis() as u64,
            })
            .await;

        token
    }

    fn validate_session(&self, token: &str) -> Option<&SessionInfo> {
        self.sessions.get(token)
    }

    async fn invalidate_session(&mut self, token: &str) -> bool {
        if self.sessions.remove(token).is_some() {
            self.message_bus
                .publish(ServiceMessage {
                    id: generate_test_id(),
                    source: "auth".to_string(),
                    destination: "*".to_string(),
                    message_type: "session.invalidated".to_string(),
                    payload: serde_json::json!({
                        "token_hash": format!("hash_{}", &token[..8]),
                    }).to_string(),
                    timestamp: chrono::Utc::now().timestamp_millis() as u64,
                })
                .await;
            true
        } else {
            false
        }
    }
}

/// Mock Metrics Service
struct MockMetricsService {
    message_bus: Arc<MockMessageBus>,
    metrics: HashMap<String, Vec<MetricPoint>>,
    alerts: Vec<Alert>,
}

#[derive(Clone)]
struct MetricPoint {
    timestamp: u64,
    value: f64,
    labels: HashMap<String, String>,
}

#[derive(Clone)]
struct Alert {
    id: String,
    name: String,
    condition: String,
    triggered: bool,
    triggered_at: Option<u64>,
}

impl MockMetricsService {
    fn new(message_bus: Arc<MockMessageBus>) -> Self {
        Self {
            message_bus,
            metrics: HashMap::new(),
            alerts: Vec::new(),
        }
    }

    async fn record_metric(&mut self, name: &str, value: f64, labels: HashMap<String, String>) {
        let point = MetricPoint {
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            value,
            labels: labels.clone(),
        };

        self.metrics
            .entry(name.to_string())
            .or_insert_with(Vec::new)
            .push(point);

        // Check alerts
        for alert in &mut self.alerts {
            if self.evaluate_alert_condition(&alert.condition, name, value) && !alert.triggered {
                alert.triggered = true;
                alert.triggered_at = Some(chrono::Utc::now().timestamp_millis() as u64);

                self.message_bus
                    .publish(ServiceMessage {
                        id: generate_test_id(),
                        source: "metrics".to_string(),
                        destination: "gateway".to_string(),
                        message_type: "alert.triggered".to_string(),
                        payload: serde_json::json!({
                            "alert_id": alert.id,
                            "name": alert.name,
                            "metric": name,
                            "value": value,
                        }).to_string(),
                        timestamp: chrono::Utc::now().timestamp_millis() as u64,
                    })
                    .await;
            }
        }
    }

    fn evaluate_alert_condition(&self, condition: &str, metric_name: &str, value: f64) -> bool {
        // Simple condition evaluation (metric > threshold)
        if let Some((name, threshold)) = condition.split_once('>') {
            if name.trim() == metric_name {
                if let Ok(t) = threshold.trim().parse::<f64>() {
                    return value > t;
                }
            }
        }
        false
    }

    fn add_alert(&mut self, name: &str, condition: &str) -> String {
        let id = generate_test_id();
        self.alerts.push(Alert {
            id: id.clone(),
            name: name.to_string(),
            condition: condition.to_string(),
            triggered: false,
            triggered_at: None,
        });
        id
    }

    fn get_metric(&self, name: &str) -> Option<&Vec<MetricPoint>> {
        self.metrics.get(name)
    }

    fn get_triggered_alerts(&self) -> Vec<&Alert> {
        self.alerts.iter().filter(|a| a.triggered).collect()
    }
}

// ============================================================================
// Service Communication Tests
// ============================================================================

#[cfg(test)]
mod message_bus_tests {
    use super::*;

    #[tokio::test]
    async fn test_publish_message() {
        let bus = MockMessageBus::new();

        bus.publish(ServiceMessage {
            id: "msg-1".to_string(),
            source: "gateway".to_string(),
            destination: "metrics".to_string(),
            message_type: "test.event".to_string(),
            payload: "{}".to_string(),
            timestamp: 0,
        })
        .await;

        let messages = bus.get_messages().await;
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].source, "gateway");
    }

    #[tokio::test]
    async fn test_filter_messages_by_type() {
        let bus = MockMessageBus::new();

        bus.publish(ServiceMessage {
            id: "1".to_string(),
            source: "gateway".to_string(),
            destination: "metrics".to_string(),
            message_type: "event.a".to_string(),
            payload: "{}".to_string(),
            timestamp: 0,
        })
        .await;

        bus.publish(ServiceMessage {
            id: "2".to_string(),
            source: "gateway".to_string(),
            destination: "metrics".to_string(),
            message_type: "event.b".to_string(),
            payload: "{}".to_string(),
            timestamp: 0,
        })
        .await;

        let messages = bus.get_messages_by_type("event.a").await;
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].id, "1");
    }
}

#[cfg(test)]
mod service_registry_tests {
    use super::*;

    #[test]
    fn test_register_service() {
        let mut registry = MockServiceRegistry::new();
        registry.register("gateway", "localhost", 8080);

        let service = registry.discover("gateway").unwrap();
        assert_eq!(service.name, "gateway");
        assert_eq!(service.port, 8080);
    }

    #[test]
    fn test_health_status() {
        let mut registry = MockServiceRegistry::new();
        registry.register("gateway", "localhost", 8080);
        registry.register("auth", "localhost", 8081);

        registry.set_health("gateway", false);

        let healthy = registry.get_healthy_services();
        assert_eq!(healthy.len(), 1);
        assert_eq!(healthy[0].name, "auth");
    }
}

#[cfg(test)]
mod gateway_auth_communication_tests {
    use super::*;

    #[tokio::test]
    async fn test_session_creation_publishes_event() {
        let bus = Arc::new(MockMessageBus::new());
        let mut auth_service = MockAuthService::new(Arc::clone(&bus));

        auth_service.create_session("user-1", Some("org-1")).await;

        let messages = bus.get_messages_by_type("session.created").await;
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].source, "auth");
    }

    #[tokio::test]
    async fn test_session_invalidation_publishes_event() {
        let bus = Arc::new(MockMessageBus::new());
        let mut auth_service = MockAuthService::new(Arc::clone(&bus));

        let token = auth_service.create_session("user-1", None).await;
        auth_service.invalidate_session(&token).await;

        let messages = bus.get_messages_by_type("session.invalidated").await;
        assert_eq!(messages.len(), 1);
    }

    #[tokio::test]
    async fn test_gateway_validates_session() {
        let bus = Arc::new(MockMessageBus::new());
        let mut auth_service = MockAuthService::new(Arc::clone(&bus));

        let token = auth_service.create_session("user-1", Some("org-1")).await;

        let session = auth_service.validate_session(&token).unwrap();
        assert_eq!(session.user_id, "user-1");
        assert_eq!(session.org_id, Some("org-1".to_string()));
    }
}

#[cfg(test)]
mod gateway_metrics_communication_tests {
    use super::*;

    #[tokio::test]
    async fn test_backend_registration_publishes_event() {
        let bus = Arc::new(MockMessageBus::new());
        let mut gateway = MockGatewayService::new(Arc::clone(&bus));

        let backend = TestBackend::minecraft_java("mc-server", "10.0.0.1");
        gateway.register_backend(&backend).await;

        let messages = bus.get_messages_by_type("backend.registered").await;
        assert_eq!(messages.len(), 1);
    }

    #[tokio::test]
    async fn test_connection_event_publishes() {
        let bus = Arc::new(MockMessageBus::new());
        let mut gateway = MockGatewayService::new(Arc::clone(&bus));

        let backend = TestBackend::minecraft_java("mc-server", "10.0.0.1");
        let backend_id = gateway.register_backend(&backend).await;

        gateway.record_connection(&backend_id).await;

        let messages = bus.get_messages_by_type("connection.established").await;
        assert_eq!(messages.len(), 1);
    }

    #[tokio::test]
    async fn test_backend_status_change_publishes_event() {
        let bus = Arc::new(MockMessageBus::new());
        let mut gateway = MockGatewayService::new(Arc::clone(&bus));

        let backend = TestBackend::minecraft_java("mc-server", "10.0.0.1");
        let backend_id = gateway.register_backend(&backend).await;

        gateway.update_backend_status(&backend_id, "unhealthy").await;

        let messages = bus.get_messages_by_type("backend.status_changed").await;
        assert_eq!(messages.len(), 1);
    }
}

#[cfg(test)]
mod metrics_alert_tests {
    use super::*;

    #[tokio::test]
    async fn test_alert_triggers_on_threshold() {
        let bus = Arc::new(MockMessageBus::new());
        let mut metrics = MockMetricsService::new(Arc::clone(&bus));

        // Add alert for high PPS
        metrics.add_alert("High PPS", "pps > 10000");

        // Record metric that triggers alert
        metrics
            .record_metric("pps", 15000.0, HashMap::new())
            .await;

        let triggered = metrics.get_triggered_alerts();
        assert_eq!(triggered.len(), 1);
        assert_eq!(triggered[0].name, "High PPS");
    }

    #[tokio::test]
    async fn test_alert_publishes_event() {
        let bus = Arc::new(MockMessageBus::new());
        let mut metrics = MockMetricsService::new(Arc::clone(&bus));

        metrics.add_alert("High PPS", "pps > 10000");
        metrics
            .record_metric("pps", 15000.0, HashMap::new())
            .await;

        let messages = bus.get_messages_by_type("alert.triggered").await;
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].destination, "gateway");
    }

    #[tokio::test]
    async fn test_alert_not_triggered_below_threshold() {
        let bus = Arc::new(MockMessageBus::new());
        let mut metrics = MockMetricsService::new(Arc::clone(&bus));

        metrics.add_alert("High PPS", "pps > 10000");
        metrics.record_metric("pps", 5000.0, HashMap::new()).await;

        let triggered = metrics.get_triggered_alerts();
        assert!(triggered.is_empty());
    }

    #[tokio::test]
    async fn test_metric_recording() {
        let bus = Arc::new(MockMessageBus::new());
        let mut metrics = MockMetricsService::new(Arc::clone(&bus));

        let mut labels = HashMap::new();
        labels.insert("backend_id".to_string(), "backend-1".to_string());

        metrics.record_metric("connections", 100.0, labels).await;

        let points = metrics.get_metric("connections").unwrap();
        assert_eq!(points.len(), 1);
        assert_eq!(points[0].value, 100.0);
    }
}

#[cfg(test)]
mod full_flow_communication_tests {
    use super::*;

    #[tokio::test]
    async fn test_full_service_communication_flow() {
        let bus = Arc::new(MockMessageBus::new());

        // Initialize services
        let mut auth = MockAuthService::new(Arc::clone(&bus));
        let mut gateway = MockGatewayService::new(Arc::clone(&bus));
        let mut metrics = MockMetricsService::new(Arc::clone(&bus));

        // Set up alert
        metrics.add_alert("High Connections", "connections > 100");

        // 1. User authenticates
        let token = auth.create_session("user-1", Some("org-1")).await;
        assert!(!token.is_empty());

        // 2. Register backend (as if through gateway API)
        let backend = TestBackend::minecraft_java("mc-server", "10.0.0.1");
        let backend_id = gateway.register_backend(&backend).await;

        // 3. Connections come in
        for _ in 0..5 {
            gateway.record_connection(&backend_id).await;
        }

        // 4. Record metrics
        metrics
            .record_metric("connections", 150.0, HashMap::new())
            .await;

        // 5. Verify events were published
        let all_messages = bus.get_messages().await;
        assert!(all_messages.len() >= 5);

        // Verify specific events
        let session_events = bus.get_messages_by_type("session.created").await;
        assert_eq!(session_events.len(), 1);

        let backend_events = bus.get_messages_by_type("backend.registered").await;
        assert_eq!(backend_events.len(), 1);

        let connection_events = bus.get_messages_by_type("connection.established").await;
        assert_eq!(connection_events.len(), 5);

        let alert_events = bus.get_messages_by_type("alert.triggered").await;
        assert_eq!(alert_events.len(), 1);
    }
}
