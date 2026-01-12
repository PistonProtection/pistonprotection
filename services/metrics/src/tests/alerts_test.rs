//! Alert evaluation tests

use super::test_utils::{TestAlertConfig, constants};
use crate::alerts::{
    Alert, AlertCondition, AlertConfig, AlertManager, AlertNotification, AlertNotificationType,
    AlertOperator, AlertState,
};
use chrono::{Duration, Utc};
use std::collections::HashMap;
use std::time::Duration as StdDuration;

/// Create a test alert manager
fn create_test_alert_manager() -> AlertManager {
    let config = AlertConfig {
        eval_interval: StdDuration::from_secs(1),
        min_repeat_interval: StdDuration::from_secs(60),
        notification_retries: 3,
        notification_timeout: StdDuration::from_secs(5),
    };
    AlertManager::new_with_memory_store(config)
}

/// Create a test alert
fn create_test_alert(id: &str, metric: &str, threshold: f64) -> Alert {
    Alert {
        id: id.to_string(),
        backend_id: constants::TEST_BACKEND_ID.to_string(),
        name: format!("Test Alert {}", id),
        condition: Some(AlertCondition {
            metric: metric.to_string(),
            operator: AlertOperator::GreaterThan as i32,
            threshold,
            duration_seconds: 0,
        }),
        notifications: vec![],
        enabled: true,
        state: AlertState::Ok as i32,
        last_triggered: None,
        created_at: None,
        updated_at: None,
    }
}

// ============================================================================
// Alert CRUD Tests
// ============================================================================

#[cfg(test)]
mod crud_tests {
    use super::*;

    /// Test creating an alert
    #[tokio::test]
    async fn test_create_alert() {
        let manager = create_test_alert_manager();
        let alert = create_test_alert("alert-1", "requests_per_second", 1000.0);

        let result = manager
            .create_alert(constants::TEST_BACKEND_ID, alert)
            .await;

        assert!(result.is_ok());
        let created = result.unwrap();
        assert!(!created.id.is_empty());
        assert_eq!(created.backend_id, constants::TEST_BACKEND_ID);
    }

    /// Test getting an alert
    #[tokio::test]
    async fn test_get_alert() {
        let manager = create_test_alert_manager();
        let alert = create_test_alert("get-test", "rps", 500.0);

        let created = manager
            .create_alert(constants::TEST_BACKEND_ID, alert)
            .await
            .unwrap();

        let result = manager.get_alert(&created.id).await;

        assert!(result.is_ok());
        let retrieved = result.unwrap();
        assert_eq!(retrieved.id, created.id);
    }

    /// Test updating an alert
    #[tokio::test]
    async fn test_update_alert() {
        let manager = create_test_alert_manager();
        let alert = create_test_alert("update-test", "rps", 500.0);

        let created = manager
            .create_alert(constants::TEST_BACKEND_ID, alert)
            .await
            .unwrap();

        let mut updated = created.clone();
        updated.name = "Updated Alert Name".to_string();

        let result = manager.update_alert(updated).await;

        assert!(result.is_ok());
        let retrieved = result.unwrap();
        assert_eq!(retrieved.name, "Updated Alert Name");
    }

    /// Test deleting an alert
    #[tokio::test]
    async fn test_delete_alert() {
        let manager = create_test_alert_manager();
        let alert = create_test_alert("delete-test", "rps", 500.0);

        let created = manager
            .create_alert(constants::TEST_BACKEND_ID, alert)
            .await
            .unwrap();

        let result = manager.delete_alert(&created.id).await;
        assert!(result.is_ok());

        let get_result = manager.get_alert(&created.id).await;
        assert!(get_result.is_err());
    }

    /// Test listing alerts
    #[tokio::test]
    async fn test_list_alerts() {
        let manager = create_test_alert_manager();

        manager
            .create_alert(
                constants::TEST_BACKEND_ID,
                create_test_alert("list-1", "rps", 100.0),
            )
            .await
            .unwrap();
        manager
            .create_alert(
                constants::TEST_BACKEND_ID,
                create_test_alert("list-2", "latency", 200.0),
            )
            .await
            .unwrap();

        let result = manager.list_alerts(constants::TEST_BACKEND_ID, None).await;

        assert!(result.is_ok());
        let (alerts, pagination) = result.unwrap();
        assert!(alerts.len() >= 2);
    }
}

// ============================================================================
// Alert Condition Tests
// ============================================================================

#[cfg(test)]
mod condition_tests {
    use super::*;

    /// Test greater than operator
    #[test]
    fn test_condition_greater_than() {
        let manager = create_test_alert_manager();
        let condition = AlertCondition {
            metric: "rps".to_string(),
            operator: AlertOperator::GreaterThan as i32,
            threshold: 100.0,
            duration_seconds: 0,
        };

        assert!(manager.check_condition(150.0, &condition));
        assert!(!manager.check_condition(50.0, &condition));
        assert!(!manager.check_condition(100.0, &condition)); // Not greater
    }

    /// Test less than operator
    #[test]
    fn test_condition_less_than() {
        let manager = create_test_alert_manager();
        let condition = AlertCondition {
            metric: "uptime".to_string(),
            operator: AlertOperator::LessThan as i32,
            threshold: 99.0,
            duration_seconds: 0,
        };

        assert!(manager.check_condition(95.0, &condition));
        assert!(!manager.check_condition(99.5, &condition));
        assert!(!manager.check_condition(99.0, &condition)); // Not less
    }

    /// Test equal operator
    #[test]
    fn test_condition_equal() {
        let manager = create_test_alert_manager();
        let condition = AlertCondition {
            metric: "connections".to_string(),
            operator: AlertOperator::Equal as i32,
            threshold: 0.0,
            duration_seconds: 0,
        };

        assert!(manager.check_condition(0.0, &condition));
        assert!(!manager.check_condition(0.001, &condition));
    }

    /// Test not equal operator
    #[test]
    fn test_condition_not_equal() {
        let manager = create_test_alert_manager();
        let condition = AlertCondition {
            metric: "status".to_string(),
            operator: AlertOperator::NotEqual as i32,
            threshold: 1.0,
            duration_seconds: 0,
        };

        assert!(manager.check_condition(0.0, &condition));
        assert!(!manager.check_condition(1.0, &condition));
    }
}

// ============================================================================
// Alert Evaluation Tests
// ============================================================================

#[cfg(test)]
mod evaluation_tests {
    use super::*;

    /// Test evaluating alerts with metrics
    #[tokio::test]
    async fn test_evaluate_alerts() {
        let manager = create_test_alert_manager();

        // Create alert that should fire
        let alert = Alert {
            id: "eval-test".to_string(),
            backend_id: constants::TEST_BACKEND_ID.to_string(),
            name: "High RPS Alert".to_string(),
            condition: Some(AlertCondition {
                metric: "rps".to_string(),
                operator: AlertOperator::GreaterThan as i32,
                threshold: 100.0,
                duration_seconds: 0, // Immediate
            }),
            enabled: true,
            ..Default::default()
        };

        manager
            .create_alert(constants::TEST_BACKEND_ID, alert)
            .await
            .unwrap();

        // Evaluate with metrics that exceed threshold
        let mut metrics = HashMap::new();
        metrics.insert("rps".to_string(), 150.0);

        let result = manager
            .evaluate_alerts(constants::TEST_BACKEND_ID, &metrics)
            .await;

        assert!(result.is_ok());
    }

    /// Test alert with duration threshold
    #[tokio::test]
    async fn test_alert_duration_threshold() {
        let manager = create_test_alert_manager();

        let alert = Alert {
            id: "duration-test".to_string(),
            backend_id: constants::TEST_BACKEND_ID.to_string(),
            name: "Sustained High RPS".to_string(),
            condition: Some(AlertCondition {
                metric: "rps".to_string(),
                operator: AlertOperator::GreaterThan as i32,
                threshold: 100.0,
                duration_seconds: 60, // Must exceed for 60 seconds
            }),
            enabled: true,
            ..Default::default()
        };

        manager
            .create_alert(constants::TEST_BACKEND_ID, alert)
            .await
            .unwrap();

        // First evaluation - condition met but not for duration
        let mut metrics = HashMap::new();
        metrics.insert("rps".to_string(), 150.0);

        manager
            .evaluate_alerts(constants::TEST_BACKEND_ID, &metrics)
            .await
            .unwrap();

        // Alert should be pending, not firing
        let state = manager.get_alert_state("duration-test").await.unwrap();
        assert_eq!(state, AlertState::Pending);
    }

    /// Test disabled alerts are not evaluated
    #[tokio::test]
    async fn test_disabled_alert_not_evaluated() {
        let manager = create_test_alert_manager();

        let mut alert = create_test_alert("disabled-test", "rps", 100.0);
        alert.enabled = false;

        manager
            .create_alert(constants::TEST_BACKEND_ID, alert)
            .await
            .unwrap();

        let mut metrics = HashMap::new();
        metrics.insert("rps".to_string(), 150.0); // Exceeds threshold

        manager
            .evaluate_alerts(constants::TEST_BACKEND_ID, &metrics)
            .await
            .unwrap();

        // Alert should still be OK (not evaluated)
        let state = manager.get_alert_state("disabled-test").await.unwrap();
        assert_eq!(state, AlertState::Ok);
    }

    /// Test alert state transitions
    #[tokio::test]
    async fn test_alert_state_transitions() {
        let manager = create_test_alert_manager();

        let alert = Alert {
            id: "state-test".to_string(),
            backend_id: constants::TEST_BACKEND_ID.to_string(),
            name: "State Test Alert".to_string(),
            condition: Some(AlertCondition {
                metric: "rps".to_string(),
                operator: AlertOperator::GreaterThan as i32,
                threshold: 100.0,
                duration_seconds: 0,
            }),
            enabled: true,
            ..Default::default()
        };

        manager
            .create_alert(constants::TEST_BACKEND_ID, alert)
            .await
            .unwrap();

        // Initial state: OK
        let state = manager.get_alert_state("state-test").await.unwrap();
        assert_eq!(state, AlertState::Ok);

        // Exceed threshold -> Firing
        let mut metrics = HashMap::new();
        metrics.insert("rps".to_string(), 150.0);
        manager
            .evaluate_alerts(constants::TEST_BACKEND_ID, &metrics)
            .await
            .unwrap();

        let state = manager.get_alert_state("state-test").await.unwrap();
        assert_eq!(state, AlertState::Firing);

        // Below threshold -> OK
        metrics.insert("rps".to_string(), 50.0);
        manager
            .evaluate_alerts(constants::TEST_BACKEND_ID, &metrics)
            .await
            .unwrap();

        let state = manager.get_alert_state("state-test").await.unwrap();
        assert_eq!(state, AlertState::Ok);
    }
}

// ============================================================================
// Notification Tests
// ============================================================================

#[cfg(test)]
mod notification_tests {
    use super::*;

    /// Test alert with webhook notification
    #[tokio::test]
    async fn test_webhook_notification() {
        let manager = create_test_alert_manager();

        let alert = Alert {
            id: "webhook-test".to_string(),
            backend_id: constants::TEST_BACKEND_ID.to_string(),
            name: "Webhook Alert".to_string(),
            condition: Some(AlertCondition {
                metric: "rps".to_string(),
                operator: AlertOperator::GreaterThan as i32,
                threshold: 100.0,
                duration_seconds: 0,
            }),
            notifications: vec![AlertNotification {
                r#type: AlertNotificationType::Webhook as i32,
                destination: "https://example.com/webhook".to_string(),
            }],
            enabled: true,
            ..Default::default()
        };

        manager
            .create_alert(constants::TEST_BACKEND_ID, alert)
            .await
            .unwrap();

        // Trigger alert (notification would fail without real endpoint)
        let mut metrics = HashMap::new();
        metrics.insert("rps".to_string(), 150.0);

        // This will attempt to send notification
        let result = manager
            .evaluate_alerts(constants::TEST_BACKEND_ID, &metrics)
            .await;

        // Should succeed even if notification fails
        assert!(result.is_ok());
    }

    /// Test alert repeat interval
    #[tokio::test]
    async fn test_notification_repeat_interval() {
        let config = AlertConfig {
            eval_interval: StdDuration::from_secs(1),
            min_repeat_interval: StdDuration::from_secs(300), // 5 minutes
            notification_retries: 3,
            notification_timeout: StdDuration::from_secs(5),
        };
        let manager = AlertManager::new_with_memory_store(config);

        let alert = Alert {
            id: "repeat-test".to_string(),
            backend_id: constants::TEST_BACKEND_ID.to_string(),
            name: "Repeat Test Alert".to_string(),
            condition: Some(AlertCondition {
                metric: "rps".to_string(),
                operator: AlertOperator::GreaterThan as i32,
                threshold: 100.0,
                duration_seconds: 0,
            }),
            notifications: vec![AlertNotification {
                r#type: AlertNotificationType::Webhook as i32,
                destination: "https://example.com/webhook".to_string(),
            }],
            enabled: true,
            ..Default::default()
        };

        manager
            .create_alert(constants::TEST_BACKEND_ID, alert)
            .await
            .unwrap();

        let mut metrics = HashMap::new();
        metrics.insert("rps".to_string(), 150.0);

        // First evaluation triggers notification
        manager
            .evaluate_alerts(constants::TEST_BACKEND_ID, &metrics)
            .await
            .unwrap();

        let first_trigger = manager.get_last_triggered("repeat-test").await.unwrap();

        // Second evaluation within repeat interval should not re-trigger
        manager
            .evaluate_alerts(constants::TEST_BACKEND_ID, &metrics)
            .await
            .unwrap();

        let second_trigger = manager.get_last_triggered("repeat-test").await.unwrap();

        // Last triggered time should be the same (no repeat)
        assert_eq!(first_trigger, second_trigger);
    }
}

// ============================================================================
// Validation Tests
// ============================================================================

#[cfg(test)]
mod validation_tests {
    use super::*;

    /// Test alert validation - missing name
    #[tokio::test]
    async fn test_validation_missing_name() {
        let manager = create_test_alert_manager();

        let alert = Alert {
            id: "validation-1".to_string(),
            backend_id: constants::TEST_BACKEND_ID.to_string(),
            name: "".to_string(), // Empty name
            condition: Some(AlertCondition {
                metric: "rps".to_string(),
                operator: AlertOperator::GreaterThan as i32,
                threshold: 100.0,
                duration_seconds: 0,
            }),
            enabled: true,
            ..Default::default()
        };

        let result = manager
            .create_alert(constants::TEST_BACKEND_ID, alert)
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().to_lowercase().contains("name"));
    }

    /// Test alert validation - missing condition
    #[tokio::test]
    async fn test_validation_missing_condition() {
        let manager = create_test_alert_manager();

        let alert = Alert {
            id: "validation-2".to_string(),
            backend_id: constants::TEST_BACKEND_ID.to_string(),
            name: "Test Alert".to_string(),
            condition: None, // No condition
            enabled: true,
            ..Default::default()
        };

        let result = manager
            .create_alert(constants::TEST_BACKEND_ID, alert)
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().to_lowercase().contains("condition"));
    }

    /// Test alert validation - missing metric
    #[tokio::test]
    async fn test_validation_missing_metric() {
        let manager = create_test_alert_manager();

        let alert = Alert {
            id: "validation-3".to_string(),
            backend_id: constants::TEST_BACKEND_ID.to_string(),
            name: "Test Alert".to_string(),
            condition: Some(AlertCondition {
                metric: "".to_string(), // Empty metric
                operator: AlertOperator::GreaterThan as i32,
                threshold: 100.0,
                duration_seconds: 0,
            }),
            enabled: true,
            ..Default::default()
        };

        let result = manager
            .create_alert(constants::TEST_BACKEND_ID, alert)
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().to_lowercase().contains("metric"));
    }

    /// Test alert validation - invalid operator
    #[tokio::test]
    async fn test_validation_invalid_operator() {
        let manager = create_test_alert_manager();

        let alert = Alert {
            id: "validation-4".to_string(),
            backend_id: constants::TEST_BACKEND_ID.to_string(),
            name: "Test Alert".to_string(),
            condition: Some(AlertCondition {
                metric: "rps".to_string(),
                operator: AlertOperator::Unspecified as i32, // Invalid
                threshold: 100.0,
                duration_seconds: 0,
            }),
            enabled: true,
            ..Default::default()
        };

        let result = manager
            .create_alert(constants::TEST_BACKEND_ID, alert)
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().to_lowercase().contains("operator"));
    }
}
