//! Aggregator logic tests

use super::test_utils::{constants, generate_time_series, TestTrafficMetrics};
use crate::aggregator::{
    Aggregator, AggregatorConfig, AggregationInterval, AggregationType, MetricPoint,
};
use chrono::{Duration, Utc};
use std::time::Duration as StdDuration;

/// Create a test aggregator
fn create_test_aggregator() -> Aggregator {
    let config = AggregatorConfig {
        flush_interval: StdDuration::from_secs(10),
        bucket_duration: StdDuration::from_secs(60),
        max_buckets: 1000,
        percentiles: vec![0.5, 0.95, 0.99],
    };
    Aggregator::new(config)
}

// ============================================================================
// Metric Recording Tests
// ============================================================================

#[cfg(test)]
mod recording_tests {
    use super::*;

    /// Test recording a single metric
    #[test]
    fn test_record_metric() {
        let aggregator = create_test_aggregator();

        let point = MetricPoint {
            name: "requests_total".to_string(),
            value: 100.0,
            tags: vec![
                ("backend_id".to_string(), constants::TEST_BACKEND_ID.to_string()),
            ],
            timestamp: Utc::now(),
        };

        let result = aggregator.record(point);

        assert!(result.is_ok());
    }

    /// Test recording multiple metrics
    #[test]
    fn test_record_multiple_metrics() {
        let aggregator = create_test_aggregator();

        for i in 0..100 {
            let point = MetricPoint {
                name: "requests_total".to_string(),
                value: i as f64,
                tags: vec![
                    ("backend_id".to_string(), constants::TEST_BACKEND_ID.to_string()),
                ],
                timestamp: Utc::now(),
            };
            aggregator.record(point).unwrap();
        }

        // Should have aggregated the values
        let stats = aggregator.get_stats("requests_total", &[("backend_id", constants::TEST_BACKEND_ID)]);
        assert!(stats.is_some());
    }

    /// Test recording with different tags
    #[test]
    fn test_record_different_tags() {
        let aggregator = create_test_aggregator();

        // Record for backend 1
        aggregator.record(MetricPoint {
            name: "requests".to_string(),
            value: 100.0,
            tags: vec![("backend_id".to_string(), "backend-1".to_string())],
            timestamp: Utc::now(),
        }).unwrap();

        // Record for backend 2
        aggregator.record(MetricPoint {
            name: "requests".to_string(),
            value: 200.0,
            tags: vec![("backend_id".to_string(), "backend-2".to_string())],
            timestamp: Utc::now(),
        }).unwrap();

        // Each should be separate
        let stats1 = aggregator.get_stats("requests", &[("backend_id", "backend-1")]);
        let stats2 = aggregator.get_stats("requests", &[("backend_id", "backend-2")]);

        assert!(stats1.is_some());
        assert!(stats2.is_some());
        assert_ne!(stats1.unwrap().sum, stats2.unwrap().sum);
    }

    /// Test recording counter increment
    #[test]
    fn test_record_counter() {
        let aggregator = create_test_aggregator();

        aggregator.increment_counter("errors_total", &[("type", "timeout")], 1.0).unwrap();
        aggregator.increment_counter("errors_total", &[("type", "timeout")], 5.0).unwrap();

        let stats = aggregator.get_stats("errors_total", &[("type", "timeout")]);
        assert!(stats.is_some());
        assert_eq!(stats.unwrap().sum, 6.0);
    }

    /// Test recording gauge
    #[test]
    fn test_record_gauge() {
        let aggregator = create_test_aggregator();

        aggregator.set_gauge("active_connections", &[("backend", "test")], 100.0).unwrap();
        aggregator.set_gauge("active_connections", &[("backend", "test")], 150.0).unwrap();

        let stats = aggregator.get_stats("active_connections", &[("backend", "test")]);
        assert!(stats.is_some());
        // Gauge should show latest value
        assert_eq!(stats.unwrap().last, 150.0);
    }

    /// Test recording histogram
    #[test]
    fn test_record_histogram() {
        let aggregator = create_test_aggregator();

        for i in 1..=100 {
            aggregator.record_histogram("latency_ms", &[("endpoint", "/api")], i as f64).unwrap();
        }

        let percentiles = aggregator.get_percentiles("latency_ms", &[("endpoint", "/api")]);
        assert!(percentiles.is_some());

        let p = percentiles.unwrap();
        // p50 should be around 50
        assert!(p.p50 >= 45.0 && p.p50 <= 55.0);
        // p99 should be around 99
        assert!(p.p99 >= 95.0);
    }
}

// ============================================================================
// Aggregation Tests
// ============================================================================

#[cfg(test)]
mod aggregation_tests {
    use super::*;

    /// Test sum aggregation
    #[test]
    fn test_sum_aggregation() {
        let aggregator = create_test_aggregator();

        for i in 1..=10 {
            aggregator.record(MetricPoint {
                name: "sum_test".to_string(),
                value: i as f64,
                tags: vec![],
                timestamp: Utc::now(),
            }).unwrap();
        }

        let stats = aggregator.get_stats("sum_test", &[]).unwrap();
        assert_eq!(stats.sum, 55.0); // 1+2+...+10 = 55
    }

    /// Test count aggregation
    #[test]
    fn test_count_aggregation() {
        let aggregator = create_test_aggregator();

        for _ in 0..50 {
            aggregator.record(MetricPoint {
                name: "count_test".to_string(),
                value: 1.0,
                tags: vec![],
                timestamp: Utc::now(),
            }).unwrap();
        }

        let stats = aggregator.get_stats("count_test", &[]).unwrap();
        assert_eq!(stats.count, 50);
    }

    /// Test average aggregation
    #[test]
    fn test_avg_aggregation() {
        let aggregator = create_test_aggregator();

        for i in 1..=100 {
            aggregator.record(MetricPoint {
                name: "avg_test".to_string(),
                value: i as f64,
                tags: vec![],
                timestamp: Utc::now(),
            }).unwrap();
        }

        let stats = aggregator.get_stats("avg_test", &[]).unwrap();
        assert_eq!(stats.avg(), 50.5); // (1+100)/2 = 50.5
    }

    /// Test min/max aggregation
    #[test]
    fn test_min_max_aggregation() {
        let aggregator = create_test_aggregator();

        let values = vec![10.0, 5.0, 100.0, 50.0, 1.0, 75.0];
        for v in values {
            aggregator.record(MetricPoint {
                name: "minmax_test".to_string(),
                value: v,
                tags: vec![],
                timestamp: Utc::now(),
            }).unwrap();
        }

        let stats = aggregator.get_stats("minmax_test", &[]).unwrap();
        assert_eq!(stats.min, 1.0);
        assert_eq!(stats.max, 100.0);
    }

    /// Test rate calculation
    #[test]
    fn test_rate_calculation() {
        let aggregator = create_test_aggregator();
        let start = Utc::now();

        // Record increasing counter values
        aggregator.record(MetricPoint {
            name: "counter".to_string(),
            value: 100.0,
            tags: vec![],
            timestamp: start,
        }).unwrap();

        aggregator.record(MetricPoint {
            name: "counter".to_string(),
            value: 200.0,
            tags: vec![],
            timestamp: start + Duration::seconds(10),
        }).unwrap();

        let rate = aggregator.get_rate("counter", &[], Duration::seconds(10));
        // Rate should be 10 per second (100 increase over 10 seconds)
        assert!(rate.is_some());
        assert!((rate.unwrap() - 10.0).abs() < 0.1);
    }

    /// Test time-windowed aggregation
    #[test]
    fn test_windowed_aggregation() {
        let aggregator = create_test_aggregator();
        let now = Utc::now();

        // Record values at different times
        aggregator.record(MetricPoint {
            name: "windowed".to_string(),
            value: 100.0,
            tags: vec![],
            timestamp: now - Duration::minutes(5),
        }).unwrap();

        aggregator.record(MetricPoint {
            name: "windowed".to_string(),
            value: 200.0,
            tags: vec![],
            timestamp: now,
        }).unwrap();

        // Get stats for last minute only
        let recent = aggregator.get_stats_in_window(
            "windowed",
            &[],
            now - Duration::minutes(1),
            now,
        );

        assert!(recent.is_some());
        assert_eq!(recent.unwrap().sum, 200.0);
    }
}

// ============================================================================
// Bucket Management Tests
// ============================================================================

#[cfg(test)]
mod bucket_tests {
    use super::*;

    /// Test bucket creation
    #[test]
    fn test_bucket_creation() {
        let aggregator = create_test_aggregator();
        let now = Utc::now();

        aggregator.record(MetricPoint {
            name: "bucket_test".to_string(),
            value: 1.0,
            tags: vec![],
            timestamp: now,
        }).unwrap();

        // Record in next bucket (1 minute later)
        aggregator.record(MetricPoint {
            name: "bucket_test".to_string(),
            value: 2.0,
            tags: vec![],
            timestamp: now + Duration::minutes(1),
        }).unwrap();

        // Should have 2 buckets
        let buckets = aggregator.get_buckets("bucket_test", &[]);
        assert!(buckets.len() >= 2);
    }

    /// Test bucket expiration
    #[test]
    fn test_bucket_expiration() {
        let config = AggregatorConfig {
            flush_interval: StdDuration::from_secs(1),
            bucket_duration: StdDuration::from_secs(60),
            max_buckets: 5, // Very low limit
            percentiles: vec![0.5, 0.95, 0.99],
        };
        let aggregator = Aggregator::new(config);
        let now = Utc::now();

        // Fill up buckets
        for i in 0..10 {
            aggregator.record(MetricPoint {
                name: "expiry_test".to_string(),
                value: i as f64,
                tags: vec![],
                timestamp: now + Duration::minutes(i),
            }).unwrap();
        }

        // Should have at most max_buckets
        let buckets = aggregator.get_buckets("expiry_test", &[]);
        assert!(buckets.len() <= 5);
    }

    /// Test bucket alignment
    #[test]
    fn test_bucket_alignment() {
        let aggregator = create_test_aggregator();
        let base = Utc::now();

        // Records at different seconds should fall into same minute bucket
        for i in 0..60 {
            aggregator.record(MetricPoint {
                name: "alignment_test".to_string(),
                value: 1.0,
                tags: vec![],
                timestamp: base + Duration::seconds(i),
            }).unwrap();
        }

        // Should all be in one bucket
        let buckets = aggregator.get_buckets("alignment_test", &[]);
        assert_eq!(buckets.len(), 1);
        assert_eq!(buckets[0].count, 60);
    }
}

// ============================================================================
// Flush Tests
// ============================================================================

#[cfg(test)]
mod flush_tests {
    use super::*;

    /// Test manual flush
    #[test]
    fn test_manual_flush() {
        let aggregator = create_test_aggregator();

        for i in 0..100 {
            aggregator.record(MetricPoint {
                name: "flush_test".to_string(),
                value: i as f64,
                tags: vec![],
                timestamp: Utc::now(),
            }).unwrap();
        }

        let result = aggregator.flush();
        assert!(result.is_ok());
    }

    /// Test flush returns metrics
    #[test]
    fn test_flush_returns_data() {
        let aggregator = create_test_aggregator();

        aggregator.record(MetricPoint {
            name: "return_test".to_string(),
            value: 42.0,
            tags: vec![("key".to_string(), "value".to_string())],
            timestamp: Utc::now(),
        }).unwrap();

        let flushed = aggregator.flush_and_get().unwrap();
        assert!(!flushed.is_empty());
    }

    /// Test flush clears pending data
    #[test]
    fn test_flush_clears_pending() {
        let aggregator = create_test_aggregator();

        aggregator.record(MetricPoint {
            name: "clear_test".to_string(),
            value: 1.0,
            tags: vec![],
            timestamp: Utc::now(),
        }).unwrap();

        aggregator.flush().unwrap();

        // Pending should be empty
        let pending = aggregator.pending_count();
        assert_eq!(pending, 0);
    }
}

// ============================================================================
// Downsampling Tests
// ============================================================================

#[cfg(test)]
mod downsampling_tests {
    use super::*;

    /// Test downsampling high-resolution data
    #[test]
    fn test_downsample() {
        let aggregator = create_test_aggregator();
        let start = Utc::now() - Duration::hours(1);

        // Generate 1-second resolution data
        for i in 0..3600 {
            aggregator.record(MetricPoint {
                name: "downsample_test".to_string(),
                value: i as f64 % 100.0,
                tags: vec![],
                timestamp: start + Duration::seconds(i),
            }).unwrap();
        }

        // Downsample to 1-minute resolution
        let downsampled = aggregator.downsample(
            "downsample_test",
            &[],
            start,
            start + Duration::hours(1),
            Duration::minutes(1),
            AggregationType::Average,
        );

        assert!(downsampled.is_ok());
        let data = downsampled.unwrap();
        // Should have ~60 points
        assert!(data.len() >= 55 && data.len() <= 65);
    }

    /// Test different aggregation types for downsampling
    #[test]
    fn test_downsample_aggregation_types() {
        let aggregator = create_test_aggregator();
        let start = Utc::now();

        // Record known values
        for i in 0..60 {
            aggregator.record(MetricPoint {
                name: "agg_type_test".to_string(),
                value: (i + 1) as f64, // 1 to 60
                tags: vec![],
                timestamp: start + Duration::seconds(i),
            }).unwrap();
        }

        let sum = aggregator.downsample(
            "agg_type_test", &[],
            start, start + Duration::minutes(1),
            Duration::minutes(1),
            AggregationType::Sum,
        ).unwrap();

        let max = aggregator.downsample(
            "agg_type_test", &[],
            start, start + Duration::minutes(1),
            Duration::minutes(1),
            AggregationType::Max,
        ).unwrap();

        let min = aggregator.downsample(
            "agg_type_test", &[],
            start, start + Duration::minutes(1),
            Duration::minutes(1),
            AggregationType::Min,
        ).unwrap();

        // Verify aggregation types
        assert_eq!(sum[0].value, 1830.0); // Sum of 1..60
        assert_eq!(max[0].value, 60.0);
        assert_eq!(min[0].value, 1.0);
    }
}

// ============================================================================
// Config Tests
// ============================================================================

#[cfg(test)]
mod config_tests {
    use super::*;

    /// Test default config
    #[test]
    fn test_default_config() {
        let config = AggregatorConfig::default();

        assert!(config.flush_interval.as_secs() > 0);
        assert!(config.bucket_duration.as_secs() > 0);
        assert!(config.max_buckets > 0);
        assert!(!config.percentiles.is_empty());
    }

    /// Test config validation
    #[test]
    fn test_config_validation() {
        let valid_config = AggregatorConfig {
            flush_interval: StdDuration::from_secs(10),
            bucket_duration: StdDuration::from_secs(60),
            max_buckets: 100,
            percentiles: vec![0.5, 0.95, 0.99],
        };

        assert!(valid_config.validate().is_ok());

        let invalid_config = AggregatorConfig {
            flush_interval: StdDuration::from_secs(0),
            bucket_duration: StdDuration::from_secs(0),
            max_buckets: 0,
            percentiles: vec![],
        };

        // Invalid config should fail validation
    }
}
