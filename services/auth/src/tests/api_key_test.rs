//! API key management tests

use super::test_utils::{constants, generate_test_id, TestOrganization, TestUser};
use crate::services::api_key::{ApiKey, ApiKeyConfig, ApiKeyPermission, ApiKeyService};
use std::time::Duration;

/// Create a test API key service
fn create_test_api_key_service() -> ApiKeyService {
    let config = ApiKeyConfig {
        prefix: "pp_test".to_string(),
        hash_algorithm: "sha256".to_string(),
        max_keys_per_org: 10,
        default_expiry: Duration::from_secs(365 * 24 * 3600), // 1 year
    };
    ApiKeyService::new_with_memory_store(config)
}

// ============================================================================
// API Key Creation Tests
// ============================================================================

#[cfg(test)]
mod creation_tests {
    use super::*;

    /// Test creating an API key
    #[tokio::test]
    async fn test_create_api_key() {
        let service = create_test_api_key_service();

        let result = service
            .create_key(
                constants::TEST_ORG_ID,
                constants::TEST_USER_ID,
                "Production Key",
                vec![ApiKeyPermission::Read, ApiKeyPermission::Write],
                None,
            )
            .await;

        assert!(result.is_ok());
        let (key, secret) = result.unwrap();
        assert!(!key.id.is_empty());
        assert_eq!(key.name, "Production Key");
        assert!(secret.starts_with("pp_test_"));
    }

    /// Test API key has correct prefix
    #[tokio::test]
    async fn test_api_key_prefix() {
        let service = create_test_api_key_service();

        let (_, secret) = service
            .create_key(
                constants::TEST_ORG_ID,
                constants::TEST_USER_ID,
                "Test Key",
                vec![],
                None,
            )
            .await
            .unwrap();

        assert!(secret.starts_with("pp_test_"));
    }

    /// Test API key secret is only shown once
    #[tokio::test]
    async fn test_key_secret_shown_once() {
        let service = create_test_api_key_service();

        let (key, secret) = service
            .create_key(
                constants::TEST_ORG_ID,
                constants::TEST_USER_ID,
                "One Time Key",
                vec![],
                None,
            )
            .await
            .unwrap();

        // Get the key again - secret should not be available
        let retrieved = service.get_key(&key.id).await.unwrap();
        assert!(retrieved.secret_hash != secret);
    }

    /// Test creating key with expiry
    #[tokio::test]
    async fn test_create_key_with_expiry() {
        let service = create_test_api_key_service();

        let expires_at = chrono::Utc::now() + chrono::Duration::days(30);

        let (key, _) = service
            .create_key(
                constants::TEST_ORG_ID,
                constants::TEST_USER_ID,
                "Expiring Key",
                vec![],
                Some(expires_at),
            )
            .await
            .unwrap();

        assert!(key.expires_at.is_some());
    }

    /// Test max keys per org limit
    #[tokio::test]
    async fn test_max_keys_limit() {
        let config = ApiKeyConfig {
            prefix: "pp_test".to_string(),
            hash_algorithm: "sha256".to_string(),
            max_keys_per_org: 2,
            default_expiry: Duration::from_secs(3600),
        };
        let service = ApiKeyService::new_with_memory_store(config);

        // Create max keys
        service
            .create_key(constants::TEST_ORG_ID, constants::TEST_USER_ID, "Key 1", vec![], None)
            .await
            .unwrap();
        service
            .create_key(constants::TEST_ORG_ID, constants::TEST_USER_ID, "Key 2", vec![], None)
            .await
            .unwrap();

        // Third key should fail
        let result = service
            .create_key(constants::TEST_ORG_ID, constants::TEST_USER_ID, "Key 3", vec![], None)
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().to_lowercase().contains("limit")
            || err.to_string().to_lowercase().contains("maximum"));
    }

    /// Test creating key with all permissions
    #[tokio::test]
    async fn test_create_key_all_permissions() {
        let service = create_test_api_key_service();

        let permissions = vec![
            ApiKeyPermission::Read,
            ApiKeyPermission::Write,
            ApiKeyPermission::Delete,
            ApiKeyPermission::Admin,
        ];

        let (key, _) = service
            .create_key(
                constants::TEST_ORG_ID,
                constants::TEST_USER_ID,
                "Admin Key",
                permissions.clone(),
                None,
            )
            .await
            .unwrap();

        assert_eq!(key.permissions.len(), permissions.len());
    }
}

// ============================================================================
// API Key Validation Tests
// ============================================================================

#[cfg(test)]
mod validation_tests {
    use super::*;

    /// Test validating a valid API key
    #[tokio::test]
    async fn test_validate_valid_key() {
        let service = create_test_api_key_service();

        let (key, secret) = service
            .create_key(
                constants::TEST_ORG_ID,
                constants::TEST_USER_ID,
                "Valid Key",
                vec![ApiKeyPermission::Read],
                None,
            )
            .await
            .unwrap();

        let result = service.validate_key(&secret).await;

        assert!(result.is_ok());
        let validated = result.unwrap();
        assert_eq!(validated.id, key.id);
    }

    /// Test validating an invalid key
    #[tokio::test]
    async fn test_validate_invalid_key() {
        let service = create_test_api_key_service();

        let result = service.validate_key("pp_test_invalid_key_xyz").await;

        assert!(result.is_err());
    }

    /// Test validating updates last_used
    #[tokio::test]
    async fn test_validate_updates_last_used() {
        let service = create_test_api_key_service();

        let (key, secret) = service
            .create_key(
                constants::TEST_ORG_ID,
                constants::TEST_USER_ID,
                "Track Key",
                vec![],
                None,
            )
            .await
            .unwrap();

        // Initially, last_used should be None
        let initial = service.get_key(&key.id).await.unwrap();
        assert!(initial.last_used.is_none());

        // Validate the key
        service.validate_key(&secret).await.unwrap();

        // last_used should be updated
        let updated = service.get_key(&key.id).await.unwrap();
        assert!(updated.last_used.is_some());
    }

    /// Test validating expired key fails
    #[tokio::test]
    async fn test_validate_expired_key() {
        let service = create_test_api_key_service();

        let expires_at = chrono::Utc::now() - chrono::Duration::days(1);

        let (_, secret) = service
            .create_key(
                constants::TEST_ORG_ID,
                constants::TEST_USER_ID,
                "Expired Key",
                vec![],
                Some(expires_at),
            )
            .await
            .unwrap();

        let result = service.validate_key(&secret).await;

        assert!(result.is_err());
    }

    /// Test validating revoked key fails
    #[tokio::test]
    async fn test_validate_revoked_key() {
        let service = create_test_api_key_service();

        let (key, secret) = service
            .create_key(
                constants::TEST_ORG_ID,
                constants::TEST_USER_ID,
                "Revoked Key",
                vec![],
                None,
            )
            .await
            .unwrap();

        // Revoke the key
        service.revoke_key(&key.id).await.unwrap();

        let result = service.validate_key(&secret).await;

        assert!(result.is_err());
    }
}

// ============================================================================
// API Key Permission Tests
// ============================================================================

#[cfg(test)]
mod permission_tests {
    use super::*;

    /// Test checking key has permission
    #[tokio::test]
    async fn test_key_has_permission() {
        let service = create_test_api_key_service();

        let (key, secret) = service
            .create_key(
                constants::TEST_ORG_ID,
                constants::TEST_USER_ID,
                "Permission Key",
                vec![ApiKeyPermission::Read, ApiKeyPermission::Write],
                None,
            )
            .await
            .unwrap();

        let validated = service.validate_key(&secret).await.unwrap();

        assert!(validated.has_permission(ApiKeyPermission::Read));
        assert!(validated.has_permission(ApiKeyPermission::Write));
        assert!(!validated.has_permission(ApiKeyPermission::Delete));
        assert!(!validated.has_permission(ApiKeyPermission::Admin));
    }

    /// Test admin permission grants all access
    #[tokio::test]
    async fn test_admin_permission() {
        let service = create_test_api_key_service();

        let (_, secret) = service
            .create_key(
                constants::TEST_ORG_ID,
                constants::TEST_USER_ID,
                "Admin Key",
                vec![ApiKeyPermission::Admin],
                None,
            )
            .await
            .unwrap();

        let validated = service.validate_key(&secret).await.unwrap();

        // Admin should have all permissions
        assert!(validated.has_permission(ApiKeyPermission::Read));
        assert!(validated.has_permission(ApiKeyPermission::Write));
        assert!(validated.has_permission(ApiKeyPermission::Delete));
        assert!(validated.has_permission(ApiKeyPermission::Admin));
    }

    /// Test key without permissions
    #[tokio::test]
    async fn test_key_no_permissions() {
        let service = create_test_api_key_service();

        let (_, secret) = service
            .create_key(
                constants::TEST_ORG_ID,
                constants::TEST_USER_ID,
                "No Perm Key",
                vec![],
                None,
            )
            .await
            .unwrap();

        let validated = service.validate_key(&secret).await.unwrap();

        assert!(!validated.has_permission(ApiKeyPermission::Read));
        assert!(!validated.has_permission(ApiKeyPermission::Write));
    }
}

// ============================================================================
// API Key Management Tests
// ============================================================================

#[cfg(test)]
mod management_tests {
    use super::*;

    /// Test listing API keys for organization
    #[tokio::test]
    async fn test_list_keys() {
        let service = create_test_api_key_service();

        service
            .create_key(constants::TEST_ORG_ID, constants::TEST_USER_ID, "Key 1", vec![], None)
            .await
            .unwrap();
        service
            .create_key(constants::TEST_ORG_ID, constants::TEST_USER_ID, "Key 2", vec![], None)
            .await
            .unwrap();

        let result = service.list_keys(constants::TEST_ORG_ID).await;

        assert!(result.is_ok());
        let keys = result.unwrap();
        assert!(keys.len() >= 2);
    }

    /// Test getting a specific key
    #[tokio::test]
    async fn test_get_key() {
        let service = create_test_api_key_service();

        let (created, _) = service
            .create_key(constants::TEST_ORG_ID, constants::TEST_USER_ID, "Get Key", vec![], None)
            .await
            .unwrap();

        let result = service.get_key(&created.id).await;

        assert!(result.is_ok());
        let key = result.unwrap();
        assert_eq!(key.id, created.id);
        assert_eq!(key.name, "Get Key");
    }

    /// Test revoking a key
    #[tokio::test]
    async fn test_revoke_key() {
        let service = create_test_api_key_service();

        let (key, _) = service
            .create_key(
                constants::TEST_ORG_ID,
                constants::TEST_USER_ID,
                "Revoke Key",
                vec![],
                None,
            )
            .await
            .unwrap();

        let result = service.revoke_key(&key.id).await;

        assert!(result.is_ok());

        // Key should be revoked
        let revoked = service.get_key(&key.id).await.unwrap();
        assert!(revoked.revoked);
    }

    /// Test deleting a key
    #[tokio::test]
    async fn test_delete_key() {
        let service = create_test_api_key_service();

        let (key, _) = service
            .create_key(
                constants::TEST_ORG_ID,
                constants::TEST_USER_ID,
                "Delete Key",
                vec![],
                None,
            )
            .await
            .unwrap();

        let result = service.delete_key(&key.id).await;

        assert!(result.is_ok());

        // Key should not be retrievable
        let get_result = service.get_key(&key.id).await;
        assert!(get_result.is_err());
    }

    /// Test updating key name
    #[tokio::test]
    async fn test_update_key_name() {
        let service = create_test_api_key_service();

        let (key, _) = service
            .create_key(
                constants::TEST_ORG_ID,
                constants::TEST_USER_ID,
                "Original Name",
                vec![],
                None,
            )
            .await
            .unwrap();

        let result = service.update_key(&key.id, Some("Updated Name"), None).await;

        assert!(result.is_ok());
        let updated = result.unwrap();
        assert_eq!(updated.name, "Updated Name");
    }

    /// Test updating key permissions
    #[tokio::test]
    async fn test_update_key_permissions() {
        let service = create_test_api_key_service();

        let (key, _) = service
            .create_key(
                constants::TEST_ORG_ID,
                constants::TEST_USER_ID,
                "Perm Key",
                vec![ApiKeyPermission::Read],
                None,
            )
            .await
            .unwrap();

        let result = service
            .update_key(
                &key.id,
                None,
                Some(vec![ApiKeyPermission::Read, ApiKeyPermission::Write]),
            )
            .await;

        assert!(result.is_ok());
        let updated = result.unwrap();
        assert!(updated.permissions.contains(&ApiKeyPermission::Write));
    }
}

// ============================================================================
// API Key Security Tests
// ============================================================================

#[cfg(test)]
mod security_tests {
    use super::*;

    /// Test key secret is properly hashed
    #[tokio::test]
    async fn test_secret_is_hashed() {
        let service = create_test_api_key_service();

        let (key, secret) = service
            .create_key(
                constants::TEST_ORG_ID,
                constants::TEST_USER_ID,
                "Hash Key",
                vec![],
                None,
            )
            .await
            .unwrap();

        // The stored hash should not equal the secret
        assert_ne!(key.secret_hash, secret);
        // Hash should be a hex string
        assert!(key.secret_hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    /// Test key prefix is stored for display
    #[tokio::test]
    async fn test_key_prefix_stored() {
        let service = create_test_api_key_service();

        let (key, secret) = service
            .create_key(
                constants::TEST_ORG_ID,
                constants::TEST_USER_ID,
                "Prefix Key",
                vec![],
                None,
            )
            .await
            .unwrap();

        // Key should have prefix stored for UI display
        assert!(key.prefix.starts_with("pp_test_"));
        assert!(secret.starts_with(&key.prefix));
    }

    /// Test cannot validate with prefix only
    #[tokio::test]
    async fn test_cannot_validate_prefix() {
        let service = create_test_api_key_service();

        let (key, _) = service
            .create_key(
                constants::TEST_ORG_ID,
                constants::TEST_USER_ID,
                "Security Key",
                vec![],
                None,
            )
            .await
            .unwrap();

        // Try to validate with just the prefix
        let result = service.validate_key(&key.prefix).await;

        assert!(result.is_err());
    }

    /// Test key rotation
    #[tokio::test]
    async fn test_key_rotation() {
        let service = create_test_api_key_service();

        let (key, old_secret) = service
            .create_key(
                constants::TEST_ORG_ID,
                constants::TEST_USER_ID,
                "Rotate Key",
                vec![ApiKeyPermission::Read],
                None,
            )
            .await
            .unwrap();

        // Rotate the key
        let result = service.rotate_key(&key.id).await;

        assert!(result.is_ok());
        let new_secret = result.unwrap();
        assert_ne!(new_secret, old_secret);

        // Old secret should no longer work
        let old_result = service.validate_key(&old_secret).await;
        assert!(old_result.is_err());

        // New secret should work
        let new_result = service.validate_key(&new_secret).await;
        assert!(new_result.is_ok());
    }
}
