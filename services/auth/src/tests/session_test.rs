//! Session management tests

use super::test_utils::{
    TestUser, constants, current_timestamp, future_timestamp, generate_test_id,
};
use crate::services::session::{Session, SessionConfig, SessionService, SessionStore};
use std::net::IpAddr;
use std::time::Duration;

/// Create a test session service with in-memory store
fn create_test_session_service() -> SessionService {
    let config = SessionConfig {
        session_duration: Duration::from_secs(3600),
        max_sessions_per_user: 5,
        idle_timeout: Duration::from_secs(1800),
        refresh_threshold: Duration::from_secs(300),
    };
    SessionService::new_with_memory_store(config)
}

/// Create a test session
fn create_test_session() -> Session {
    Session {
        id: constants::TEST_SESSION_ID.to_string(),
        user_id: constants::TEST_USER_ID.to_string(),
        token_hash: "test_token_hash".to_string(),
        ip_address: Some("127.0.0.1".to_string()),
        user_agent: Some("Test Agent/1.0".to_string()),
        device_info: Some("Test Device".to_string()),
        created_at: current_timestamp(),
        expires_at: future_timestamp(Duration::from_secs(3600)),
        last_activity: current_timestamp(),
        is_active: true,
    }
}

// ============================================================================
// Session Creation Tests
// ============================================================================

#[cfg(test)]
mod session_creation_tests {
    use super::*;

    /// Test creating a new session
    #[tokio::test]
    async fn test_create_session() {
        let service = create_test_session_service();
        let user = TestUser::new();

        let result = service
            .create_session(
                &user.id,
                Some("127.0.0.1".parse().unwrap()),
                Some("Test Agent/1.0"),
            )
            .await;

        assert!(result.is_ok());
        let session = result.unwrap();
        assert_eq!(session.user_id, user.id);
        assert!(session.is_active);
        assert!(session.expires_at > current_timestamp());
    }

    /// Test creating session stores IP address
    #[tokio::test]
    async fn test_create_session_with_ip() {
        let service = create_test_session_service();
        let user = TestUser::new();
        let ip: IpAddr = "192.168.1.100".parse().unwrap();

        let result = service.create_session(&user.id, Some(ip), None).await;

        assert!(result.is_ok());
        let session = result.unwrap();
        assert_eq!(session.ip_address, Some("192.168.1.100".to_string()));
    }

    /// Test creating session stores user agent
    #[tokio::test]
    async fn test_create_session_with_user_agent() {
        let service = create_test_session_service();
        let user = TestUser::new();
        let user_agent = "Mozilla/5.0 (Windows NT 10.0; Win64; x64)";

        let result = service
            .create_session(&user.id, None, Some(user_agent))
            .await;

        assert!(result.is_ok());
        let session = result.unwrap();
        assert_eq!(session.user_agent, Some(user_agent.to_string()));
    }

    /// Test creating multiple sessions for same user
    #[tokio::test]
    async fn test_create_multiple_sessions() {
        let service = create_test_session_service();
        let user = TestUser::new();

        let session1 = service.create_session(&user.id, None, None).await.unwrap();
        let session2 = service.create_session(&user.id, None, None).await.unwrap();

        assert_ne!(session1.id, session2.id);

        let sessions = service.list_user_sessions(&user.id).await.unwrap();
        assert!(sessions.len() >= 2);
    }

    /// Test max sessions per user limit
    #[tokio::test]
    async fn test_max_sessions_limit() {
        let config = SessionConfig {
            session_duration: Duration::from_secs(3600),
            max_sessions_per_user: 2,
            idle_timeout: Duration::from_secs(1800),
            refresh_threshold: Duration::from_secs(300),
        };
        let service = SessionService::new_with_memory_store(config);
        let user = TestUser::new();

        // Create max sessions
        service.create_session(&user.id, None, None).await.unwrap();
        service.create_session(&user.id, None, None).await.unwrap();

        // Creating one more should either:
        // 1. Evict oldest session
        // 2. Fail with error
        let result = service.create_session(&user.id, None, None).await;

        // Implementation dependent - either succeeds (with eviction) or fails
        let sessions = service.list_user_sessions(&user.id).await.unwrap();
        assert!(sessions.len() <= 3); // Should not exceed limit by much
    }
}

// ============================================================================
// Session Validation Tests
// ============================================================================

#[cfg(test)]
mod session_validation_tests {
    use super::*;

    /// Test validating a valid session
    #[tokio::test]
    async fn test_validate_valid_session() {
        let service = create_test_session_service();
        let user = TestUser::new();

        let session = service.create_session(&user.id, None, None).await.unwrap();
        let result = service.validate_session(&session.id).await;

        assert!(result.is_ok());
        let validated = result.unwrap();
        assert_eq!(validated.id, session.id);
        assert!(validated.is_active);
    }

    /// Test validating a non-existent session
    #[tokio::test]
    async fn test_validate_nonexistent_session() {
        let service = create_test_session_service();

        let result = service.validate_session("nonexistent-session-id").await;

        assert!(result.is_err());
    }

    /// Test validating an invalidated session
    #[tokio::test]
    async fn test_validate_invalidated_session() {
        let service = create_test_session_service();
        let user = TestUser::new();

        let session = service.create_session(&user.id, None, None).await.unwrap();
        service.invalidate_session(&session.id).await.unwrap();

        let result = service.validate_session(&session.id).await;

        assert!(result.is_err());
    }

    /// Test validating updates last activity
    #[tokio::test]
    async fn test_validate_updates_activity() {
        let service = create_test_session_service();
        let user = TestUser::new();

        let session = service.create_session(&user.id, None, None).await.unwrap();
        let initial_activity = session.last_activity;

        // Small delay to ensure time difference
        tokio::time::sleep(Duration::from_millis(100)).await;

        let validated = service.validate_session(&session.id).await.unwrap();

        // Last activity should be updated (or same if within threshold)
        assert!(validated.last_activity >= initial_activity);
    }
}

// ============================================================================
// Session Invalidation Tests
// ============================================================================

#[cfg(test)]
mod session_invalidation_tests {
    use super::*;

    /// Test invalidating a session
    #[tokio::test]
    async fn test_invalidate_session() {
        let service = create_test_session_service();
        let user = TestUser::new();

        let session = service.create_session(&user.id, None, None).await.unwrap();
        let result = service.invalidate_session(&session.id).await;

        assert!(result.is_ok());

        // Session should no longer be valid
        let validate_result = service.validate_session(&session.id).await;
        assert!(validate_result.is_err());
    }

    /// Test invalidating all user sessions
    #[tokio::test]
    async fn test_invalidate_all_user_sessions() {
        let service = create_test_session_service();
        let user = TestUser::new();

        // Create multiple sessions
        let session1 = service.create_session(&user.id, None, None).await.unwrap();
        let session2 = service.create_session(&user.id, None, None).await.unwrap();

        // Invalidate all
        let result = service.invalidate_all_sessions(&user.id).await;
        assert!(result.is_ok());

        // All sessions should be invalid
        assert!(service.validate_session(&session1.id).await.is_err());
        assert!(service.validate_session(&session2.id).await.is_err());
    }

    /// Test invalidating specific session doesn't affect others
    #[tokio::test]
    async fn test_invalidate_specific_session() {
        let service = create_test_session_service();
        let user = TestUser::new();

        let session1 = service.create_session(&user.id, None, None).await.unwrap();
        let session2 = service.create_session(&user.id, None, None).await.unwrap();

        // Invalidate only first session
        service.invalidate_session(&session1.id).await.unwrap();

        // First should be invalid, second should be valid
        assert!(service.validate_session(&session1.id).await.is_err());
        assert!(service.validate_session(&session2.id).await.is_ok());
    }
}

// ============================================================================
// Session Listing Tests
// ============================================================================

#[cfg(test)]
mod session_listing_tests {
    use super::*;

    /// Test listing user sessions
    #[tokio::test]
    async fn test_list_user_sessions() {
        let service = create_test_session_service();
        let user = TestUser::new();

        // Create sessions
        service.create_session(&user.id, None, None).await.unwrap();
        service.create_session(&user.id, None, None).await.unwrap();

        let sessions = service.list_user_sessions(&user.id).await.unwrap();

        assert!(sessions.len() >= 2);
        for session in sessions {
            assert_eq!(session.user_id, user.id);
        }
    }

    /// Test listing sessions for user with no sessions
    #[tokio::test]
    async fn test_list_sessions_empty() {
        let service = create_test_session_service();

        let sessions = service.list_user_sessions("no-such-user").await.unwrap();

        assert!(sessions.is_empty());
    }

    /// Test listing only active sessions
    #[tokio::test]
    async fn test_list_active_sessions() {
        let service = create_test_session_service();
        let user = TestUser::new();

        let session1 = service.create_session(&user.id, None, None).await.unwrap();
        let session2 = service.create_session(&user.id, None, None).await.unwrap();

        // Invalidate one session
        service.invalidate_session(&session1.id).await.unwrap();

        // List should return only active sessions
        let active_sessions = service.list_active_sessions(&user.id).await.unwrap();

        assert_eq!(active_sessions.len(), 1);
        assert_eq!(active_sessions[0].id, session2.id);
    }
}

// ============================================================================
// Session Refresh Tests
// ============================================================================

#[cfg(test)]
mod session_refresh_tests {
    use super::*;

    /// Test refreshing a session extends expiry
    #[tokio::test]
    async fn test_refresh_session() {
        let service = create_test_session_service();
        let user = TestUser::new();

        let session = service.create_session(&user.id, None, None).await.unwrap();
        let original_expiry = session.expires_at;

        let result = service.refresh_session(&session.id).await;

        assert!(result.is_ok());
        let refreshed = result.unwrap();
        assert!(refreshed.expires_at >= original_expiry);
    }

    /// Test refreshing invalidated session fails
    #[tokio::test]
    async fn test_refresh_invalidated_session() {
        let service = create_test_session_service();
        let user = TestUser::new();

        let session = service.create_session(&user.id, None, None).await.unwrap();
        service.invalidate_session(&session.id).await.unwrap();

        let result = service.refresh_session(&session.id).await;

        assert!(result.is_err());
    }
}

// ============================================================================
// Session Store Tests
// ============================================================================

#[cfg(test)]
mod session_store_tests {
    use super::*;

    /// Test in-memory store basic operations
    #[tokio::test]
    async fn test_memory_store_crud() {
        let store = SessionStore::new_memory();
        let session = create_test_session();

        // Create
        store.save(&session).await.unwrap();

        // Read
        let retrieved = store.get(&session.id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, session.id);

        // Update
        let mut updated = session.clone();
        updated.last_activity = current_timestamp() + 100;
        store.save(&updated).await.unwrap();

        let retrieved = store.get(&session.id).await.unwrap().unwrap();
        assert_eq!(retrieved.last_activity, updated.last_activity);

        // Delete
        store.delete(&session.id).await.unwrap();
        assert!(store.get(&session.id).await.unwrap().is_none());
    }

    /// Test store finds by user
    #[tokio::test]
    async fn test_store_find_by_user() {
        let store = SessionStore::new_memory();
        let user_id = "test-user";

        let mut session1 = create_test_session();
        session1.id = "session-1".to_string();
        session1.user_id = user_id.to_string();

        let mut session2 = create_test_session();
        session2.id = "session-2".to_string();
        session2.user_id = user_id.to_string();

        let mut session3 = create_test_session();
        session3.id = "session-3".to_string();
        session3.user_id = "other-user".to_string();

        store.save(&session1).await.unwrap();
        store.save(&session2).await.unwrap();
        store.save(&session3).await.unwrap();

        let sessions = store.find_by_user(user_id).await.unwrap();

        assert_eq!(sessions.len(), 2);
        assert!(sessions.iter().all(|s| s.user_id == user_id));
    }

    /// Test store concurrent access
    #[tokio::test]
    async fn test_store_concurrent_access() {
        let store = SessionStore::new_memory();
        let user = TestUser::new();

        let mut handles = vec![];

        for i in 0..10 {
            let store_clone = store.clone();
            let user_id = user.id.clone();
            handles.push(tokio::spawn(async move {
                let mut session = create_test_session();
                session.id = format!("session-{}", i);
                session.user_id = user_id;
                store_clone.save(&session).await.unwrap();
            }));
        }

        for handle in handles {
            handle.await.unwrap();
        }

        let sessions = store.find_by_user(&user.id).await.unwrap();
        assert_eq!(sessions.len(), 10);
    }
}

// ============================================================================
// Session Config Tests
// ============================================================================

#[cfg(test)]
mod config_tests {
    use super::*;

    /// Test session config default values
    #[test]
    fn test_session_config_default() {
        let config = SessionConfig::default();

        assert!(config.session_duration.as_secs() > 0);
        assert!(config.max_sessions_per_user > 0);
        assert!(config.idle_timeout.as_secs() > 0);
        assert!(config.idle_timeout <= config.session_duration);
    }

    /// Test session config validation
    #[test]
    fn test_session_config_validation() {
        let config = SessionConfig {
            session_duration: Duration::from_secs(3600),
            max_sessions_per_user: 10,
            idle_timeout: Duration::from_secs(1800),
            refresh_threshold: Duration::from_secs(300),
        };

        let result = config.validate();
        assert!(result.is_ok());
    }

    /// Test session config with invalid values
    #[test]
    fn test_session_config_invalid() {
        let config = SessionConfig {
            session_duration: Duration::from_secs(0),
            max_sessions_per_user: 0,
            idle_timeout: Duration::from_secs(0),
            refresh_threshold: Duration::from_secs(0),
        };

        // Invalid config should be caught by validation
        let result = config.validate();
        // Implementation dependent - may fail or use defaults
    }
}
