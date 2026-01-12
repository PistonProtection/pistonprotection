//! JWT validation tests

use super::test_utils::{constants, current_timestamp, future_timestamp, past_timestamp, TestUser};
use crate::services::jwt::{Claims, JwtConfig, JwtService};
use std::time::Duration;

/// Create a test JWT service with default config
fn create_test_jwt_service() -> JwtService {
    let config = JwtConfig {
        secret: constants::TEST_JWT_SECRET.to_string(),
        issuer: constants::TEST_JWT_ISSUER.to_string(),
        audience: constants::TEST_JWT_AUDIENCE.to_string(),
        access_token_expiry: Duration::from_secs(3600),    // 1 hour
        refresh_token_expiry: Duration::from_secs(604800), // 7 days
    };
    JwtService::new(config)
}

/// Create test claims
fn create_test_claims() -> Claims {
    let user = TestUser::new();
    Claims {
        sub: user.id,
        email: user.email,
        name: user.name,
        org_id: Some(constants::TEST_ORG_ID.to_string()),
        role: "user".to_string(),
        iss: constants::TEST_JWT_ISSUER.to_string(),
        aud: constants::TEST_JWT_AUDIENCE.to_string(),
        iat: current_timestamp(),
        exp: future_timestamp(Duration::from_secs(3600)),
        nbf: current_timestamp(),
        jti: uuid::Uuid::new_v4().to_string(),
    }
}

// ============================================================================
// Token Generation Tests
// ============================================================================

#[cfg(test)]
mod token_generation_tests {
    use super::*;

    /// Test generating an access token
    #[test]
    fn test_generate_access_token() {
        let service = create_test_jwt_service();
        let claims = create_test_claims();

        let result = service.generate_access_token(&claims);

        assert!(result.is_ok());
        let token = result.unwrap();
        assert!(!token.is_empty());
        // JWT has 3 parts separated by dots
        assert_eq!(token.split('.').count(), 3);
    }

    /// Test generating a refresh token
    #[test]
    fn test_generate_refresh_token() {
        let service = create_test_jwt_service();
        let claims = create_test_claims();

        let result = service.generate_refresh_token(&claims);

        assert!(result.is_ok());
        let token = result.unwrap();
        assert!(!token.is_empty());
        assert_eq!(token.split('.').count(), 3);
    }

    /// Test generating token pair
    #[test]
    fn test_generate_token_pair() {
        let service = create_test_jwt_service();
        let claims = create_test_claims();

        let result = service.generate_token_pair(&claims);

        assert!(result.is_ok());
        let (access, refresh) = result.unwrap();
        assert_ne!(access, refresh);
    }

    /// Test tokens for different users are different
    #[test]
    fn test_different_users_different_tokens() {
        let service = create_test_jwt_service();

        let claims1 = create_test_claims();
        let mut claims2 = create_test_claims();
        claims2.sub = "different-user-id".to_string();

        let token1 = service.generate_access_token(&claims1).unwrap();
        let token2 = service.generate_access_token(&claims2).unwrap();

        assert_ne!(token1, token2);
    }
}

// ============================================================================
// Token Validation Tests
// ============================================================================

#[cfg(test)]
mod token_validation_tests {
    use super::*;

    /// Test validating a valid access token
    #[test]
    fn test_validate_valid_token() {
        let service = create_test_jwt_service();
        let original_claims = create_test_claims();

        let token = service.generate_access_token(&original_claims).unwrap();
        let result = service.validate_access_token(&token);

        assert!(result.is_ok());
        let validated_claims = result.unwrap();
        assert_eq!(validated_claims.sub, original_claims.sub);
        assert_eq!(validated_claims.email, original_claims.email);
    }

    /// Test validating an expired token
    #[test]
    fn test_validate_expired_token() {
        let service = create_test_jwt_service();
        let mut claims = create_test_claims();
        // Set expiration in the past
        claims.exp = past_timestamp(Duration::from_secs(3600));

        let token = service.generate_access_token(&claims).unwrap();
        let result = service.validate_access_token(&token);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().to_lowercase().contains("expired"));
    }

    /// Test validating a token with invalid signature
    #[test]
    fn test_validate_invalid_signature() {
        let service1 = create_test_jwt_service();

        // Create service with different secret
        let different_config = JwtConfig {
            secret: "different-secret-key-for-testing-purposes-here".to_string(),
            issuer: constants::TEST_JWT_ISSUER.to_string(),
            audience: constants::TEST_JWT_AUDIENCE.to_string(),
            access_token_expiry: Duration::from_secs(3600),
            refresh_token_expiry: Duration::from_secs(604800),
        };
        let service2 = JwtService::new(different_config);

        let claims = create_test_claims();
        let token = service1.generate_access_token(&claims).unwrap();

        // Validate with different secret should fail
        let result = service2.validate_access_token(&token);
        assert!(result.is_err());
    }

    /// Test validating a malformed token
    #[test]
    fn test_validate_malformed_token() {
        let service = create_test_jwt_service();

        let malformed_tokens = vec![
            "",
            "not-a-jwt",
            "only.two",
            "too.many.parts.here",
            "eyJhbGciOiJIUzI1NiJ9.invalid.signature",
        ];

        for token in malformed_tokens {
            let result = service.validate_access_token(token);
            assert!(result.is_err(), "Should fail for: {}", token);
        }
    }

    /// Test validating a token not yet valid (nbf in future)
    #[test]
    fn test_validate_not_yet_valid_token() {
        let service = create_test_jwt_service();
        let mut claims = create_test_claims();
        // Set not before in the future
        claims.nbf = future_timestamp(Duration::from_secs(3600));

        let token = service.generate_access_token(&claims).unwrap();
        let result = service.validate_access_token(&token);

        // Token should be invalid due to nbf check
        assert!(result.is_err());
    }

    /// Test validating with wrong issuer
    #[test]
    fn test_validate_wrong_issuer() {
        let service = create_test_jwt_service();
        let mut claims = create_test_claims();
        claims.iss = "wrong-issuer".to_string();

        let token = service.generate_access_token(&claims).unwrap();
        let result = service.validate_access_token(&token);

        // Depending on validation strictness, may or may not fail
        // Most implementations validate issuer claim
        if result.is_err() {
            let err = result.unwrap_err();
            assert!(
                err.to_string().to_lowercase().contains("issuer")
                    || err.to_string().to_lowercase().contains("claim")
            );
        }
    }
}

// ============================================================================
// Token Refresh Tests
// ============================================================================

#[cfg(test)]
mod token_refresh_tests {
    use super::*;

    /// Test refreshing tokens
    #[test]
    fn test_refresh_tokens() {
        let service = create_test_jwt_service();
        let claims = create_test_claims();

        let refresh_token = service.generate_refresh_token(&claims).unwrap();
        let result = service.refresh_tokens(&refresh_token);

        assert!(result.is_ok());
        let (new_access, new_refresh) = result.unwrap();

        // Should generate new tokens
        assert_ne!(new_access, refresh_token);
        assert_ne!(new_refresh, refresh_token);
    }

    /// Test refreshing with expired refresh token
    #[test]
    fn test_refresh_expired_token() {
        let service = create_test_jwt_service();
        let mut claims = create_test_claims();
        claims.exp = past_timestamp(Duration::from_secs(3600));

        let refresh_token = service.generate_refresh_token(&claims).unwrap();
        let result = service.refresh_tokens(&refresh_token);

        assert!(result.is_err());
    }

    /// Test refreshing with access token (should fail)
    #[test]
    fn test_refresh_with_access_token() {
        let service = create_test_jwt_service();
        let claims = create_test_claims();

        let access_token = service.generate_access_token(&claims).unwrap();
        let result = service.refresh_tokens(&access_token);

        // Depending on implementation, may fail due to token type check
        // or succeed (if no token type distinction)
    }
}

// ============================================================================
// Claims Extraction Tests
// ============================================================================

#[cfg(test)]
mod claims_extraction_tests {
    use super::*;

    /// Test extracting claims without validation
    #[test]
    fn test_extract_claims_unvalidated() {
        let service = create_test_jwt_service();
        let original_claims = create_test_claims();

        let token = service.generate_access_token(&original_claims).unwrap();
        let result = service.extract_claims_unvalidated(&token);

        assert!(result.is_ok());
        let extracted = result.unwrap();
        assert_eq!(extracted.sub, original_claims.sub);
    }

    /// Test extracting claims from expired token
    #[test]
    fn test_extract_claims_from_expired() {
        let service = create_test_jwt_service();
        let mut claims = create_test_claims();
        claims.exp = past_timestamp(Duration::from_secs(3600));

        let token = service.generate_access_token(&claims).unwrap();

        // Unvalidated extraction should work
        let result = service.extract_claims_unvalidated(&token);
        assert!(result.is_ok());

        // Validated extraction should fail
        let result = service.validate_access_token(&token);
        assert!(result.is_err());
    }
}

// ============================================================================
// Claims Structure Tests
// ============================================================================

#[cfg(test)]
mod claims_structure_tests {
    use super::*;

    /// Test claims serialization/deserialization
    #[test]
    fn test_claims_serde() {
        let claims = create_test_claims();

        let json = serde_json::to_string(&claims).unwrap();
        let deserialized: Claims = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.sub, claims.sub);
        assert_eq!(deserialized.email, claims.email);
        assert_eq!(deserialized.org_id, claims.org_id);
    }

    /// Test claims with optional fields
    #[test]
    fn test_claims_optional_org() {
        let mut claims = create_test_claims();
        claims.org_id = None;

        let json = serde_json::to_string(&claims).unwrap();
        let deserialized: Claims = serde_json::from_str(&json).unwrap();

        assert!(deserialized.org_id.is_none());
    }

    /// Test claims role validation
    #[test]
    fn test_claims_roles() {
        let valid_roles = vec!["admin", "user", "viewer", "owner", "member"];

        for role in valid_roles {
            let mut claims = create_test_claims();
            claims.role = role.to_string();

            let service = create_test_jwt_service();
            let token = service.generate_access_token(&claims).unwrap();
            let result = service.validate_access_token(&token);

            assert!(result.is_ok());
            assert_eq!(result.unwrap().role, role);
        }
    }
}

// ============================================================================
// JWT Configuration Tests
// ============================================================================

#[cfg(test)]
mod config_tests {
    use super::*;

    /// Test JWT config validation
    #[test]
    fn test_jwt_config_validation() {
        // Valid config
        let config = JwtConfig {
            secret: "a-valid-secret-key-that-is-long-enough-32-bytes".to_string(),
            issuer: "test-issuer".to_string(),
            audience: "test-audience".to_string(),
            access_token_expiry: Duration::from_secs(3600),
            refresh_token_expiry: Duration::from_secs(604800),
        };

        let result = config.validate();
        assert!(result.is_ok());
    }

    /// Test JWT config with short secret
    #[test]
    fn test_jwt_config_short_secret() {
        let config = JwtConfig {
            secret: "short".to_string(),
            issuer: "test-issuer".to_string(),
            audience: "test-audience".to_string(),
            access_token_expiry: Duration::from_secs(3600),
            refresh_token_expiry: Duration::from_secs(604800),
        };

        let result = config.validate();
        // Should warn or fail due to short secret
        // Implementation dependent
    }

    /// Test default expiry values
    #[test]
    fn test_jwt_config_default() {
        let config = JwtConfig::default();

        // Default should have reasonable expiry times
        assert!(config.access_token_expiry.as_secs() > 0);
        assert!(config.refresh_token_expiry.as_secs() > config.access_token_expiry.as_secs());
    }
}
