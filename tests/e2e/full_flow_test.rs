//! Full end-to-end flow tests
//!
//! These tests verify complete user flows from registration
//! through backend protection.

use super::test_fixtures::{generate_test_id, TestDDoSProtection, TestFilterRule};
use std::collections::HashMap;

// ============================================================================
// Mock System State
// ============================================================================

/// Complete mock system for e2e testing
struct MockSystem {
    users: HashMap<String, MockUser>,
    organizations: HashMap<String, MockOrganization>,
    backends: HashMap<String, MockBackend>,
    protections: HashMap<String, MockProtection>,
    filter_rules: HashMap<String, MockRule>,
    sessions: HashMap<String, String>, // token -> user_id
    api_keys: HashMap<String, MockApiKey>,
    metrics: MockMetrics,
    gateway_state: MockGatewayState,
}

#[derive(Clone)]
struct MockUser {
    id: String,
    email: String,
    password_hash: String,
    verified: bool,
}

#[derive(Clone)]
struct MockOrganization {
    id: String,
    name: String,
    owner_id: String,
    members: Vec<String>,
    plan: String,
}

#[derive(Clone)]
struct MockBackend {
    id: String,
    org_id: String,
    name: String,
    address: String,
    protocol: String,
    status: String,
}

#[derive(Clone)]
struct MockProtection {
    id: String,
    backend_id: String,
    protection_level: i32,
    status: String,
    workers: i32,
}

#[derive(Clone)]
struct MockRule {
    id: String,
    backend_id: String,
    rule_type: String,
    action: String,
    priority: i32,
    active: bool,
}

#[derive(Clone)]
struct MockApiKey {
    id: String,
    org_id: String,
    key_hash: String,
    permissions: Vec<String>,
    active: bool,
}

struct MockMetrics {
    backend_metrics: HashMap<String, BackendMetrics>,
}

#[derive(Clone, Default)]
struct BackendMetrics {
    total_requests: u64,
    blocked_requests: u64,
    bytes_in: u64,
    bytes_out: u64,
    active_connections: u32,
}

struct MockGatewayState {
    synced_backends: Vec<String>,
    synced_rules: Vec<String>,
    active: bool,
}

impl MockSystem {
    fn new() -> Self {
        Self {
            users: HashMap::new(),
            organizations: HashMap::new(),
            backends: HashMap::new(),
            protections: HashMap::new(),
            filter_rules: HashMap::new(),
            sessions: HashMap::new(),
            api_keys: HashMap::new(),
            metrics: MockMetrics {
                backend_metrics: HashMap::new(),
            },
            gateway_state: MockGatewayState {
                synced_backends: Vec::new(),
                synced_rules: Vec::new(),
                active: true,
            },
        }
    }

    // ========== User Operations ==========

    fn register(&mut self, email: &str, password: &str) -> Result<String, String> {
        if self.users.values().any(|u| u.email == email) {
            return Err("Email already registered".to_string());
        }

        let id = generate_test_id();
        self.users.insert(
            id.clone(),
            MockUser {
                id: id.clone(),
                email: email.to_string(),
                password_hash: format!("hash_{}", password),
                verified: false,
            },
        );

        Ok(id)
    }

    fn verify_email(&mut self, user_id: &str) -> Result<(), String> {
        let user = self.users.get_mut(user_id).ok_or("User not found")?;
        user.verified = true;
        Ok(())
    }

    fn login(&mut self, email: &str, password: &str) -> Result<String, String> {
        let user = self
            .users
            .values()
            .find(|u| u.email == email)
            .ok_or("User not found")?;

        if user.password_hash != format!("hash_{}", password) {
            return Err("Invalid password".to_string());
        }

        if !user.verified {
            return Err("Email not verified".to_string());
        }

        let token = format!("token_{}", generate_test_id());
        self.sessions.insert(token.clone(), user.id.clone());
        Ok(token)
    }

    fn get_current_user(&self, token: &str) -> Result<&MockUser, String> {
        let user_id = self.sessions.get(token).ok_or("Invalid token")?;
        self.users.get(user_id).ok_or("User not found".to_string())
    }

    // ========== Organization Operations ==========

    fn create_organization(&mut self, token: &str, name: &str, plan: &str) -> Result<String, String> {
        let user_id = self.sessions.get(token).ok_or("Invalid token")?.clone();

        if self.organizations.values().any(|o| o.name == name) {
            return Err("Organization name already exists".to_string());
        }

        let id = generate_test_id();
        self.organizations.insert(
            id.clone(),
            MockOrganization {
                id: id.clone(),
                name: name.to_string(),
                owner_id: user_id.clone(),
                members: vec![user_id],
                plan: plan.to_string(),
            },
        );

        Ok(id)
    }

    fn invite_member(&mut self, token: &str, org_id: &str, user_email: &str) -> Result<(), String> {
        let user_id = self.sessions.get(token).ok_or("Invalid token")?;
        let org = self.organizations.get_mut(org_id).ok_or("Organization not found")?;

        if org.owner_id != *user_id {
            return Err("Only owner can invite members".to_string());
        }

        let invite_user = self
            .users
            .values()
            .find(|u| u.email == user_email)
            .ok_or("User not found")?;

        if !org.members.contains(&invite_user.id) {
            org.members.push(invite_user.id.clone());
        }

        Ok(())
    }

    fn has_permission(&self, token: &str, org_id: &str) -> bool {
        if let Some(user_id) = self.sessions.get(token) {
            if let Some(org) = self.organizations.get(org_id) {
                return org.members.contains(user_id);
            }
        }
        false
    }

    // ========== Backend Operations ==========

    fn create_backend(&mut self, token: &str, org_id: &str, name: &str, address: &str, protocol: &str) -> Result<String, String> {
        if !self.has_permission(token, org_id) {
            return Err("Access denied".to_string());
        }

        let id = generate_test_id();
        self.backends.insert(
            id.clone(),
            MockBackend {
                id: id.clone(),
                org_id: org_id.to_string(),
                name: name.to_string(),
                address: address.to_string(),
                protocol: protocol.to_string(),
                status: "pending".to_string(),
            },
        );

        // Initialize metrics
        self.metrics
            .backend_metrics
            .insert(id.clone(), BackendMetrics::default());

        Ok(id)
    }

    fn get_backend(&self, token: &str, backend_id: &str) -> Result<&MockBackend, String> {
        let backend = self.backends.get(backend_id).ok_or("Backend not found")?;
        if !self.has_permission(token, &backend.org_id) {
            return Err("Access denied".to_string());
        }
        Ok(backend)
    }

    fn list_backends(&self, token: &str, org_id: &str) -> Result<Vec<&MockBackend>, String> {
        if !self.has_permission(token, org_id) {
            return Err("Access denied".to_string());
        }
        Ok(self
            .backends
            .values()
            .filter(|b| b.org_id == org_id)
            .collect())
    }

    // ========== Protection Operations ==========

    fn enable_protection(&mut self, token: &str, backend_id: &str, level: i32) -> Result<String, String> {
        let backend = self.backends.get(backend_id).ok_or("Backend not found")?;
        if !self.has_permission(token, &backend.org_id) {
            return Err("Access denied".to_string());
        }

        let id = generate_test_id();
        self.protections.insert(
            id.clone(),
            MockProtection {
                id: id.clone(),
                backend_id: backend_id.to_string(),
                protection_level: level,
                status: "provisioning".to_string(),
                workers: 0,
            },
        );

        // Simulate gateway sync
        self.gateway_state.synced_backends.push(backend_id.to_string());

        // Update backend status
        if let Some(backend) = self.backends.get_mut(backend_id) {
            backend.status = "protected".to_string();
        }

        // Simulate workers coming up
        if let Some(protection) = self.protections.get_mut(&id) {
            protection.workers = 2;
            protection.status = "active".to_string();
        }

        Ok(id)
    }

    fn get_protection_status(&self, token: &str, protection_id: &str) -> Result<&MockProtection, String> {
        let protection = self.protections.get(protection_id).ok_or("Protection not found")?;
        let backend = self.backends.get(&protection.backend_id).ok_or("Backend not found")?;
        if !self.has_permission(token, &backend.org_id) {
            return Err("Access denied".to_string());
        }
        Ok(protection)
    }

    // ========== Filter Rule Operations ==========

    fn create_filter_rule(&mut self, token: &str, backend_id: &str, rule_type: &str, action: &str, priority: i32) -> Result<String, String> {
        let backend = self.backends.get(backend_id).ok_or("Backend not found")?;
        if !self.has_permission(token, &backend.org_id) {
            return Err("Access denied".to_string());
        }

        let id = generate_test_id();
        self.filter_rules.insert(
            id.clone(),
            MockRule {
                id: id.clone(),
                backend_id: backend_id.to_string(),
                rule_type: rule_type.to_string(),
                action: action.to_string(),
                priority,
                active: true,
            },
        );

        // Sync to gateway
        self.gateway_state.synced_rules.push(id.clone());

        Ok(id)
    }

    fn list_filter_rules(&self, token: &str, backend_id: &str) -> Result<Vec<&MockRule>, String> {
        let backend = self.backends.get(backend_id).ok_or("Backend not found")?;
        if !self.has_permission(token, &backend.org_id) {
            return Err("Access denied".to_string());
        }
        Ok(self
            .filter_rules
            .values()
            .filter(|r| r.backend_id == backend_id)
            .collect())
    }

    // ========== API Key Operations ==========

    fn create_api_key(&mut self, token: &str, org_id: &str, permissions: Vec<&str>) -> Result<(String, String), String> {
        let user_id = self.sessions.get(token).ok_or("Invalid token")?;
        let org = self.organizations.get(org_id).ok_or("Organization not found")?;

        if org.owner_id != *user_id {
            return Err("Only owner can create API keys".to_string());
        }

        let id = generate_test_id();
        let key = format!("pp_live_{}", generate_test_id());

        self.api_keys.insert(
            id.clone(),
            MockApiKey {
                id: id.clone(),
                org_id: org_id.to_string(),
                key_hash: format!("hash_{}", key),
                permissions: permissions.iter().map(|s| s.to_string()).collect(),
                active: true,
            },
        );

        Ok((id, key))
    }

    fn validate_api_key(&self, key: &str) -> Result<&MockApiKey, String> {
        self.api_keys
            .values()
            .find(|k| k.key_hash == format!("hash_{}", key) && k.active)
            .ok_or("Invalid API key".to_string())
    }

    // ========== Metrics Operations ==========

    fn get_backend_metrics(&self, token: &str, backend_id: &str) -> Result<&BackendMetrics, String> {
        let backend = self.backends.get(backend_id).ok_or("Backend not found")?;
        if !self.has_permission(token, &backend.org_id) {
            return Err("Access denied".to_string());
        }
        self.metrics
            .backend_metrics
            .get(backend_id)
            .ok_or("Metrics not found".to_string())
    }

    fn simulate_traffic(&mut self, backend_id: &str, requests: u64, blocked: u64) {
        if let Some(metrics) = self.metrics.backend_metrics.get_mut(backend_id) {
            metrics.total_requests += requests;
            metrics.blocked_requests += blocked;
            metrics.bytes_in += requests * 500;
            metrics.bytes_out += (requests - blocked) * 1000;
        }
    }
}

// ============================================================================
// Complete User Journey Tests
// ============================================================================

#[cfg(test)]
mod user_journey_tests {
    use super::*;

    /// Test new user onboarding flow
    #[test]
    fn test_new_user_onboarding() {
        let mut system = MockSystem::new();

        // 1. Register
        let user_id = system.register("user@example.com", "SecurePassword123!").unwrap();
        assert!(!user_id.is_empty());

        // 2. Verify email
        system.verify_email(&user_id).unwrap();

        // 3. Login
        let token = system.login("user@example.com", "SecurePassword123!").unwrap();
        assert!(!token.is_empty());

        // 4. Create organization
        let org_id = system.create_organization(&token, "my-org", "starter").unwrap();
        assert!(!org_id.is_empty());

        // 5. Create backend
        let backend_id = system
            .create_backend(&token, &org_id, "mc-server", "10.0.0.1:25565", "minecraft-java")
            .unwrap();
        assert!(!backend_id.is_empty());

        // Verify everything is set up
        let user = system.get_current_user(&token).unwrap();
        assert!(user.verified);

        let backends = system.list_backends(&token, &org_id).unwrap();
        assert_eq!(backends.len(), 1);
    }

    /// Test team collaboration flow
    #[test]
    fn test_team_collaboration() {
        let mut system = MockSystem::new();

        // User 1 creates organization
        let user1_id = system.register("owner@example.com", "Password1!").unwrap();
        system.verify_email(&user1_id).unwrap();
        let token1 = system.login("owner@example.com", "Password1!").unwrap();
        let org_id = system.create_organization(&token1, "team-org", "pro").unwrap();

        // User 2 registers
        let user2_id = system.register("member@example.com", "Password2!").unwrap();
        system.verify_email(&user2_id).unwrap();

        // Owner invites User 2
        system.invite_member(&token1, &org_id, "member@example.com").unwrap();

        // User 2 logs in and accesses org
        let token2 = system.login("member@example.com", "Password2!").unwrap();

        // User 2 creates a backend in the org
        let backend_id = system
            .create_backend(&token2, &org_id, "member-server", "10.0.0.2:25565", "minecraft-java")
            .unwrap();

        // Both users can see the backend
        let backends1 = system.list_backends(&token1, &org_id).unwrap();
        let backends2 = system.list_backends(&token2, &org_id).unwrap();
        assert_eq!(backends1.len(), backends2.len());
    }
}

// ============================================================================
// Protection Flow Tests
// ============================================================================

#[cfg(test)]
mod protection_flow_tests {
    use super::*;

    fn setup_system_with_backend() -> (MockSystem, String, String, String) {
        let mut system = MockSystem::new();
        let user_id = system.register("user@example.com", "Password!").unwrap();
        system.verify_email(&user_id).unwrap();
        let token = system.login("user@example.com", "Password!").unwrap();
        let org_id = system.create_organization(&token, "my-org", "pro").unwrap();
        let backend_id = system
            .create_backend(&token, &org_id, "game-server", "10.0.0.1:25565", "minecraft-java")
            .unwrap();
        (system, token, org_id, backend_id)
    }

    /// Test enabling protection
    #[test]
    fn test_enable_protection() {
        let (mut system, token, _, backend_id) = setup_system_with_backend();

        // Enable protection
        let protection_id = system.enable_protection(&token, &backend_id, 3).unwrap();

        // Check status
        let protection = system.get_protection_status(&token, &protection_id).unwrap();
        assert_eq!(protection.status, "active");
        assert_eq!(protection.workers, 2);
        assert_eq!(protection.protection_level, 3);

        // Backend should be marked as protected
        let backend = system.get_backend(&token, &backend_id).unwrap();
        assert_eq!(backend.status, "protected");

        // Gateway should be synced
        assert!(system.gateway_state.synced_backends.contains(&backend_id));
    }

    /// Test creating filter rules
    #[test]
    fn test_create_filter_rules() {
        let (mut system, token, _, backend_id) = setup_system_with_backend();

        // Enable protection first
        system.enable_protection(&token, &backend_id, 3).unwrap();

        // Create IP blocklist rule
        let rule1_id = system
            .create_filter_rule(&token, &backend_id, "ip-blocklist", "drop", 100)
            .unwrap();

        // Create rate limit rule
        let rule2_id = system
            .create_filter_rule(&token, &backend_id, "rate-limit", "rate-limit", 50)
            .unwrap();

        // List rules
        let rules = system.list_filter_rules(&token, &backend_id).unwrap();
        assert_eq!(rules.len(), 2);

        // Check gateway sync
        assert!(system.gateway_state.synced_rules.contains(&rule1_id));
        assert!(system.gateway_state.synced_rules.contains(&rule2_id));
    }

    /// Test protection with traffic simulation
    #[test]
    fn test_protection_with_traffic() {
        let (mut system, token, _, backend_id) = setup_system_with_backend();

        // Enable protection
        system.enable_protection(&token, &backend_id, 4).unwrap();

        // Create blocklist
        system
            .create_filter_rule(&token, &backend_id, "ip-blocklist", "drop", 100)
            .unwrap();

        // Simulate traffic (some blocked)
        system.simulate_traffic(&backend_id, 1000, 150);

        // Check metrics
        let metrics = system.get_backend_metrics(&token, &backend_id).unwrap();
        assert_eq!(metrics.total_requests, 1000);
        assert_eq!(metrics.blocked_requests, 150);
        assert!(metrics.bytes_in > 0);
        assert!(metrics.bytes_out > 0);
    }
}

// ============================================================================
// API Key Flow Tests
// ============================================================================

#[cfg(test)]
mod api_key_flow_tests {
    use super::*;

    /// Test API key creation and usage
    #[test]
    fn test_api_key_flow() {
        let mut system = MockSystem::new();

        // Setup user and org
        let user_id = system.register("owner@example.com", "Password!").unwrap();
        system.verify_email(&user_id).unwrap();
        let token = system.login("owner@example.com", "Password!").unwrap();
        let org_id = system.create_organization(&token, "api-org", "enterprise").unwrap();

        // Create API key
        let (key_id, key) = system
            .create_api_key(&token, &org_id, vec!["read:backends", "write:backends", "read:metrics"])
            .unwrap();

        // Validate key
        let key_data = system.validate_api_key(&key).unwrap();
        assert!(key_data.active);
        assert!(key_data.permissions.contains(&"read:backends".to_string()));
        assert!(key_data.permissions.contains(&"read:metrics".to_string()));
    }

    /// Test non-owner cannot create API keys
    #[test]
    fn test_non_owner_cannot_create_api_key() {
        let mut system = MockSystem::new();

        // Owner creates org
        let owner_id = system.register("owner@example.com", "Password1!").unwrap();
        system.verify_email(&owner_id).unwrap();
        let owner_token = system.login("owner@example.com", "Password1!").unwrap();
        let org_id = system.create_organization(&owner_token, "api-org", "pro").unwrap();

        // Add member
        let member_id = system.register("member@example.com", "Password2!").unwrap();
        system.verify_email(&member_id).unwrap();
        system.invite_member(&owner_token, &org_id, "member@example.com").unwrap();

        // Member tries to create API key
        let member_token = system.login("member@example.com", "Password2!").unwrap();
        let result = system.create_api_key(&member_token, &org_id, vec!["read:backends"]);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("owner"));
    }
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[cfg(test)]
mod error_handling_tests {
    use super::*;

    /// Test unverified email login
    #[test]
    fn test_unverified_email_login() {
        let mut system = MockSystem::new();
        system.register("user@example.com", "Password!").unwrap();
        // Don't verify email

        let result = system.login("user@example.com", "Password!");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not verified"));
    }

    /// Test access control
    #[test]
    fn test_access_control() {
        let mut system = MockSystem::new();

        // User 1 creates org and backend
        let user1_id = system.register("user1@example.com", "Password1!").unwrap();
        system.verify_email(&user1_id).unwrap();
        let token1 = system.login("user1@example.com", "Password1!").unwrap();
        let org1_id = system.create_organization(&token1, "org1", "pro").unwrap();
        let backend1_id = system
            .create_backend(&token1, &org1_id, "server1", "10.0.0.1:25565", "minecraft-java")
            .unwrap();

        // User 2 tries to access User 1's backend
        let user2_id = system.register("user2@example.com", "Password2!").unwrap();
        system.verify_email(&user2_id).unwrap();
        let token2 = system.login("user2@example.com", "Password2!").unwrap();

        let result = system.get_backend(&token2, &backend1_id);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Access denied"));
    }

    /// Test invalid token
    #[test]
    fn test_invalid_token() {
        let mut system = MockSystem::new();

        let result = system.create_organization("invalid_token", "test-org", "starter");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid token"));
    }
}

// ============================================================================
// Complete E2E Flow Test
// ============================================================================

#[cfg(test)]
mod complete_flow_tests {
    use super::*;

    /// Test complete e2e flow from signup to protection
    #[test]
    fn test_complete_e2e_flow() {
        let mut system = MockSystem::new();

        // ===== Phase 1: User Onboarding =====

        // Register and verify
        let user_id = system.register("admin@gameserver.com", "SuperSecure123!").unwrap();
        system.verify_email(&user_id).unwrap();

        // Login
        let token = system.login("admin@gameserver.com", "SuperSecure123!").unwrap();

        // Create organization
        let org_id = system.create_organization(&token, "awesome-games", "enterprise").unwrap();

        // ===== Phase 2: Backend Setup =====

        // Create Minecraft Java backend
        let mc_java_id = system
            .create_backend(
                &token,
                &org_id,
                "survival-server",
                "mc.awesome-games.com:25565",
                "minecraft-java",
            )
            .unwrap();

        // Create Minecraft Bedrock backend
        let mc_bedrock_id = system
            .create_backend(
                &token,
                &org_id,
                "bedrock-server",
                "bedrock.awesome-games.com:19132",
                "minecraft-bedrock",
            )
            .unwrap();

        // ===== Phase 3: Enable Protection =====

        // Enable high protection for Java server
        let java_protection_id = system.enable_protection(&token, &mc_java_id, 4).unwrap();

        // Enable standard protection for Bedrock
        let bedrock_protection_id = system.enable_protection(&token, &mc_bedrock_id, 3).unwrap();

        // Verify protection status
        let java_protection = system.get_protection_status(&token, &java_protection_id).unwrap();
        assert_eq!(java_protection.status, "active");

        // ===== Phase 4: Configure Filter Rules =====

        // Block known bad IPs for Java server
        system
            .create_filter_rule(&token, &mc_java_id, "ip-blocklist", "drop", 100)
            .unwrap();

        // Rate limit for Java server
        system
            .create_filter_rule(&token, &mc_java_id, "rate-limit", "rate-limit", 50)
            .unwrap();

        // Geo-block for Bedrock server
        system
            .create_filter_rule(&token, &mc_bedrock_id, "geo-block", "drop", 90)
            .unwrap();

        // ===== Phase 5: Create API Key for Automation =====

        let (api_key_id, api_key) = system
            .create_api_key(&token, &org_id, vec!["read:backends", "read:metrics"])
            .unwrap();

        // Validate API key works
        let key_data = system.validate_api_key(&api_key).unwrap();
        assert!(key_data.active);

        // ===== Phase 6: Simulate Traffic & Check Metrics =====

        // Simulate traffic on Java server
        system.simulate_traffic(&mc_java_id, 10000, 500);

        // Simulate traffic on Bedrock server
        system.simulate_traffic(&mc_bedrock_id, 5000, 200);

        // Check metrics
        let java_metrics = system.get_backend_metrics(&token, &mc_java_id).unwrap();
        assert_eq!(java_metrics.total_requests, 10000);
        assert_eq!(java_metrics.blocked_requests, 500);

        let bedrock_metrics = system.get_backend_metrics(&token, &mc_bedrock_id).unwrap();
        assert_eq!(bedrock_metrics.total_requests, 5000);
        assert_eq!(bedrock_metrics.blocked_requests, 200);

        // ===== Phase 7: Verify Final State =====

        // List all backends
        let backends = system.list_backends(&token, &org_id).unwrap();
        assert_eq!(backends.len(), 2);

        // All backends should be protected
        assert!(backends.iter().all(|b| b.status == "protected"));

        // Gateway should have synced everything
        assert!(system.gateway_state.synced_backends.contains(&mc_java_id));
        assert!(system.gateway_state.synced_backends.contains(&mc_bedrock_id));
        assert_eq!(system.gateway_state.synced_rules.len(), 3);

        println!("E2E test completed successfully!");
        println!("- Users: 1");
        println!("- Organizations: 1");
        println!("- Backends: 2");
        println!("- Filter Rules: 3");
        println!("- API Keys: 1");
        println!("- Total Requests Processed: 15000");
        println!("- Total Requests Blocked: 700");
    }
}
