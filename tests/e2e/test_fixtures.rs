//! Test fixtures for e2e tests

use std::collections::HashMap;
use std::process::Command;
use std::time::Duration;

/// Kubernetes cluster configuration for testing
pub struct K8sTestCluster {
    pub cluster_type: ClusterType,
    pub kubeconfig_path: String,
    pub namespace: String,
    pub resources_deployed: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ClusterType {
    Minikube,
    Kind,
    K3s,
    Mock,
}

impl K8sTestCluster {
    /// Create a new test cluster configuration
    pub fn new(cluster_type: ClusterType, namespace: &str) -> Self {
        let kubeconfig_path = std::env::var("KUBECONFIG")
            .unwrap_or_else(|_| format!("{}/.kube/config", std::env::var("HOME").unwrap()));

        Self {
            cluster_type,
            kubeconfig_path,
            namespace: namespace.to_string(),
            resources_deployed: Vec::new(),
        }
    }

    /// Create a mock cluster for unit testing
    pub fn mock(namespace: &str) -> Self {
        Self {
            cluster_type: ClusterType::Mock,
            kubeconfig_path: String::new(),
            namespace: namespace.to_string(),
            resources_deployed: Vec::new(),
        }
    }

    /// Check if cluster is available
    pub fn is_available(&self) -> bool {
        if self.cluster_type == ClusterType::Mock {
            return true;
        }

        let output = Command::new("kubectl")
            .args(["cluster-info"])
            .env("KUBECONFIG", &self.kubeconfig_path)
            .output();

        match output {
            Ok(o) => o.status.success(),
            Err(_) => false,
        }
    }

    /// Create test namespace
    pub fn create_namespace(&self) -> Result<(), String> {
        if self.cluster_type == ClusterType::Mock {
            return Ok(());
        }

        let output = Command::new("kubectl")
            .args(["create", "namespace", &self.namespace])
            .env("KUBECONFIG", &self.kubeconfig_path)
            .output()
            .map_err(|e| e.to_string())?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if !stderr.contains("already exists") {
                return Err(stderr.to_string());
            }
        }

        Ok(())
    }

    /// Delete test namespace
    pub fn delete_namespace(&self) -> Result<(), String> {
        if self.cluster_type == ClusterType::Mock {
            return Ok(());
        }

        let output = Command::new("kubectl")
            .args(["delete", "namespace", &self.namespace, "--ignore-not-found"])
            .env("KUBECONFIG", &self.kubeconfig_path)
            .output()
            .map_err(|e| e.to_string())?;

        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).to_string());
        }

        Ok(())
    }

    /// Apply Kubernetes manifest
    pub fn apply_manifest(&mut self, manifest: &str) -> Result<(), String> {
        if self.cluster_type == ClusterType::Mock {
            self.resources_deployed.push(manifest.to_string());
            return Ok(());
        }

        let output = Command::new("kubectl")
            .args(["apply", "-f", manifest, "-n", &self.namespace])
            .env("KUBECONFIG", &self.kubeconfig_path)
            .output()
            .map_err(|e| e.to_string())?;

        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).to_string());
        }

        self.resources_deployed.push(manifest.to_string());
        Ok(())
    }

    /// Delete Kubernetes manifest
    pub fn delete_manifest(&mut self, manifest: &str) -> Result<(), String> {
        if self.cluster_type == ClusterType::Mock {
            self.resources_deployed.retain(|r| r != manifest);
            return Ok(());
        }

        let output = Command::new("kubectl")
            .args([
                "delete",
                "-f",
                manifest,
                "-n",
                &self.namespace,
                "--ignore-not-found",
            ])
            .env("KUBECONFIG", &self.kubeconfig_path)
            .output()
            .map_err(|e| e.to_string())?;

        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).to_string());
        }

        self.resources_deployed.retain(|r| r != manifest);
        Ok(())
    }

    /// Wait for deployment to be ready
    pub fn wait_for_deployment(&self, name: &str, timeout: Duration) -> Result<(), String> {
        if self.cluster_type == ClusterType::Mock {
            return Ok(());
        }

        let output = Command::new("kubectl")
            .args([
                "rollout",
                "status",
                "deployment",
                name,
                "-n",
                &self.namespace,
                "--timeout",
                &format!("{}s", timeout.as_secs()),
            ])
            .env("KUBECONFIG", &self.kubeconfig_path)
            .output()
            .map_err(|e| e.to_string())?;

        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).to_string());
        }

        Ok(())
    }

    /// Get pod logs
    pub fn get_logs(&self, selector: &str) -> Result<String, String> {
        if self.cluster_type == ClusterType::Mock {
            return Ok("Mock logs".to_string());
        }

        let output = Command::new("kubectl")
            .args([
                "logs",
                "-l",
                selector,
                "-n",
                &self.namespace,
                "--tail",
                "100",
            ])
            .env("KUBECONFIG", &self.kubeconfig_path)
            .output()
            .map_err(|e| e.to_string())?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Execute command in pod
    pub fn exec_in_pod(
        &self,
        selector: &str,
        command: &[&str],
    ) -> Result<String, String> {
        if self.cluster_type == ClusterType::Mock {
            return Ok("Mock exec output".to_string());
        }

        // Get pod name
        let pod_output = Command::new("kubectl")
            .args([
                "get",
                "pods",
                "-l",
                selector,
                "-n",
                &self.namespace,
                "-o",
                "jsonpath={.items[0].metadata.name}",
            ])
            .env("KUBECONFIG", &self.kubeconfig_path)
            .output()
            .map_err(|e| e.to_string())?;

        let pod_name = String::from_utf8_lossy(&pod_output.stdout).to_string();
        if pod_name.is_empty() {
            return Err("No pod found".to_string());
        }

        let mut args = vec!["exec", &pod_name, "-n", &self.namespace, "--"];
        args.extend(command);

        let output = Command::new("kubectl")
            .args(&args)
            .env("KUBECONFIG", &self.kubeconfig_path)
            .output()
            .map_err(|e| e.to_string())?;

        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).to_string());
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Port forward to a service
    pub fn port_forward(&self, service: &str, local_port: u16, remote_port: u16) -> Result<std::process::Child, String> {
        if self.cluster_type == ClusterType::Mock {
            return Err("Cannot port-forward in mock mode".to_string());
        }

        let child = Command::new("kubectl")
            .args([
                "port-forward",
                &format!("svc/{}", service),
                &format!("{}:{}", local_port, remote_port),
                "-n",
                &self.namespace,
            ])
            .env("KUBECONFIG", &self.kubeconfig_path)
            .spawn()
            .map_err(|e| e.to_string())?;

        Ok(child)
    }
}

/// Test DDoSProtection CRD instance
#[derive(Clone, Debug)]
pub struct TestDDoSProtection {
    pub name: String,
    pub namespace: String,
    pub backends: Vec<TestBackendSpec>,
    pub protection_level: i32,
    pub replicas: i32,
}

#[derive(Clone, Debug)]
pub struct TestBackendSpec {
    pub name: String,
    pub address: String,
    pub protocol: String,
    pub port: u16,
}

impl TestDDoSProtection {
    pub fn new(name: &str, namespace: &str) -> Self {
        Self {
            name: name.to_string(),
            namespace: namespace.to_string(),
            backends: Vec::new(),
            protection_level: 3,
            replicas: 2,
        }
    }

    pub fn with_backend(mut self, name: &str, address: &str, protocol: &str, port: u16) -> Self {
        self.backends.push(TestBackendSpec {
            name: name.to_string(),
            address: address.to_string(),
            protocol: protocol.to_string(),
            port,
        });
        self
    }

    pub fn with_protection_level(mut self, level: i32) -> Self {
        self.protection_level = level;
        self
    }

    pub fn with_replicas(mut self, replicas: i32) -> Self {
        self.replicas = replicas;
        self
    }

    /// Generate YAML manifest
    pub fn to_yaml(&self) -> String {
        let backends_yaml: Vec<String> = self
            .backends
            .iter()
            .map(|b| {
                format!(
                    r#"    - name: {}
      address: "{}:{}"
      protocol: {}"#,
                    b.name, b.address, b.port, b.protocol
                )
            })
            .collect();

        format!(
            r#"apiVersion: pistonprotection.io/v1alpha1
kind: DDoSProtection
metadata:
  name: {}
  namespace: {}
spec:
  backends:
{}
  protectionLevel: {}
  replicas: {}"#,
            self.name,
            self.namespace,
            backends_yaml.join("\n"),
            self.protection_level,
            self.replicas
        )
    }
}

/// Test FilterRule CRD instance
#[derive(Clone, Debug)]
pub struct TestFilterRule {
    pub name: String,
    pub namespace: String,
    pub rule_type: String,
    pub action: String,
    pub priority: i32,
    pub config: HashMap<String, Vec<String>>,
}

impl TestFilterRule {
    pub fn new(name: &str, namespace: &str) -> Self {
        Self {
            name: name.to_string(),
            namespace: namespace.to_string(),
            rule_type: "ip-blocklist".to_string(),
            action: "drop".to_string(),
            priority: 50,
            config: HashMap::new(),
        }
    }

    pub fn with_type(mut self, rule_type: &str) -> Self {
        self.rule_type = rule_type.to_string();
        self
    }

    pub fn with_action(mut self, action: &str) -> Self {
        self.action = action.to_string();
        self
    }

    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_ip_ranges(mut self, ranges: Vec<&str>) -> Self {
        self.config.insert(
            "ipRanges".to_string(),
            ranges.iter().map(|s| s.to_string()).collect(),
        );
        self
    }

    pub fn with_countries(mut self, countries: Vec<&str>) -> Self {
        self.config.insert(
            "countries".to_string(),
            countries.iter().map(|s| s.to_string()).collect(),
        );
        self
    }

    /// Generate YAML manifest
    pub fn to_yaml(&self) -> String {
        let mut config_yaml = String::new();
        for (key, values) in &self.config {
            config_yaml.push_str(&format!("    {}:\n", key));
            for value in values {
                config_yaml.push_str(&format!("      - \"{}\"\n", value));
            }
        }

        format!(
            r#"apiVersion: pistonprotection.io/v1alpha1
kind: FilterRule
metadata:
  name: {}
  namespace: {}
spec:
  name: "Test Rule {}"
  ruleType: {}
  action: {}
  priority: {}
  config:
{}"#,
            self.name,
            self.namespace,
            self.name,
            self.rule_type,
            self.action,
            self.priority,
            config_yaml
        )
    }
}

/// Generate unique test ID
pub fn generate_test_id() -> String {
    uuid::Uuid::new_v4().to_string()[..8].to_string()
}

/// Wait for condition with timeout
pub async fn wait_for_condition<F, Fut>(
    condition: F,
    timeout: Duration,
    interval: Duration,
) -> Result<(), String>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = bool>,
{
    let start = std::time::Instant::now();

    while start.elapsed() < timeout {
        if condition().await {
            return Ok(());
        }
        tokio::time::sleep(interval).await;
    }

    Err("Timeout waiting for condition".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_k8s_cluster_mock() {
        let cluster = K8sTestCluster::mock("test-namespace");
        assert!(cluster.is_available());
        assert_eq!(cluster.cluster_type, ClusterType::Mock);
    }

    #[test]
    fn test_mock_cluster_operations() {
        let mut cluster = K8sTestCluster::mock("test-namespace");

        // Create namespace
        assert!(cluster.create_namespace().is_ok());

        // Apply manifest
        assert!(cluster.apply_manifest("test.yaml").is_ok());
        assert!(cluster.resources_deployed.contains(&"test.yaml".to_string()));

        // Delete manifest
        assert!(cluster.delete_manifest("test.yaml").is_ok());
        assert!(!cluster.resources_deployed.contains(&"test.yaml".to_string()));
    }

    #[test]
    fn test_ddos_protection_yaml() {
        let protection = TestDDoSProtection::new("test", "default")
            .with_backend("mc-server", "10.0.0.1", "minecraft-java", 25565)
            .with_protection_level(4)
            .with_replicas(3);

        let yaml = protection.to_yaml();
        assert!(yaml.contains("kind: DDoSProtection"));
        assert!(yaml.contains("name: test"));
        assert!(yaml.contains("protectionLevel: 4"));
        assert!(yaml.contains("replicas: 3"));
    }

    #[test]
    fn test_filter_rule_yaml() {
        let rule = TestFilterRule::new("block-bad-ips", "default")
            .with_type("ip-blocklist")
            .with_action("drop")
            .with_priority(100)
            .with_ip_ranges(vec!["192.168.1.0/24", "10.0.0.0/8"]);

        let yaml = rule.to_yaml();
        assert!(yaml.contains("kind: FilterRule"));
        assert!(yaml.contains("ruleType: ip-blocklist"));
        assert!(yaml.contains("priority: 100"));
    }
}
