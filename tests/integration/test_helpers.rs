//! Test helpers for integration tests

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

/// Configuration for test environment
pub struct TestEnvironment {
    pub gateway_url: String,
    pub auth_url: String,
    pub metrics_url: String,
    pub database_url: String,
    pub redis_url: String,
    pub services: HashMap<String, ServiceInfo>,
}

/// Information about a running service
#[derive(Clone, Debug)]
pub struct ServiceInfo {
    pub name: String,
    pub url: String,
    pub health_endpoint: String,
    pub ready: bool,
}

impl Default for TestEnvironment {
    fn default() -> Self {
        Self {
            gateway_url: "http://localhost:8080".to_string(),
            auth_url: "http://localhost:8081".to_string(),
            metrics_url: "http://localhost:8082".to_string(),
            database_url: "postgres://test:test@localhost:5432/pistonprotection_test".to_string(),
            redis_url: "redis://localhost:6379".to_string(),
            services: HashMap::new(),
        }
    }
}

impl TestEnvironment {
    /// Create from environment variables
    pub fn from_env() -> Self {
        Self {
            gateway_url: std::env::var("GATEWAY_URL")
                .unwrap_or_else(|_| "http://localhost:8080".to_string()),
            auth_url: std::env::var("AUTH_URL")
                .unwrap_or_else(|_| "http://localhost:8081".to_string()),
            metrics_url: std::env::var("METRICS_URL")
                .unwrap_or_else(|_| "http://localhost:8082".to_string()),
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://test:test@localhost:5432/pistonprotection_test".to_string()),
            redis_url: std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            services: HashMap::new(),
        }
    }

    /// Add a service to the environment
    pub fn add_service(&mut self, name: &str, url: &str, health_endpoint: &str) {
        self.services.insert(
            name.to_string(),
            ServiceInfo {
                name: name.to_string(),
                url: url.to_string(),
                health_endpoint: health_endpoint.to_string(),
                ready: false,
            },
        );
    }
}

/// HTTP client for integration tests
pub struct TestClient {
    client: reqwest::Client,
    base_url: String,
    auth_token: Option<String>,
    api_key: Option<String>,
}

impl TestClient {
    /// Create a new test client
    pub fn new(base_url: &str) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            base_url: base_url.to_string(),
            auth_token: None,
            api_key: None,
        }
    }

    /// Set auth token
    pub fn with_auth_token(mut self, token: &str) -> Self {
        self.auth_token = Some(token.to_string());
        self
    }

    /// Set API key
    pub fn with_api_key(mut self, key: &str) -> Self {
        self.api_key = Some(key.to_string());
        self
    }

    /// Build request with auth headers
    fn build_request(&self, method: reqwest::Method, path: &str) -> reqwest::RequestBuilder {
        let url = format!("{}{}", self.base_url, path);
        let mut request = self.client.request(method, &url);

        if let Some(ref token) = self.auth_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        if let Some(ref key) = self.api_key {
            request = request.header("X-API-Key", key);
        }

        request
    }

    /// GET request
    pub async fn get(&self, path: &str) -> Result<reqwest::Response, reqwest::Error> {
        self.build_request(reqwest::Method::GET, path).send().await
    }

    /// POST request with JSON body
    pub async fn post<T: serde::Serialize>(
        &self,
        path: &str,
        body: &T,
    ) -> Result<reqwest::Response, reqwest::Error> {
        self.build_request(reqwest::Method::POST, path)
            .json(body)
            .send()
            .await
    }

    /// PUT request with JSON body
    pub async fn put<T: serde::Serialize>(
        &self,
        path: &str,
        body: &T,
    ) -> Result<reqwest::Response, reqwest::Error> {
        self.build_request(reqwest::Method::PUT, path)
            .json(body)
            .send()
            .await
    }

    /// DELETE request
    pub async fn delete(&self, path: &str) -> Result<reqwest::Response, reqwest::Error> {
        self.build_request(reqwest::Method::DELETE, path)
            .send()
            .await
    }

    /// Check service health
    pub async fn health_check(&self) -> Result<bool, reqwest::Error> {
        let response = self.get("/health").await?;
        Ok(response.status().is_success())
    }
}

/// Test user for authentication flows
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TestUser {
    pub email: String,
    pub password: String,
    pub username: String,
}

impl TestUser {
    pub fn new(email: &str, password: &str, username: &str) -> Self {
        Self {
            email: email.to_string(),
            password: password.to_string(),
            username: username.to_string(),
        }
    }

    pub fn random() -> Self {
        let id = uuid::Uuid::new_v4().to_string()[..8].to_string();
        Self {
            email: format!("test-{}@example.com", id),
            password: "TestPassword123!".to_string(),
            username: format!("testuser_{}", id),
        }
    }
}

/// Test organization
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TestOrganization {
    pub name: String,
    pub display_name: String,
}

impl TestOrganization {
    pub fn new(name: &str, display_name: &str) -> Self {
        Self {
            name: name.to_string(),
            display_name: display_name.to_string(),
        }
    }

    pub fn random() -> Self {
        let id = uuid::Uuid::new_v4().to_string()[..8].to_string();
        Self {
            name: format!("test-org-{}", id),
            display_name: format!("Test Organization {}", id),
        }
    }
}

/// Test backend configuration
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TestBackend {
    pub name: String,
    pub address: String,
    pub protocol: String,
    pub port: u16,
}

impl TestBackend {
    pub fn new(name: &str, address: &str, protocol: &str, port: u16) -> Self {
        Self {
            name: name.to_string(),
            address: address.to_string(),
            protocol: protocol.to_string(),
            port,
        }
    }

    pub fn minecraft_java(name: &str, address: &str) -> Self {
        Self {
            name: name.to_string(),
            address: address.to_string(),
            protocol: "minecraft-java".to_string(),
            port: 25565,
        }
    }
}

/// Wait for a service to become healthy
pub async fn wait_for_service(url: &str, timeout: Duration) -> Result<(), String> {
    let client = reqwest::Client::new();
    let health_url = format!("{}/health", url);
    let start = std::time::Instant::now();

    while start.elapsed() < timeout {
        match client.get(&health_url).send().await {
            Ok(response) if response.status().is_success() => {
                return Ok(());
            }
            _ => {
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        }
    }

    Err(format!("Service at {} did not become healthy within {:?}", url, timeout))
}

/// Wait for all services to become healthy
pub async fn wait_for_all_services(env: &TestEnvironment, timeout: Duration) -> Result<(), String> {
    let services = vec![
        ("gateway", &env.gateway_url),
        ("auth", &env.auth_url),
        ("metrics", &env.metrics_url),
    ];

    for (name, url) in services {
        wait_for_service(url, timeout)
            .await
            .map_err(|e| format!("{} service: {}", name, e))?;
    }

    Ok(())
}

/// Clean up test data
pub async fn cleanup_test_data(env: &TestEnvironment) -> Result<(), String> {
    // In a real implementation, this would connect to the database
    // and clean up test data
    Ok(())
}

/// Generate unique test ID
pub fn generate_test_id() -> String {
    uuid::Uuid::new_v4().to_string()[..8].to_string()
}

/// Assert JSON response contains expected fields
pub fn assert_json_contains(json: &serde_json::Value, path: &str, expected: &str) {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = json;

    for part in &parts {
        current = &current[*part];
    }

    assert_eq!(
        current.as_str().unwrap_or(""),
        expected,
        "Expected {} at path {} but got {:?}",
        expected,
        path,
        current
    );
}

/// Mock gRPC server for testing
pub struct MockGrpcServer {
    pub address: SocketAddr,
    pub requests: Arc<RwLock<Vec<MockGrpcRequest>>>,
}

#[derive(Clone, Debug)]
pub struct MockGrpcRequest {
    pub service: String,
    pub method: String,
    pub payload: Vec<u8>,
}

impl MockGrpcServer {
    pub fn new(address: SocketAddr) -> Self {
        Self {
            address,
            requests: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn get_requests(&self) -> Vec<MockGrpcRequest> {
        self.requests.read().await.clone()
    }

    pub async fn clear_requests(&self) {
        self.requests.write().await.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test_environment_default() {
        let env = TestEnvironment::default();
        assert_eq!(env.gateway_url, "http://localhost:8080");
        assert_eq!(env.auth_url, "http://localhost:8081");
    }

    #[test]
    fn test_test_user_random() {
        let user1 = TestUser::random();
        let user2 = TestUser::random();
        assert_ne!(user1.email, user2.email);
    }

    #[test]
    fn test_test_organization_random() {
        let org1 = TestOrganization::random();
        let org2 = TestOrganization::random();
        assert_ne!(org1.name, org2.name);
    }

    #[test]
    fn test_test_backend_minecraft() {
        let backend = TestBackend::minecraft_java("mc-server", "10.0.0.1");
        assert_eq!(backend.protocol, "minecraft-java");
        assert_eq!(backend.port, 25565);
    }
}
