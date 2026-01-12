//! Streaming metrics tests

use super::test_utils::{constants, TestTrafficMetrics};
use crate::streaming::{
    MetricStream, StreamConfig, StreamFilter, StreamSubscription, StreamingService,
};
use chrono::{Duration, Utc};
use std::time::Duration as StdDuration;
use tokio::sync::mpsc;
use tokio_stream::StreamExt;

/// Create a test streaming service
fn create_test_streaming() -> StreamingService {
    let config = StreamConfig {
        max_subscribers: 100,
        buffer_size: 1000,
        heartbeat_interval: StdDuration::from_secs(30),
        max_batch_size: 100,
    };
    StreamingService::new(config)
}

// ============================================================================
// Subscription Tests
// ============================================================================

#[cfg(test)]
mod subscription_tests {
    use super::*;

    /// Test subscribing to metrics
    #[tokio::test]
    async fn test_subscribe() {
        let service = create_test_streaming();

        let filter = StreamFilter {
            backend_ids: vec![constants::TEST_BACKEND_ID.to_string()],
            metrics: vec![],
            min_interval: StdDuration::from_millis(100),
        };

        let result = service.subscribe(filter).await;

        assert!(result.is_ok());
        let subscription = result.unwrap();
        assert!(!subscription.id.is_empty());
    }

    /// Test subscribing with filters
    #[tokio::test]
    async fn test_subscribe_with_filters() {
        let service = create_test_streaming();

        let filter = StreamFilter {
            backend_ids: vec!["backend-1".to_string(), "backend-2".to_string()],
            metrics: vec!["requests_total".to_string(), "bytes_in".to_string()],
            min_interval: StdDuration::from_millis(100),
        };

        let result = service.subscribe(filter).await;

        assert!(result.is_ok());
    }

    /// Test unsubscribing
    #[tokio::test]
    async fn test_unsubscribe() {
        let service = create_test_streaming();

        let filter = StreamFilter {
            backend_ids: vec![],
            metrics: vec![],
            min_interval: StdDuration::from_millis(100),
        };

        let subscription = service.subscribe(filter).await.unwrap();
        let subscription_id = subscription.id.clone();

        let result = service.unsubscribe(&subscription_id).await;

        assert!(result.is_ok());

        // Should not be able to receive after unsubscribe
    }

    /// Test max subscribers limit
    #[tokio::test]
    async fn test_max_subscribers() {
        let config = StreamConfig {
            max_subscribers: 2,
            buffer_size: 100,
            heartbeat_interval: StdDuration::from_secs(30),
            max_batch_size: 100,
        };
        let service = StreamingService::new(config);

        let filter = StreamFilter {
            backend_ids: vec![],
            metrics: vec![],
            min_interval: StdDuration::from_millis(100),
        };

        // Subscribe up to limit
        service.subscribe(filter.clone()).await.unwrap();
        service.subscribe(filter.clone()).await.unwrap();

        // Third subscription should fail
        let result = service.subscribe(filter).await;

        assert!(result.is_err());
    }

    /// Test listing active subscriptions
    #[tokio::test]
    async fn test_list_subscriptions() {
        let service = create_test_streaming();

        let filter = StreamFilter {
            backend_ids: vec![],
            metrics: vec![],
            min_interval: StdDuration::from_millis(100),
        };

        service.subscribe(filter.clone()).await.unwrap();
        service.subscribe(filter.clone()).await.unwrap();

        let subscriptions = service.list_subscriptions().await;

        assert_eq!(subscriptions.len(), 2);
    }
}

// ============================================================================
// Publishing Tests
// ============================================================================

#[cfg(test)]
mod publishing_tests {
    use super::*;

    /// Test publishing metrics
    #[tokio::test]
    async fn test_publish_metrics() {
        let service = create_test_streaming();

        let filter = StreamFilter {
            backend_ids: vec![constants::TEST_BACKEND_ID.to_string()],
            metrics: vec![],
            min_interval: StdDuration::from_millis(0),
        };

        let subscription = service.subscribe(filter).await.unwrap();
        let mut stream = subscription.stream;

        // Publish a metric
        let metrics = TestTrafficMetrics::new();
        service.publish(constants::TEST_BACKEND_ID, &metrics).await;

        // Receive should work
        let received = tokio::time::timeout(
            StdDuration::from_secs(1),
            stream.next(),
        ).await;

        assert!(received.is_ok());
        assert!(received.unwrap().is_some());
    }

    /// Test filtering by backend
    #[tokio::test]
    async fn test_publish_filter_backend() {
        let service = create_test_streaming();

        // Subscribe only to backend-1
        let filter = StreamFilter {
            backend_ids: vec!["backend-1".to_string()],
            metrics: vec![],
            min_interval: StdDuration::from_millis(0),
        };

        let subscription = service.subscribe(filter).await.unwrap();
        let mut stream = subscription.stream;

        // Publish to backend-2 (should not receive)
        let metrics = TestTrafficMetrics::new().with_backend_id("backend-2");
        service.publish("backend-2", &metrics).await;

        // Should timeout (no message)
        let received = tokio::time::timeout(
            StdDuration::from_millis(100),
            stream.next(),
        ).await;

        assert!(received.is_err()); // Timeout means no message

        // Publish to backend-1 (should receive)
        let metrics = TestTrafficMetrics::new().with_backend_id("backend-1");
        service.publish("backend-1", &metrics).await;

        let received = tokio::time::timeout(
            StdDuration::from_secs(1),
            stream.next(),
        ).await;

        assert!(received.is_ok());
    }

    /// Test filtering by metric name
    #[tokio::test]
    async fn test_publish_filter_metrics() {
        let service = create_test_streaming();

        let filter = StreamFilter {
            backend_ids: vec![],
            metrics: vec!["requests_total".to_string()],
            min_interval: StdDuration::from_millis(0),
        };

        let subscription = service.subscribe(filter).await.unwrap();
        let mut stream = subscription.stream;

        // Publish matching metric
        service.publish_metric(
            constants::TEST_BACKEND_ID,
            "requests_total",
            1000.0,
        ).await;

        let received = tokio::time::timeout(
            StdDuration::from_secs(1),
            stream.next(),
        ).await;

        assert!(received.is_ok());
    }

    /// Test rate limiting
    #[tokio::test]
    async fn test_publish_rate_limit() {
        let service = create_test_streaming();

        let filter = StreamFilter {
            backend_ids: vec![],
            metrics: vec![],
            min_interval: StdDuration::from_millis(100), // Max 10 per second
        };

        let subscription = service.subscribe(filter).await.unwrap();
        let mut stream = subscription.stream;

        // Publish rapidly
        for i in 0..20 {
            let metrics = TestTrafficMetrics::new();
            service.publish(constants::TEST_BACKEND_ID, &metrics).await;
        }

        // Should receive fewer due to rate limiting
        let mut count = 0;
        loop {
            let received = tokio::time::timeout(
                StdDuration::from_millis(50),
                stream.next(),
            ).await;

            if received.is_err() {
                break;
            }
            count += 1;
        }

        // Should receive some but not all 20
        assert!(count > 0);
        assert!(count < 20);
    }

    /// Test batch publishing
    #[tokio::test]
    async fn test_publish_batch() {
        let service = create_test_streaming();

        let filter = StreamFilter {
            backend_ids: vec![],
            metrics: vec![],
            min_interval: StdDuration::from_millis(0),
        };

        let subscription = service.subscribe(filter).await.unwrap();
        let mut stream = subscription.stream;

        // Publish batch
        let batch: Vec<_> = (0..10)
            .map(|i| {
                (constants::TEST_BACKEND_ID.to_string(), TestTrafficMetrics::new())
            })
            .collect();

        service.publish_batch(batch).await;

        // Should receive all
        let mut count = 0;
        loop {
            let received = tokio::time::timeout(
                StdDuration::from_millis(100),
                stream.next(),
            ).await;

            if received.is_err() {
                break;
            }
            count += 1;
        }

        assert_eq!(count, 10);
    }
}

// ============================================================================
// Stream Tests
// ============================================================================

#[cfg(test)]
mod stream_tests {
    use super::*;

    /// Test stream is closeable
    #[tokio::test]
    async fn test_stream_close() {
        let service = create_test_streaming();

        let filter = StreamFilter {
            backend_ids: vec![],
            metrics: vec![],
            min_interval: StdDuration::from_millis(100),
        };

        let subscription = service.subscribe(filter).await.unwrap();
        let subscription_id = subscription.id.clone();

        // Close the stream
        drop(subscription);

        // Service should handle dropped receiver
        let metrics = TestTrafficMetrics::new();
        service.publish(constants::TEST_BACKEND_ID, &metrics).await;

        // Should not panic
    }

    /// Test multiple subscribers receive same data
    #[tokio::test]
    async fn test_multiple_subscribers() {
        let service = create_test_streaming();

        let filter = StreamFilter {
            backend_ids: vec![],
            metrics: vec![],
            min_interval: StdDuration::from_millis(0),
        };

        let sub1 = service.subscribe(filter.clone()).await.unwrap();
        let sub2 = service.subscribe(filter.clone()).await.unwrap();
        let mut stream1 = sub1.stream;
        let mut stream2 = sub2.stream;

        // Publish
        let metrics = TestTrafficMetrics::new();
        service.publish(constants::TEST_BACKEND_ID, &metrics).await;

        // Both should receive
        let recv1 = tokio::time::timeout(
            StdDuration::from_secs(1),
            stream1.next(),
        ).await;
        let recv2 = tokio::time::timeout(
            StdDuration::from_secs(1),
            stream2.next(),
        ).await;

        assert!(recv1.is_ok());
        assert!(recv2.is_ok());
    }

    /// Test buffer overflow handling
    #[tokio::test]
    async fn test_buffer_overflow() {
        let config = StreamConfig {
            max_subscribers: 100,
            buffer_size: 5, // Very small buffer
            heartbeat_interval: StdDuration::from_secs(30),
            max_batch_size: 100,
        };
        let service = StreamingService::new(config);

        let filter = StreamFilter {
            backend_ids: vec![],
            metrics: vec![],
            min_interval: StdDuration::from_millis(0),
        };

        let subscription = service.subscribe(filter).await.unwrap();
        // Don't read from stream - let buffer fill

        // Publish more than buffer size
        for _ in 0..20 {
            let metrics = TestTrafficMetrics::new();
            service.publish(constants::TEST_BACKEND_ID, &metrics).await;
        }

        // Should handle overflow gracefully (drop old messages)
        // and not panic
    }
}

// ============================================================================
// Heartbeat Tests
// ============================================================================

#[cfg(test)]
mod heartbeat_tests {
    use super::*;

    /// Test heartbeat is sent
    #[tokio::test]
    async fn test_heartbeat() {
        let config = StreamConfig {
            max_subscribers: 100,
            buffer_size: 1000,
            heartbeat_interval: StdDuration::from_millis(100), // Fast heartbeat
            max_batch_size: 100,
        };
        let service = StreamingService::new(config);

        let filter = StreamFilter {
            backend_ids: vec![],
            metrics: vec![],
            min_interval: StdDuration::from_millis(0),
        };

        let subscription = service.subscribe(filter).await.unwrap();
        let mut stream = subscription.stream;

        // Wait for heartbeat
        let received = tokio::time::timeout(
            StdDuration::from_millis(500),
            stream.next(),
        ).await;

        // Should receive heartbeat (or actual data)
        assert!(received.is_ok());
    }
}

// ============================================================================
// Configuration Tests
// ============================================================================

#[cfg(test)]
mod config_tests {
    use super::*;

    /// Test default config
    #[test]
    fn test_default_config() {
        let config = StreamConfig::default();

        assert!(config.max_subscribers > 0);
        assert!(config.buffer_size > 0);
        assert!(config.heartbeat_interval.as_secs() > 0);
    }

    /// Test config validation
    #[test]
    fn test_config_validation() {
        let valid_config = StreamConfig {
            max_subscribers: 100,
            buffer_size: 1000,
            heartbeat_interval: StdDuration::from_secs(30),
            max_batch_size: 100,
        };

        assert!(valid_config.validate().is_ok());

        let invalid_config = StreamConfig {
            max_subscribers: 0,
            buffer_size: 0,
            heartbeat_interval: StdDuration::from_secs(0),
            max_batch_size: 0,
        };

        // Invalid config should fail validation
    }
}
