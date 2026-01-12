//! Test utilities for auth service tests

use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Test configuration constants
pub mod constants {
    pub const TEST_USER_ID: &str = "test-user-001";
    pub const TEST_USER_EMAIL: &str = "test@example.com";
    pub const TEST_USER_NAME: &str = "Test User";
    pub const TEST_ORG_ID: &str = "test-org-123";
    pub const TEST_ORG_NAME: &str = "Test Organization";
    pub const TEST_SESSION_ID: &str = "test-session-abc";
    pub const TEST_API_KEY_ID: &str = "test-api-key-xyz";

    // JWT configuration for testing
    pub const TEST_JWT_SECRET: &str = "test-secret-key-for-jwt-signing-must-be-long-enough";
    pub const TEST_JWT_ISSUER: &str = "pistonprotection-test";
    pub const TEST_JWT_AUDIENCE: &str = "pistonprotection-api";
}

/// Test user data
#[derive(Debug, Clone)]
pub struct TestUser {
    pub id: String,
    pub email: String,
    pub name: String,
    pub password_hash: String,
    pub email_verified: bool,
    pub is_admin: bool,
}

impl Default for TestUser {
    fn default() -> Self {
        Self {
            id: constants::TEST_USER_ID.to_string(),
            email: constants::TEST_USER_EMAIL.to_string(),
            name: constants::TEST_USER_NAME.to_string(),
            password_hash: hash_password("password123"),
            email_verified: true,
            is_admin: false,
        }
    }
}

impl TestUser {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_email(mut self, email: &str) -> Self {
        self.email = email.to_string();
        self
    }

    pub fn with_id(mut self, id: &str) -> Self {
        self.id = id.to_string();
        self
    }

    pub fn unverified(mut self) -> Self {
        self.email_verified = false;
        self
    }

    pub fn admin(mut self) -> Self {
        self.is_admin = true;
        self
    }
}

/// Test organization data
#[derive(Debug, Clone)]
pub struct TestOrganization {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub owner_id: String,
}

impl Default for TestOrganization {
    fn default() -> Self {
        Self {
            id: constants::TEST_ORG_ID.to_string(),
            name: constants::TEST_ORG_NAME.to_string(),
            slug: "test-org".to_string(),
            owner_id: constants::TEST_USER_ID.to_string(),
        }
    }
}

impl TestOrganization {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    pub fn with_slug(mut self, slug: &str) -> Self {
        self.slug = slug.to_string();
        self
    }
}

/// Simple password hashing for tests (NOT for production use)
pub fn hash_password(password: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    password.hash(&mut hasher);
    format!("test_hash_{:x}", hasher.finish())
}

/// Verify test password hash
pub fn verify_password(password: &str, hash: &str) -> bool {
    hash_password(password) == hash
}

/// Generate a random test ID
pub fn generate_test_id() -> String {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("test-{:x}", nanos % 0xFFFFFFFF)
}

/// Get current timestamp
pub fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// Get timestamp for a future time
pub fn future_timestamp(duration: Duration) -> u64 {
    current_timestamp() + duration.as_secs()
}

/// Get timestamp for a past time
pub fn past_timestamp(duration: Duration) -> u64 {
    current_timestamp().saturating_sub(duration.as_secs())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_builder() {
        let user = TestUser::new()
            .with_email("custom@example.com")
            .admin();

        assert_eq!(user.email, "custom@example.com");
        assert!(user.is_admin);
        assert!(user.email_verified);
    }

    #[test]
    fn test_organization_builder() {
        let org = TestOrganization::new()
            .with_name("Custom Org")
            .with_slug("custom-org");

        assert_eq!(org.name, "Custom Org");
        assert_eq!(org.slug, "custom-org");
    }

    #[test]
    fn test_password_hashing() {
        let password = "test_password";
        let hash = hash_password(password);

        assert!(verify_password(password, &hash));
        assert!(!verify_password("wrong_password", &hash));
    }

    #[test]
    fn test_generate_test_id() {
        let id1 = generate_test_id();
        let id2 = generate_test_id();

        assert!(id1.starts_with("test-"));
        assert!(id2.starts_with("test-"));
        // IDs should be unique (though not guaranteed in fast execution)
    }

    #[test]
    fn test_timestamps() {
        let now = current_timestamp();
        let future = future_timestamp(Duration::from_secs(3600));
        let past = past_timestamp(Duration::from_secs(3600));

        assert!(future > now);
        assert!(past < now);
    }
}
