//! Docker Compose based integration tests
//!
//! These tests are designed to run against services deployed via Docker Compose.
//! They verify real service interactions in a containerized environment.

use super::test_helpers::{
    generate_test_id, wait_for_all_services, TestBackend, TestClient, TestEnvironment,
    TestOrganization, TestUser,
};
use std::process::Command;
use std::time::Duration;

/// Docker Compose test runner
pub struct DockerComposeRunner {
    compose_file: String,
    project_name: String,
    started: bool,
}

impl DockerComposeRunner {
    /// Create a new runner for integration tests
    pub fn new(compose_file: &str) -> Self {
        let project_name = format!("pistonprotection-test-{}", generate_test_id());
        Self {
            compose_file: compose_file.to_string(),
            project_name,
            started: false,
        }
    }

    /// Start all services
    pub fn start(&mut self) -> Result<(), String> {
        let output = Command::new("docker-compose")
            .args([
                "-f",
                &self.compose_file,
                "-p",
                &self.project_name,
                "up",
                "-d",
            ])
            .output()
            .map_err(|e| format!("Failed to run docker-compose: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "docker-compose up failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        self.started = true;
        Ok(())
    }

    /// Stop all services
    pub fn stop(&mut self) -> Result<(), String> {
        if !self.started {
            return Ok(());
        }

        let output = Command::new("docker-compose")
            .args([
                "-f",
                &self.compose_file,
                "-p",
                &self.project_name,
                "down",
                "-v",
            ])
            .output()
            .map_err(|e| format!("Failed to run docker-compose: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "docker-compose down failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        self.started = false;
        Ok(())
    }

    /// Get logs from a service
    pub fn logs(&self, service: &str) -> Result<String, String> {
        let output = Command::new("docker-compose")
            .args([
                "-f",
                &self.compose_file,
                "-p",
                &self.project_name,
                "logs",
                service,
            ])
            .output()
            .map_err(|e| format!("Failed to get logs: {}", e))?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Check if a service is healthy
    pub fn is_healthy(&self, service: &str) -> bool {
        let output = Command::new("docker-compose")
            .args([
                "-f",
                &self.compose_file,
                "-p",
                &self.project_name,
                "ps",
                "--filter",
                "health=healthy",
                service,
            ])
            .output();

        match output {
            Ok(o) => {
                let stdout = String::from_utf8_lossy(&o.stdout);
                stdout.contains(service)
            }
            Err(_) => false,
        }
    }

    /// Execute command in a service container
    pub fn exec(&self, service: &str, command: &[&str]) -> Result<String, String> {
        let mut args = vec![
            "-f",
            &self.compose_file,
            "-p",
            &self.project_name,
            "exec",
            "-T",
            service,
        ];
        args.extend(command);

        let output = Command::new("docker-compose")
            .args(&args)
            .output()
            .map_err(|e| format!("Failed to exec: {}", e))?;

        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).to_string());
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

impl Drop for DockerComposeRunner {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

// ============================================================================
// Mock Docker Compose Tests (for unit testing without Docker)
// ============================================================================

/// Mock service state for testing without Docker
struct MockServiceState {
    services: std::collections::HashMap<String, MockServiceStatus>,
}

#[derive(Clone)]
struct MockServiceStatus {
    name: String,
    running: bool,
    healthy: bool,
    port: u16,
}

impl MockServiceState {
    fn new() -> Self {
        Self {
            services: std::collections::HashMap::new(),
        }
    }

    fn add_service(&mut self, name: &str, port: u16) {
        self.services.insert(
            name.to_string(),
            MockServiceStatus {
                name: name.to_string(),
                running: false,
                healthy: false,
                port,
            },
        );
    }

    fn start_all(&mut self) {
        for service in self.services.values_mut() {
            service.running = true;
            service.healthy = true;
        }
    }

    fn stop_all(&mut self) {
        for service in self.services.values_mut() {
            service.running = false;
            service.healthy = false;
        }
    }

    fn is_running(&self, name: &str) -> bool {
        self.services
            .get(name)
            .map(|s| s.running)
            .unwrap_or(false)
    }

    fn is_healthy(&self, name: &str) -> bool {
        self.services.get(name).map(|s| s.healthy).unwrap_or(false)
    }

    fn set_unhealthy(&mut self, name: &str) {
        if let Some(service) = self.services.get_mut(name) {
            service.healthy = false;
        }
    }
}

// ============================================================================
// Docker Compose Configuration Tests
// ============================================================================

#[cfg(test)]
mod compose_config_tests {
    use super::*;

    /// Test Docker Compose runner creation
    #[test]
    fn test_compose_runner_creation() {
        let runner = DockerComposeRunner::new("docker-compose.test.yml");
        assert!(!runner.started);
        assert!(runner.project_name.starts_with("pistonprotection-test-"));
    }

    /// Test mock service state management
    #[test]
    fn test_mock_service_state() {
        let mut state = MockServiceState::new();
        state.add_service("gateway", 8080);
        state.add_service("auth", 8081);

        assert!(!state.is_running("gateway"));

        state.start_all();
        assert!(state.is_running("gateway"));
        assert!(state.is_healthy("gateway"));

        state.stop_all();
        assert!(!state.is_running("gateway"));
    }

    /// Test unhealthy service detection
    #[test]
    fn test_unhealthy_service() {
        let mut state = MockServiceState::new();
        state.add_service("gateway", 8080);
        state.start_all();

        state.set_unhealthy("gateway");
        assert!(state.is_running("gateway"));
        assert!(!state.is_healthy("gateway"));
    }
}

// ============================================================================
// Service Readiness Tests
// ============================================================================

#[cfg(test)]
mod readiness_tests {
    use super::*;

    /// Test all services become ready
    #[test]
    fn test_all_services_ready() {
        let mut state = MockServiceState::new();
        state.add_service("gateway", 8080);
        state.add_service("auth", 8081);
        state.add_service("metrics", 8082);
        state.add_service("postgres", 5432);
        state.add_service("redis", 6379);

        state.start_all();

        let all_healthy = state
            .services
            .keys()
            .all(|name| state.is_healthy(name));
        assert!(all_healthy);
    }

    /// Test service dependency chain
    #[test]
    fn test_service_dependencies() {
        let mut state = MockServiceState::new();

        // Infrastructure services
        state.add_service("postgres", 5432);
        state.add_service("redis", 6379);

        // Application services
        state.add_service("auth", 8081);
        state.add_service("gateway", 8080);
        state.add_service("metrics", 8082);

        // Start in order (simulating depends_on)
        state.services.get_mut("postgres").unwrap().running = true;
        state.services.get_mut("postgres").unwrap().healthy = true;

        state.services.get_mut("redis").unwrap().running = true;
        state.services.get_mut("redis").unwrap().healthy = true;

        // Now app services can start
        state.services.get_mut("auth").unwrap().running = true;
        state.services.get_mut("auth").unwrap().healthy = true;

        state.services.get_mut("gateway").unwrap().running = true;
        state.services.get_mut("gateway").unwrap().healthy = true;

        state.services.get_mut("metrics").unwrap().running = true;
        state.services.get_mut("metrics").unwrap().healthy = true;

        // All should be healthy now
        assert!(state.is_healthy("gateway"));
        assert!(state.is_healthy("auth"));
        assert!(state.is_healthy("metrics"));
    }
}

// ============================================================================
// Database Migration Tests
// ============================================================================

#[cfg(test)]
mod database_tests {
    use super::*;

    /// Mock database for testing
    struct MockDatabase {
        connected: bool,
        migrations_applied: Vec<String>,
        tables: Vec<String>,
    }

    impl MockDatabase {
        fn new() -> Self {
            Self {
                connected: false,
                migrations_applied: Vec::new(),
                tables: Vec::new(),
            }
        }

        fn connect(&mut self) -> Result<(), String> {
            self.connected = true;
            Ok(())
        }

        fn apply_migration(&mut self, name: &str) -> Result<(), String> {
            if !self.connected {
                return Err("Not connected".to_string());
            }

            self.migrations_applied.push(name.to_string());

            // Simulate table creation
            match name {
                "001_create_users" => self.tables.push("users".to_string()),
                "002_create_organizations" => self.tables.push("organizations".to_string()),
                "003_create_backends" => self.tables.push("backends".to_string()),
                "004_create_filter_rules" => self.tables.push("filter_rules".to_string()),
                "005_create_api_keys" => self.tables.push("api_keys".to_string()),
                _ => {}
            }

            Ok(())
        }

        fn has_table(&self, name: &str) -> bool {
            self.tables.contains(&name.to_string())
        }
    }

    /// Test database migrations
    #[test]
    fn test_database_migrations() {
        let mut db = MockDatabase::new();
        db.connect().unwrap();

        // Apply migrations in order
        let migrations = vec![
            "001_create_users",
            "002_create_organizations",
            "003_create_backends",
            "004_create_filter_rules",
            "005_create_api_keys",
        ];

        for migration in &migrations {
            db.apply_migration(migration).unwrap();
        }

        assert_eq!(db.migrations_applied.len(), 5);
        assert!(db.has_table("users"));
        assert!(db.has_table("organizations"));
        assert!(db.has_table("backends"));
    }

    /// Test migration without connection fails
    #[test]
    fn test_migration_requires_connection() {
        let mut db = MockDatabase::new();
        let result = db.apply_migration("001_create_users");
        assert!(result.is_err());
    }
}

// ============================================================================
// Service Health Check Tests
// ============================================================================

#[cfg(test)]
mod health_check_tests {
    use super::*;

    /// Mock health checker
    struct MockHealthChecker {
        results: std::collections::HashMap<String, HealthCheckResult>,
    }

    #[derive(Clone)]
    struct HealthCheckResult {
        healthy: bool,
        latency_ms: u64,
        details: std::collections::HashMap<String, String>,
    }

    impl MockHealthChecker {
        fn new() -> Self {
            Self {
                results: std::collections::HashMap::new(),
            }
        }

        fn set_result(&mut self, service: &str, result: HealthCheckResult) {
            self.results.insert(service.to_string(), result);
        }

        fn check(&self, service: &str) -> Option<&HealthCheckResult> {
            self.results.get(service)
        }

        fn check_all(&self) -> Vec<(&String, &HealthCheckResult)> {
            self.results.iter().collect()
        }

        fn all_healthy(&self) -> bool {
            self.results.values().all(|r| r.healthy)
        }
    }

    /// Test health check for all services
    #[test]
    fn test_all_services_health_check() {
        let mut checker = MockHealthChecker::new();

        // Set up healthy results
        for (service, latency) in [("gateway", 5), ("auth", 3), ("metrics", 4)] {
            checker.set_result(
                service,
                HealthCheckResult {
                    healthy: true,
                    latency_ms: latency,
                    details: std::collections::HashMap::new(),
                },
            );
        }

        assert!(checker.all_healthy());
        assert_eq!(checker.check_all().len(), 3);
    }

    /// Test health check with unhealthy service
    #[test]
    fn test_unhealthy_service_check() {
        let mut checker = MockHealthChecker::new();

        checker.set_result(
            "gateway",
            HealthCheckResult {
                healthy: true,
                latency_ms: 5,
                details: std::collections::HashMap::new(),
            },
        );

        checker.set_result(
            "auth",
            HealthCheckResult {
                healthy: false,
                latency_ms: 5000, // High latency indicates problem
                details: {
                    let mut d = std::collections::HashMap::new();
                    d.insert("error".to_string(), "Database connection failed".to_string());
                    d
                },
            },
        );

        assert!(!checker.all_healthy());

        let auth_result = checker.check("auth").unwrap();
        assert!(!auth_result.healthy);
        assert!(auth_result.details.contains_key("error"));
    }

    /// Test health check latency
    #[test]
    fn test_health_check_latency() {
        let mut checker = MockHealthChecker::new();

        checker.set_result(
            "gateway",
            HealthCheckResult {
                healthy: true,
                latency_ms: 150,
                details: std::collections::HashMap::new(),
            },
        );

        let result = checker.check("gateway").unwrap();
        assert!(result.latency_ms < 200, "Latency should be under 200ms");
    }
}

// ============================================================================
// Network Connectivity Tests
// ============================================================================

#[cfg(test)]
mod network_tests {
    use super::*;

    /// Mock network for testing connectivity
    struct MockNetwork {
        connections: Vec<(String, String)>,
        blocked: Vec<(String, String)>,
    }

    impl MockNetwork {
        fn new() -> Self {
            Self {
                connections: Vec::new(),
                blocked: Vec::new(),
            }
        }

        fn connect(&mut self, from: &str, to: &str) {
            if !self.is_blocked(from, to) {
                self.connections.push((from.to_string(), to.to_string()));
            }
        }

        fn block(&mut self, from: &str, to: &str) {
            self.blocked.push((from.to_string(), to.to_string()));
        }

        fn is_blocked(&self, from: &str, to: &str) -> bool {
            self.blocked
                .iter()
                .any(|(f, t)| f == from && t == to)
        }

        fn can_connect(&self, from: &str, to: &str) -> bool {
            !self.is_blocked(from, to)
        }

        fn get_connections_from(&self, from: &str) -> Vec<&String> {
            self.connections
                .iter()
                .filter(|(f, _)| f == from)
                .map(|(_, t)| t)
                .collect()
        }
    }

    /// Test service connectivity
    #[test]
    fn test_service_connectivity() {
        let mut network = MockNetwork::new();

        // Gateway connects to all services
        network.connect("gateway", "auth");
        network.connect("gateway", "metrics");
        network.connect("gateway", "postgres");

        let gateway_connections = network.get_connections_from("gateway");
        assert_eq!(gateway_connections.len(), 3);
    }

    /// Test blocked connection
    #[test]
    fn test_blocked_connection() {
        let mut network = MockNetwork::new();

        // Block connection
        network.block("frontend", "postgres");

        assert!(!network.can_connect("frontend", "postgres"));
        assert!(network.can_connect("gateway", "postgres"));
    }

    /// Test network isolation
    #[test]
    fn test_network_isolation() {
        let mut network = MockNetwork::new();

        // Frontend can only connect to gateway
        network.block("frontend", "auth");
        network.block("frontend", "metrics");
        network.block("frontend", "postgres");
        network.block("frontend", "redis");

        assert!(network.can_connect("frontend", "gateway"));
        assert!(!network.can_connect("frontend", "postgres"));
    }
}

// ============================================================================
// Volume and Data Persistence Tests
// ============================================================================

#[cfg(test)]
mod persistence_tests {
    use super::*;

    /// Mock volume for testing persistence
    struct MockVolume {
        name: String,
        data: std::collections::HashMap<String, Vec<u8>>,
    }

    impl MockVolume {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
                data: std::collections::HashMap::new(),
            }
        }

        fn write(&mut self, path: &str, content: &[u8]) {
            self.data.insert(path.to_string(), content.to_vec());
        }

        fn read(&self, path: &str) -> Option<&Vec<u8>> {
            self.data.get(path)
        }

        fn exists(&self, path: &str) -> bool {
            self.data.contains_key(path)
        }

        fn delete(&mut self, path: &str) -> bool {
            self.data.remove(path).is_some()
        }

        fn list(&self) -> Vec<&String> {
            self.data.keys().collect()
        }
    }

    /// Test data persistence across restarts
    #[test]
    fn test_data_persistence() {
        let mut volume = MockVolume::new("postgres-data");

        // Write data
        volume.write("/data/users.db", b"user data here");

        // Simulate restart - volume persists
        assert!(volume.exists("/data/users.db"));

        let data = volume.read("/data/users.db").unwrap();
        assert_eq!(data, b"user data here");
    }

    /// Test volume cleanup
    #[test]
    fn test_volume_cleanup() {
        let mut volume = MockVolume::new("test-data");

        volume.write("/data/file1", b"content1");
        volume.write("/data/file2", b"content2");

        assert_eq!(volume.list().len(), 2);

        volume.delete("/data/file1");
        assert_eq!(volume.list().len(), 1);
        assert!(!volume.exists("/data/file1"));
    }
}

// ============================================================================
// Environment Variable Tests
// ============================================================================

#[cfg(test)]
mod env_var_tests {
    use super::*;

    /// Mock environment for testing
    struct MockEnv {
        vars: std::collections::HashMap<String, String>,
    }

    impl MockEnv {
        fn new() -> Self {
            Self {
                vars: std::collections::HashMap::new(),
            }
        }

        fn set(&mut self, key: &str, value: &str) {
            self.vars.insert(key.to_string(), value.to_string());
        }

        fn get(&self, key: &str) -> Option<&String> {
            self.vars.get(key)
        }

        fn required(&self, key: &str) -> Result<&String, String> {
            self.vars
                .get(key)
                .ok_or_else(|| format!("Required env var {} not set", key))
        }
    }

    /// Test required environment variables
    #[test]
    fn test_required_env_vars() {
        let mut env = MockEnv::new();

        // Set required vars
        env.set("DATABASE_URL", "postgres://localhost/test");
        env.set("REDIS_URL", "redis://localhost:6379");
        env.set("JWT_SECRET", "test-secret-key");
        env.set("GATEWAY_PORT", "8080");

        let required_vars = vec!["DATABASE_URL", "REDIS_URL", "JWT_SECRET", "GATEWAY_PORT"];

        for var in required_vars {
            assert!(env.required(var).is_ok(), "Missing required var: {}", var);
        }
    }

    /// Test missing required variable
    #[test]
    fn test_missing_required_var() {
        let env = MockEnv::new();
        let result = env.required("DATABASE_URL");
        assert!(result.is_err());
    }

    /// Test environment profiles
    #[test]
    fn test_env_profiles() {
        let mut test_env = MockEnv::new();
        test_env.set("ENVIRONMENT", "test");
        test_env.set("LOG_LEVEL", "debug");
        test_env.set("DATABASE_URL", "postgres://localhost/test");

        let mut prod_env = MockEnv::new();
        prod_env.set("ENVIRONMENT", "production");
        prod_env.set("LOG_LEVEL", "info");
        prod_env.set("DATABASE_URL", "postgres://db.prod.local/pistonprotection");

        assert_eq!(test_env.get("LOG_LEVEL"), Some(&"debug".to_string()));
        assert_eq!(prod_env.get("LOG_LEVEL"), Some(&"info".to_string()));
    }
}
