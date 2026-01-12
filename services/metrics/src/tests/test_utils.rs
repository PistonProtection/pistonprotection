//! Test utilities for metrics service tests

use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;

/// Test configuration constants
pub mod constants {
    pub const TEST_BACKEND_ID: &str = "test-backend-456";
    pub const TEST_ORG_ID: &str = "test-org-123";
}

/// Sample traffic metrics for testing
#[derive(Debug, Clone)]
pub struct TestTrafficMetrics {
    pub backend_id: String,
    pub requests_total: u64,
    pub requests_blocked: u64,
    pub requests_passed: u64,
    pub bytes_in: u64,
    pub bytes_out: u64,
    pub latency_sum: f64,
    pub latency_count: u64,
    pub active_connections: u32,
    pub timestamp: DateTime<Utc>,
}

impl Default for TestTrafficMetrics {
    fn default() -> Self {
        Self {
            backend_id: constants::TEST_BACKEND_ID.to_string(),
            requests_total: 1000,
            requests_blocked: 100,
            requests_passed: 900,
            bytes_in: 1_000_000,
            bytes_out: 500_000,
            latency_sum: 50.0,
            latency_count: 1000,
            active_connections: 50,
            timestamp: Utc::now(),
        }
    }
}

impl TestTrafficMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_backend_id(mut self, id: &str) -> Self {
        self.backend_id = id.to_string();
        self
    }

    pub fn with_requests(mut self, total: u64, blocked: u64) -> Self {
        self.requests_total = total;
        self.requests_blocked = blocked;
        self.requests_passed = total.saturating_sub(blocked);
        self
    }

    pub fn with_timestamp(mut self, ts: DateTime<Utc>) -> Self {
        self.timestamp = ts;
        self
    }

    pub fn avg_latency(&self) -> f64 {
        if self.latency_count > 0 {
            self.latency_sum / self.latency_count as f64
        } else {
            0.0
        }
    }
}

/// Sample attack metrics for testing
#[derive(Debug, Clone)]
pub struct TestAttackMetrics {
    pub backend_id: String,
    pub attack_type: String,
    pub source_ips: Vec<String>,
    pub packets_dropped: u64,
    pub bytes_dropped: u64,
    pub duration_seconds: u32,
    pub peak_pps: u64,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
}

impl Default for TestAttackMetrics {
    fn default() -> Self {
        Self {
            backend_id: constants::TEST_BACKEND_ID.to_string(),
            attack_type: "syn_flood".to_string(),
            source_ips: vec!["192.168.1.100".to_string(), "192.168.1.101".to_string()],
            packets_dropped: 10000,
            bytes_dropped: 1_000_000,
            duration_seconds: 300,
            peak_pps: 50000,
            start_time: Utc::now() - Duration::minutes(5),
            end_time: Some(Utc::now()),
        }
    }
}

impl TestAttackMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn ongoing(mut self) -> Self {
        self.end_time = None;
        self
    }

    pub fn with_type(mut self, attack_type: &str) -> Self {
        self.attack_type = attack_type.to_string();
        self
    }
}

/// Generate time series data for testing
pub fn generate_time_series(
    backend_id: &str,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    interval: Duration,
) -> Vec<TestTrafficMetrics> {
    let mut metrics = Vec::new();
    let mut current = start;
    let mut base_requests = 1000u64;

    while current <= end {
        // Add some variance
        let variance = (current.timestamp() % 20) as u64;
        let requests = base_requests + variance * 10;
        let blocked = requests / 10;

        metrics.push(
            TestTrafficMetrics::new()
                .with_backend_id(backend_id)
                .with_requests(requests, blocked)
                .with_timestamp(current),
        );

        current = current + interval;
        base_requests = base_requests.saturating_add(10);
    }

    metrics
}

/// Generate geo distribution data for testing
pub fn generate_geo_data() -> HashMap<String, u64> {
    let mut data = HashMap::new();
    data.insert("US".to_string(), 5000);
    data.insert("DE".to_string(), 2000);
    data.insert("GB".to_string(), 1500);
    data.insert("FR".to_string(), 1000);
    data.insert("JP".to_string(), 800);
    data.insert("CN".to_string(), 500);
    data.insert("RU".to_string(), 200);
    data
}

/// Create a test alert configuration
#[derive(Debug, Clone)]
pub struct TestAlertConfig {
    pub id: String,
    pub backend_id: String,
    pub name: String,
    pub metric: String,
    pub operator: String,
    pub threshold: f64,
    pub duration_seconds: u32,
    pub enabled: bool,
}

impl Default for TestAlertConfig {
    fn default() -> Self {
        Self {
            id: "test-alert-001".to_string(),
            backend_id: constants::TEST_BACKEND_ID.to_string(),
            name: "High Request Rate".to_string(),
            metric: "requests_per_second".to_string(),
            operator: ">".to_string(),
            threshold: 1000.0,
            duration_seconds: 60,
            enabled: true,
        }
    }
}

impl TestAlertConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_metric(mut self, metric: &str, operator: &str, threshold: f64) -> Self {
        self.metric = metric.to_string();
        self.operator = operator.to_string();
        self.threshold = threshold;
        self
    }

    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_traffic_metrics_builder() {
        let metrics = TestTrafficMetrics::new()
            .with_backend_id("custom-backend")
            .with_requests(5000, 500);

        assert_eq!(metrics.backend_id, "custom-backend");
        assert_eq!(metrics.requests_total, 5000);
        assert_eq!(metrics.requests_blocked, 500);
        assert_eq!(metrics.requests_passed, 4500);
    }

    #[test]
    fn test_avg_latency() {
        let metrics = TestTrafficMetrics {
            latency_sum: 100.0,
            latency_count: 10,
            ..Default::default()
        };

        assert_eq!(metrics.avg_latency(), 10.0);
    }

    #[test]
    fn test_generate_time_series() {
        let start = Utc::now() - Duration::hours(1);
        let end = Utc::now();
        let interval = Duration::minutes(5);

        let series = generate_time_series("test-backend", start, end, interval);

        assert!(!series.is_empty());
        // Should have approximately 12 data points (60 min / 5 min)
        assert!(series.len() >= 10 && series.len() <= 14);
    }

    #[test]
    fn test_geo_data() {
        let data = generate_geo_data();

        assert!(data.contains_key("US"));
        assert!(data.contains_key("DE"));
        assert!(*data.get("US").unwrap() > *data.get("RU").unwrap());
    }

    #[test]
    fn test_alert_config_builder() {
        let config = TestAlertConfig::new()
            .with_metric("blocked_pps", ">", 5000.0)
            .disabled();

        assert_eq!(config.metric, "blocked_pps");
        assert_eq!(config.threshold, 5000.0);
        assert!(!config.enabled);
    }
}
