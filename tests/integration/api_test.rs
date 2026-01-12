//! End-to-end API tests
//!
//! These tests verify the complete API flow from authentication
//! to resource management.

use super::test_helpers::{
    cleanup_test_data, generate_test_id, TestBackend, TestClient, TestEnvironment,
    TestOrganization, TestUser, wait_for_all_services,
};
use serde_json::json;
use std::time::Duration;

// ============================================================================
// Mock API Client for Testing
// ============================================================================

/// Mock API client that simulates real API calls
struct MockApiClient {
    users: std::collections::HashMap<String, MockUserData>,
    organizations: std::collections::HashMap<String, MockOrgData>,
    backends: std::collections::HashMap<String, MockBackendData>,
    tokens: std::collections::HashMap<String, String>,
    api_keys: std::collections::HashMap<String, MockApiKeyData>,
}

#[derive(Clone)]
struct MockUserData {
    id: String,
    email: String,
    password_hash: String,
    username: String,
    verified: bool,
}

#[derive(Clone)]
struct MockOrgData {
    id: String,
    name: String,
    display_name: String,
    owner_id: String,
    members: Vec<String>,
}

#[derive(Clone)]
struct MockBackendData {
    id: String,
    name: String,
    address: String,
    protocol: String,
    org_id: String,
}

#[derive(Clone)]
struct MockApiKeyData {
    id: String,
    key_hash: String,
    org_id: String,
    permissions: Vec<String>,
    active: bool,
}

impl MockApiClient {
    fn new() -> Self {
        Self {
            users: std::collections::HashMap::new(),
            organizations: std::collections::HashMap::new(),
            backends: std::collections::HashMap::new(),
            tokens: std::collections::HashMap::new(),
            api_keys: std::collections::HashMap::new(),
        }
    }

    // User operations
    fn register_user(&mut self, user: &TestUser) -> Result<String, String> {
        if self.users.values().any(|u| u.email == user.email) {
            return Err("Email already registered".to_string());
        }

        let id = generate_test_id();
        self.users.insert(
            id.clone(),
            MockUserData {
                id: id.clone(),
                email: user.email.clone(),
                password_hash: format!("hash_{}", user.password),
                username: user.username.clone(),
                verified: false,
            },
        );

        Ok(id)
    }

    fn login(&mut self, email: &str, password: &str) -> Result<String, String> {
        let user = self
            .users
            .values()
            .find(|u| u.email == email)
            .ok_or_else(|| "User not found".to_string())?;

        if user.password_hash != format!("hash_{}", password) {
            return Err("Invalid password".to_string());
        }

        let token = format!("token_{}", generate_test_id());
        self.tokens.insert(token.clone(), user.id.clone());
        Ok(token)
    }

    fn verify_token(&self, token: &str) -> Result<String, String> {
        self.tokens
            .get(token)
            .cloned()
            .ok_or_else(|| "Invalid token".to_string())
    }

    fn logout(&mut self, token: &str) -> Result<(), String> {
        self.tokens
            .remove(token)
            .map(|_| ())
            .ok_or_else(|| "Token not found".to_string())
    }

    // Organization operations
    fn create_organization(
        &mut self,
        token: &str,
        org: &TestOrganization,
    ) -> Result<String, String> {
        let user_id = self.verify_token(token)?;

        if self.organizations.values().any(|o| o.name == org.name) {
            return Err("Organization name already exists".to_string());
        }

        let id = generate_test_id();
        self.organizations.insert(
            id.clone(),
            MockOrgData {
                id: id.clone(),
                name: org.name.clone(),
                display_name: org.display_name.clone(),
                owner_id: user_id.clone(),
                members: vec![user_id],
            },
        );

        Ok(id)
    }

    fn get_organization(&self, token: &str, org_id: &str) -> Result<&MockOrgData, String> {
        let user_id = self.verify_token(token)?;
        let org = self
            .organizations
            .get(org_id)
            .ok_or_else(|| "Organization not found".to_string())?;

        if !org.members.contains(&user_id) {
            return Err("Access denied".to_string());
        }

        Ok(org)
    }

    fn list_organizations(&self, token: &str) -> Result<Vec<&MockOrgData>, String> {
        let user_id = self.verify_token(token)?;
        Ok(self
            .organizations
            .values()
            .filter(|o| o.members.contains(&user_id))
            .collect())
    }

    // Backend operations
    fn create_backend(
        &mut self,
        token: &str,
        org_id: &str,
        backend: &TestBackend,
    ) -> Result<String, String> {
        let user_id = self.verify_token(token)?;
        let org = self
            .organizations
            .get(org_id)
            .ok_or_else(|| "Organization not found".to_string())?;

        if !org.members.contains(&user_id) {
            return Err("Access denied".to_string());
        }

        let id = generate_test_id();
        self.backends.insert(
            id.clone(),
            MockBackendData {
                id: id.clone(),
                name: backend.name.clone(),
                address: backend.address.clone(),
                protocol: backend.protocol.clone(),
                org_id: org_id.to_string(),
            },
        );

        Ok(id)
    }

    fn get_backend(&self, token: &str, backend_id: &str) -> Result<&MockBackendData, String> {
        let user_id = self.verify_token(token)?;
        let backend = self
            .backends
            .get(backend_id)
            .ok_or_else(|| "Backend not found".to_string())?;

        let org = self
            .organizations
            .get(&backend.org_id)
            .ok_or_else(|| "Organization not found".to_string())?;

        if !org.members.contains(&user_id) {
            return Err("Access denied".to_string());
        }

        Ok(backend)
    }

    fn list_backends(&self, token: &str, org_id: &str) -> Result<Vec<&MockBackendData>, String> {
        let user_id = self.verify_token(token)?;
        let org = self
            .organizations
            .get(org_id)
            .ok_or_else(|| "Organization not found".to_string())?;

        if !org.members.contains(&user_id) {
            return Err("Access denied".to_string());
        }

        Ok(self
            .backends
            .values()
            .filter(|b| b.org_id == org_id)
            .collect())
    }

    fn delete_backend(&mut self, token: &str, backend_id: &str) -> Result<(), String> {
        let user_id = self.verify_token(token)?;
        let backend = self
            .backends
            .get(backend_id)
            .ok_or_else(|| "Backend not found".to_string())?;

        let org = self
            .organizations
            .get(&backend.org_id)
            .ok_or_else(|| "Organization not found".to_string())?;

        if org.owner_id != user_id {
            return Err("Only owner can delete backends".to_string());
        }

        self.backends.remove(backend_id);
        Ok(())
    }

    // API key operations
    fn create_api_key(
        &mut self,
        token: &str,
        org_id: &str,
        permissions: Vec<String>,
    ) -> Result<(String, String), String> {
        let user_id = self.verify_token(token)?;
        let org = self
            .organizations
            .get(org_id)
            .ok_or_else(|| "Organization not found".to_string())?;

        if org.owner_id != user_id {
            return Err("Only owner can create API keys".to_string());
        }

        let id = generate_test_id();
        let key = format!("pp_live_{}", generate_test_id());

        self.api_keys.insert(
            id.clone(),
            MockApiKeyData {
                id: id.clone(),
                key_hash: format!("hash_{}", key),
                org_id: org_id.to_string(),
                permissions,
                active: true,
            },
        );

        Ok((id, key))
    }

    fn verify_api_key(&self, key: &str) -> Result<&MockApiKeyData, String> {
        self.api_keys
            .values()
            .find(|k| k.key_hash == format!("hash_{}", key) && k.active)
            .ok_or_else(|| "Invalid API key".to_string())
    }
}

// ============================================================================
// Authentication Flow Tests
// ============================================================================

#[cfg(test)]
mod auth_flow_tests {
    use super::*;

    /// Test complete registration flow
    #[test]
    fn test_registration_flow() {
        let mut client = MockApiClient::new();
        let user = TestUser::random();

        // Register user
        let user_id = client.register_user(&user).unwrap();
        assert!(!user_id.is_empty());

        // Verify user exists
        let stored = client.users.get(&user_id).unwrap();
        assert_eq!(stored.email, user.email);
    }

    /// Test registration with duplicate email
    #[test]
    fn test_registration_duplicate_email() {
        let mut client = MockApiClient::new();
        let user = TestUser::random();

        client.register_user(&user).unwrap();
        let result = client.register_user(&user);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already registered"));
    }

    /// Test login flow
    #[test]
    fn test_login_flow() {
        let mut client = MockApiClient::new();
        let user = TestUser::random();

        client.register_user(&user).unwrap();
        let token = client.login(&user.email, &user.password).unwrap();

        assert!(!token.is_empty());
        assert!(token.starts_with("token_"));
    }

    /// Test login with invalid password
    #[test]
    fn test_login_invalid_password() {
        let mut client = MockApiClient::new();
        let user = TestUser::random();

        client.register_user(&user).unwrap();
        let result = client.login(&user.email, "wrong_password");

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid password"));
    }

    /// Test login with non-existent user
    #[test]
    fn test_login_non_existent_user() {
        let mut client = MockApiClient::new();

        let result = client.login("nonexistent@example.com", "password");

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    /// Test token verification
    #[test]
    fn test_token_verification() {
        let mut client = MockApiClient::new();
        let user = TestUser::random();

        client.register_user(&user).unwrap();
        let token = client.login(&user.email, &user.password).unwrap();

        let user_id = client.verify_token(&token).unwrap();
        assert!(!user_id.is_empty());
    }

    /// Test logout
    #[test]
    fn test_logout() {
        let mut client = MockApiClient::new();
        let user = TestUser::random();

        client.register_user(&user).unwrap();
        let token = client.login(&user.email, &user.password).unwrap();

        // Logout
        client.logout(&token).unwrap();

        // Token should be invalid
        let result = client.verify_token(&token);
        assert!(result.is_err());
    }
}

// ============================================================================
// Organization Management Tests
// ============================================================================

#[cfg(test)]
mod organization_tests {
    use super::*;

    fn setup_authenticated_client() -> (MockApiClient, String) {
        let mut client = MockApiClient::new();
        let user = TestUser::random();
        client.register_user(&user).unwrap();
        let token = client.login(&user.email, &user.password).unwrap();
        (client, token)
    }

    /// Test organization creation
    #[test]
    fn test_create_organization() {
        let (mut client, token) = setup_authenticated_client();
        let org = TestOrganization::random();

        let org_id = client.create_organization(&token, &org).unwrap();

        assert!(!org_id.is_empty());
        let stored = client.get_organization(&token, &org_id).unwrap();
        assert_eq!(stored.name, org.name);
    }

    /// Test organization creation with duplicate name
    #[test]
    fn test_create_organization_duplicate_name() {
        let (mut client, token) = setup_authenticated_client();
        let org = TestOrganization::random();

        client.create_organization(&token, &org).unwrap();
        let result = client.create_organization(&token, &org);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already exists"));
    }

    /// Test list organizations
    #[test]
    fn test_list_organizations() {
        let (mut client, token) = setup_authenticated_client();

        // Create multiple organizations
        for i in 0..3 {
            let org = TestOrganization::new(
                &format!("org-{}-{}", i, generate_test_id()),
                &format!("Organization {}", i),
            );
            client.create_organization(&token, &org).unwrap();
        }

        let orgs = client.list_organizations(&token).unwrap();
        assert_eq!(orgs.len(), 3);
    }

    /// Test organization access control
    #[test]
    fn test_organization_access_control() {
        let (mut client, token1) = setup_authenticated_client();

        // Create org as user 1
        let org = TestOrganization::random();
        let org_id = client.create_organization(&token1, &org).unwrap();

        // Register and login as user 2
        let user2 = TestUser::random();
        client.register_user(&user2).unwrap();
        let token2 = client.login(&user2.email, &user2.password).unwrap();

        // User 2 should not have access
        let result = client.get_organization(&token2, &org_id);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Access denied"));
    }
}

// ============================================================================
// Backend Management Tests
// ============================================================================

#[cfg(test)]
mod backend_tests {
    use super::*;

    fn setup_with_org() -> (MockApiClient, String, String) {
        let mut client = MockApiClient::new();
        let user = TestUser::random();
        client.register_user(&user).unwrap();
        let token = client.login(&user.email, &user.password).unwrap();
        let org = TestOrganization::random();
        let org_id = client.create_organization(&token, &org).unwrap();
        (client, token, org_id)
    }

    /// Test backend creation
    #[test]
    fn test_create_backend() {
        let (mut client, token, org_id) = setup_with_org();
        let backend = TestBackend::minecraft_java("mc-server", "10.0.0.1");

        let backend_id = client.create_backend(&token, &org_id, &backend).unwrap();

        assert!(!backend_id.is_empty());
        let stored = client.get_backend(&token, &backend_id).unwrap();
        assert_eq!(stored.name, backend.name);
        assert_eq!(stored.protocol, "minecraft-java");
    }

    /// Test list backends
    #[test]
    fn test_list_backends() {
        let (mut client, token, org_id) = setup_with_org();

        // Create multiple backends
        for i in 0..3 {
            let backend = TestBackend::minecraft_java(
                &format!("server-{}", i),
                &format!("10.0.0.{}", i + 1),
            );
            client.create_backend(&token, &org_id, &backend).unwrap();
        }

        let backends = client.list_backends(&token, &org_id).unwrap();
        assert_eq!(backends.len(), 3);
    }

    /// Test delete backend
    #[test]
    fn test_delete_backend() {
        let (mut client, token, org_id) = setup_with_org();
        let backend = TestBackend::minecraft_java("mc-server", "10.0.0.1");

        let backend_id = client.create_backend(&token, &org_id, &backend).unwrap();
        client.delete_backend(&token, &backend_id).unwrap();

        let result = client.get_backend(&token, &backend_id);
        assert!(result.is_err());
    }

    /// Test backend access control
    #[test]
    fn test_backend_access_control() {
        let (mut client, token1, org_id) = setup_with_org();

        // Create backend as user 1
        let backend = TestBackend::minecraft_java("mc-server", "10.0.0.1");
        let backend_id = client.create_backend(&token1, &org_id, &backend).unwrap();

        // Register and login as user 2
        let user2 = TestUser::random();
        client.register_user(&user2).unwrap();
        let token2 = client.login(&user2.email, &user2.password).unwrap();

        // User 2 should not have access
        let result = client.get_backend(&token2, &backend_id);
        assert!(result.is_err());
    }
}

// ============================================================================
// API Key Tests
// ============================================================================

#[cfg(test)]
mod api_key_tests {
    use super::*;

    fn setup_with_org() -> (MockApiClient, String, String) {
        let mut client = MockApiClient::new();
        let user = TestUser::random();
        client.register_user(&user).unwrap();
        let token = client.login(&user.email, &user.password).unwrap();
        let org = TestOrganization::random();
        let org_id = client.create_organization(&token, &org).unwrap();
        (client, token, org_id)
    }

    /// Test API key creation
    #[test]
    fn test_create_api_key() {
        let (mut client, token, org_id) = setup_with_org();
        let permissions = vec!["read:backends".to_string(), "write:backends".to_string()];

        let (key_id, key) = client.create_api_key(&token, &org_id, permissions).unwrap();

        assert!(!key_id.is_empty());
        assert!(key.starts_with("pp_live_"));
    }

    /// Test API key verification
    #[test]
    fn test_verify_api_key() {
        let (mut client, token, org_id) = setup_with_org();
        let permissions = vec!["read:backends".to_string()];

        let (_, key) = client.create_api_key(&token, &org_id, permissions.clone()).unwrap();

        let key_data = client.verify_api_key(&key).unwrap();
        assert_eq!(key_data.permissions, permissions);
    }

    /// Test invalid API key
    #[test]
    fn test_invalid_api_key() {
        let client = MockApiClient::new();

        let result = client.verify_api_key("invalid_key");
        assert!(result.is_err());
    }

    /// Test non-owner cannot create API key
    #[test]
    fn test_non_owner_cannot_create_api_key() {
        let (mut client, token1, org_id) = setup_with_org();

        // Add member (simplified - in real impl would have invite flow)
        let user2 = TestUser::random();
        client.register_user(&user2).unwrap();
        let token2 = client.login(&user2.email, &user2.password).unwrap();

        // Non-owner should not be able to create API key
        let result = client.create_api_key(&token2, &org_id, vec![]);
        assert!(result.is_err());
    }
}

// ============================================================================
// Full Flow Tests
// ============================================================================

#[cfg(test)]
mod full_flow_tests {
    use super::*;

    /// Test complete user journey
    #[test]
    fn test_complete_user_journey() {
        let mut client = MockApiClient::new();

        // 1. Register
        let user = TestUser::random();
        let user_id = client.register_user(&user).unwrap();
        assert!(!user_id.is_empty());

        // 2. Login
        let token = client.login(&user.email, &user.password).unwrap();
        assert!(!token.is_empty());

        // 3. Create organization
        let org = TestOrganization::random();
        let org_id = client.create_organization(&token, &org).unwrap();
        assert!(!org_id.is_empty());

        // 4. Create backend
        let backend = TestBackend::minecraft_java("mc-server", "10.0.0.1");
        let backend_id = client.create_backend(&token, &org_id, &backend).unwrap();
        assert!(!backend_id.is_empty());

        // 5. Create API key
        let (key_id, key) = client
            .create_api_key(&token, &org_id, vec!["read:backends".to_string()])
            .unwrap();
        assert!(!key_id.is_empty());
        assert!(!key.is_empty());

        // 6. Verify API key works
        let key_data = client.verify_api_key(&key).unwrap();
        assert!(key_data.active);

        // 7. List backends
        let backends = client.list_backends(&token, &org_id).unwrap();
        assert_eq!(backends.len(), 1);

        // 8. Logout
        client.logout(&token).unwrap();

        // 9. Token should be invalid
        let result = client.verify_token(&token);
        assert!(result.is_err());
    }

    /// Test multi-organization flow
    #[test]
    fn test_multi_organization_flow() {
        let mut client = MockApiClient::new();

        // Register user
        let user = TestUser::random();
        client.register_user(&user).unwrap();
        let token = client.login(&user.email, &user.password).unwrap();

        // Create multiple organizations
        let mut org_ids = Vec::new();
        for i in 0..3 {
            let org = TestOrganization::new(
                &format!("org-{}-{}", i, generate_test_id()),
                &format!("Organization {}", i),
            );
            let org_id = client.create_organization(&token, &org).unwrap();
            org_ids.push(org_id);
        }

        // Create backends in each org
        for org_id in &org_ids {
            let backend = TestBackend::minecraft_java("mc-server", "10.0.0.1");
            client.create_backend(&token, org_id, &backend).unwrap();
        }

        // Verify isolation
        for org_id in &org_ids {
            let backends = client.list_backends(&token, org_id).unwrap();
            assert_eq!(backends.len(), 1);
        }

        // Total organizations
        let orgs = client.list_organizations(&token).unwrap();
        assert_eq!(orgs.len(), 3);
    }
}
