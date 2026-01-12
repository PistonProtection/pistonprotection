//! Time-series storage tests

use super::test_utils::{TestTrafficMetrics, constants, generate_time_series};
use crate::storage::{
    MetricStorage, RetentionPolicy, StorageBackend, StorageConfig, TimeSeriesPoint,
};
use chrono::{Duration, Utc};
use std::time::Duration as StdDuration;

/// Create a test storage with in-memory backend
fn create_test_storage() -> MetricStorage {
    let config = StorageConfig {
        backend: StorageBackend::Memory,
        retention: RetentionPolicy {
            raw_retention: StdDuration::from_secs(3600),     // 1 hour
            hourly_retention: StdDuration::from_secs(86400), // 1 day
            daily_retention: StdDuration::from_secs(604800), // 1 week
        },
        write_buffer_size: 1000,
        max_points_per_query: 10000,
    };
    MetricStorage::new(config)
}

// ============================================================================
// Write Tests
// ============================================================================

#[cfg(test)]
mod write_tests {
    use super::*;

    /// Test writing a single point
    #[tokio::test]
    async fn test_write_single_point() {
        let storage = create_test_storage();

        let point = TimeSeriesPoint {
            metric: "test_metric".to_string(),
            value: 42.0,
            timestamp: Utc::now(),
            tags: vec![("backend".to_string(), "test".to_string())],
        };

        let result = storage.write(point).await;

        assert!(result.is_ok());
    }

    /// Test writing batch of points
    #[tokio::test]
    async fn test_write_batch() {
        let storage = create_test_storage();

        let points: Vec<TimeSeriesPoint> = (0..100)
            .map(|i| TimeSeriesPoint {
                metric: "batch_metric".to_string(),
                value: i as f64,
                timestamp: Utc::now(),
                tags: vec![("index".to_string(), i.to_string())],
            })
            .collect();

        let result = storage.write_batch(points).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 100);
    }

    /// Test write with buffer overflow
    #[tokio::test]
    async fn test_write_buffer_flush() {
        let config = StorageConfig {
            backend: StorageBackend::Memory,
            retention: RetentionPolicy::default(),
            write_buffer_size: 10, // Small buffer
            max_points_per_query: 10000,
        };
        let storage = MetricStorage::new(config);

        // Write more than buffer size
        for i in 0..50 {
            storage
                .write(TimeSeriesPoint {
                    metric: "buffer_test".to_string(),
                    value: i as f64,
                    timestamp: Utc::now(),
                    tags: vec![],
                })
                .await
                .unwrap();
        }

        // Should have flushed multiple times
        let count = storage.point_count("buffer_test").await.unwrap();
        assert_eq!(count, 50);
    }

    /// Test concurrent writes
    #[tokio::test]
    async fn test_concurrent_writes() {
        let storage = create_test_storage();

        let mut handles = vec![];

        for i in 0..10 {
            let s = storage.clone();
            handles.push(tokio::spawn(async move {
                for j in 0..100 {
                    s.write(TimeSeriesPoint {
                        metric: format!("concurrent_{}", i),
                        value: j as f64,
                        timestamp: Utc::now(),
                        tags: vec![],
                    })
                    .await
                    .unwrap();
                }
            }));
        }

        for handle in handles {
            handle.await.unwrap();
        }

        // Should have written all points
        let mut total = 0;
        for i in 0..10 {
            total += storage
                .point_count(&format!("concurrent_{}", i))
                .await
                .unwrap();
        }
        assert_eq!(total, 1000);
    }
}

// ============================================================================
// Read Tests
// ============================================================================

#[cfg(test)]
mod read_tests {
    use super::*;

    /// Test reading points
    #[tokio::test]
    async fn test_read_points() {
        let storage = create_test_storage();
        let start = Utc::now() - Duration::minutes(10);

        // Write points
        for i in 0..100 {
            storage
                .write(TimeSeriesPoint {
                    metric: "read_test".to_string(),
                    value: i as f64,
                    timestamp: start + Duration::seconds(i * 6),
                    tags: vec![("backend".to_string(), "test".to_string())],
                })
                .await
                .unwrap();
        }

        // Read points
        let result = storage
            .read("read_test", &[("backend", "test")], start, Utc::now())
            .await;

        assert!(result.is_ok());
        let points = result.unwrap();
        assert_eq!(points.len(), 100);
    }

    /// Test reading with time range
    #[tokio::test]
    async fn test_read_time_range() {
        let storage = create_test_storage();
        let start = Utc::now() - Duration::minutes(10);

        // Write points
        for i in 0..100 {
            storage
                .write(TimeSeriesPoint {
                    metric: "range_test".to_string(),
                    value: i as f64,
                    timestamp: start + Duration::seconds(i * 6),
                    tags: vec![],
                })
                .await
                .unwrap();
        }

        // Read only middle portion
        let result = storage
            .read(
                "range_test",
                &[],
                start + Duration::minutes(2),
                start + Duration::minutes(5),
            )
            .await;

        assert!(result.is_ok());
        let points = result.unwrap();
        // Should have roughly 30 points (3 minutes at 1 point per 6 seconds)
        assert!(points.len() >= 25 && points.len() <= 35);
    }

    /// Test reading with tag filter
    #[tokio::test]
    async fn test_read_tag_filter() {
        let storage = create_test_storage();

        // Write points with different tags
        for i in 0..50 {
            storage
                .write(TimeSeriesPoint {
                    metric: "tag_test".to_string(),
                    value: i as f64,
                    timestamp: Utc::now(),
                    tags: vec![("type".to_string(), "a".to_string())],
                })
                .await
                .unwrap();
        }

        for i in 0..30 {
            storage
                .write(TimeSeriesPoint {
                    metric: "tag_test".to_string(),
                    value: i as f64,
                    timestamp: Utc::now(),
                    tags: vec![("type".to_string(), "b".to_string())],
                })
                .await
                .unwrap();
        }

        // Read only type=a
        let result = storage
            .read(
                "tag_test",
                &[("type", "a")],
                Utc::now() - Duration::minutes(1),
                Utc::now(),
            )
            .await;

        assert!(result.is_ok());
        let points = result.unwrap();
        assert_eq!(points.len(), 50);
    }

    /// Test reading non-existent metric
    #[tokio::test]
    async fn test_read_nonexistent() {
        let storage = create_test_storage();

        let result = storage
            .read(
                "nonexistent_metric",
                &[],
                Utc::now() - Duration::hours(1),
                Utc::now(),
            )
            .await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    /// Test read limit
    #[tokio::test]
    async fn test_read_limit() {
        let config = StorageConfig {
            backend: StorageBackend::Memory,
            retention: RetentionPolicy::default(),
            write_buffer_size: 1000,
            max_points_per_query: 10, // Low limit
        };
        let storage = MetricStorage::new(config);

        // Write many points
        for i in 0..100 {
            storage
                .write(TimeSeriesPoint {
                    metric: "limit_test".to_string(),
                    value: i as f64,
                    timestamp: Utc::now(),
                    tags: vec![],
                })
                .await
                .unwrap();
        }

        // Read should be limited
        let result = storage
            .read(
                "limit_test",
                &[],
                Utc::now() - Duration::minutes(1),
                Utc::now(),
            )
            .await;

        assert!(result.is_ok());
        let points = result.unwrap();
        assert!(points.len() <= 10);
    }
}

// ============================================================================
// Aggregation Query Tests
// ============================================================================

#[cfg(test)]
mod aggregation_query_tests {
    use super::*;

    /// Test sum query
    #[tokio::test]
    async fn test_sum_query() {
        let storage = create_test_storage();

        // Write known values
        for i in 1..=10 {
            storage
                .write(TimeSeriesPoint {
                    metric: "sum_query".to_string(),
                    value: i as f64,
                    timestamp: Utc::now(),
                    tags: vec![],
                })
                .await
                .unwrap();
        }

        let result = storage
            .sum(
                "sum_query",
                &[],
                Utc::now() - Duration::minutes(1),
                Utc::now(),
            )
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 55.0);
    }

    /// Test average query
    #[tokio::test]
    async fn test_avg_query() {
        let storage = create_test_storage();

        // Write known values
        for i in 1..=100 {
            storage
                .write(TimeSeriesPoint {
                    metric: "avg_query".to_string(),
                    value: i as f64,
                    timestamp: Utc::now(),
                    tags: vec![],
                })
                .await
                .unwrap();
        }

        let result = storage
            .avg(
                "avg_query",
                &[],
                Utc::now() - Duration::minutes(1),
                Utc::now(),
            )
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 50.5);
    }

    /// Test min/max query
    #[tokio::test]
    async fn test_min_max_query() {
        let storage = create_test_storage();

        for v in vec![50.0, 10.0, 100.0, 25.0, 75.0] {
            storage
                .write(TimeSeriesPoint {
                    metric: "minmax_query".to_string(),
                    value: v,
                    timestamp: Utc::now(),
                    tags: vec![],
                })
                .await
                .unwrap();
        }

        let min = storage
            .min(
                "minmax_query",
                &[],
                Utc::now() - Duration::minutes(1),
                Utc::now(),
            )
            .await
            .unwrap();

        let max = storage
            .max(
                "minmax_query",
                &[],
                Utc::now() - Duration::minutes(1),
                Utc::now(),
            )
            .await
            .unwrap();

        assert_eq!(min, 10.0);
        assert_eq!(max, 100.0);
    }

    /// Test count query
    #[tokio::test]
    async fn test_count_query() {
        let storage = create_test_storage();

        for i in 0..42 {
            storage
                .write(TimeSeriesPoint {
                    metric: "count_query".to_string(),
                    value: i as f64,
                    timestamp: Utc::now(),
                    tags: vec![],
                })
                .await
                .unwrap();
        }

        let result = storage
            .count(
                "count_query",
                &[],
                Utc::now() - Duration::minutes(1),
                Utc::now(),
            )
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    /// Test percentile query
    #[tokio::test]
    async fn test_percentile_query() {
        let storage = create_test_storage();

        for i in 1..=100 {
            storage
                .write(TimeSeriesPoint {
                    metric: "percentile_query".to_string(),
                    value: i as f64,
                    timestamp: Utc::now(),
                    tags: vec![],
                })
                .await
                .unwrap();
        }

        let p50 = storage
            .percentile(
                "percentile_query",
                &[],
                Utc::now() - Duration::minutes(1),
                Utc::now(),
                0.5,
            )
            .await
            .unwrap();

        let p99 = storage
            .percentile(
                "percentile_query",
                &[],
                Utc::now() - Duration::minutes(1),
                Utc::now(),
                0.99,
            )
            .await
            .unwrap();

        assert!(p50 >= 49.0 && p50 <= 51.0);
        assert!(p99 >= 98.0);
    }
}

// ============================================================================
// Retention Tests
// ============================================================================

#[cfg(test)]
mod retention_tests {
    use super::*;

    /// Test expired data cleanup
    #[tokio::test]
    async fn test_retention_cleanup() {
        let config = StorageConfig {
            backend: StorageBackend::Memory,
            retention: RetentionPolicy {
                raw_retention: StdDuration::from_secs(60), // 1 minute
                hourly_retention: StdDuration::from_secs(3600),
                daily_retention: StdDuration::from_secs(86400),
            },
            write_buffer_size: 1000,
            max_points_per_query: 10000,
        };
        let storage = MetricStorage::new(config);

        // Write old data
        storage
            .write(TimeSeriesPoint {
                metric: "retention_test".to_string(),
                value: 1.0,
                timestamp: Utc::now() - Duration::minutes(5), // Old
                tags: vec![],
            })
            .await
            .unwrap();

        // Write recent data
        storage
            .write(TimeSeriesPoint {
                metric: "retention_test".to_string(),
                value: 2.0,
                timestamp: Utc::now(), // Recent
                tags: vec![],
            })
            .await
            .unwrap();

        // Run cleanup
        storage.cleanup().await.unwrap();

        // Old data should be removed
        let points = storage
            .read(
                "retention_test",
                &[],
                Utc::now() - Duration::hours(1),
                Utc::now(),
            )
            .await
            .unwrap();

        assert_eq!(points.len(), 1);
        assert_eq!(points[0].value, 2.0);
    }

    /// Test downsampling for retention
    #[tokio::test]
    async fn test_retention_downsampling() {
        let storage = create_test_storage();
        let start = Utc::now() - Duration::hours(2);

        // Write high-resolution data
        for i in 0..7200 {
            storage
                .write(TimeSeriesPoint {
                    metric: "downsample_retention".to_string(),
                    value: (i % 100) as f64,
                    timestamp: start + Duration::seconds(i),
                    tags: vec![],
                })
                .await
                .unwrap();
        }

        // Trigger downsampling
        storage.downsample_old_data().await.unwrap();

        // Old data should be downsampled (fewer points)
        let points = storage
            .read(
                "downsample_retention",
                &[],
                start,
                start + Duration::hours(1),
            )
            .await
            .unwrap();

        // Should have fewer than 3600 points after downsampling
        assert!(points.len() < 3600);
    }
}

// ============================================================================
// Listing Tests
// ============================================================================

#[cfg(test)]
mod listing_tests {
    use super::*;

    /// Test listing metrics
    #[tokio::test]
    async fn test_list_metrics() {
        let storage = create_test_storage();

        storage
            .write(TimeSeriesPoint {
                metric: "metric_a".to_string(),
                value: 1.0,
                timestamp: Utc::now(),
                tags: vec![],
            })
            .await
            .unwrap();

        storage
            .write(TimeSeriesPoint {
                metric: "metric_b".to_string(),
                value: 2.0,
                timestamp: Utc::now(),
                tags: vec![],
            })
            .await
            .unwrap();

        let metrics = storage.list_metrics().await.unwrap();

        assert!(metrics.contains(&"metric_a".to_string()));
        assert!(metrics.contains(&"metric_b".to_string()));
    }

    /// Test listing tag values
    #[tokio::test]
    async fn test_list_tag_values() {
        let storage = create_test_storage();

        for region in vec!["us-east", "us-west", "eu-central"] {
            storage
                .write(TimeSeriesPoint {
                    metric: "tag_values_test".to_string(),
                    value: 1.0,
                    timestamp: Utc::now(),
                    tags: vec![("region".to_string(), region.to_string())],
                })
                .await
                .unwrap();
        }

        let values = storage
            .list_tag_values("tag_values_test", "region")
            .await
            .unwrap();

        assert!(values.contains(&"us-east".to_string()));
        assert!(values.contains(&"us-west".to_string()));
        assert!(values.contains(&"eu-central".to_string()));
    }
}
