# Installation Guide

This guide covers the installation of PistonProtection in various environments, from quick development setups to production-grade deployments.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Quick Start](#quick-start)
- [Production Deployment](#production-deployment)
- [Cloud Provider Guides](#cloud-provider-guides)
  - [AWS EKS](#aws-eks)
  - [Google GKE](#google-gke)
  - [Azure AKS](#azure-aks)
- [Self-Hosted Kubernetes](#self-hosted-kubernetes)
  - [k0s Setup](#k0s-setup)
  - [k3s Setup](#k3s-setup)
- [Verification](#verification)
- [Upgrading](#upgrading)
- [Uninstallation](#uninstallation)
- [Troubleshooting](#troubleshooting)

---

## Prerequisites

Before installing PistonProtection, ensure your environment meets the following requirements.

### Kubernetes Cluster Requirements

| Component | Minimum Version | Recommended Version |
|-----------|-----------------|---------------------|
| Kubernetes | 1.27+ | 1.29+ |
| Cilium CNI | 1.14+ | 1.16+ |
| Helm | 3.12+ | 3.14+ |
| kubectl | 1.27+ | 1.29+ |

### Worker Node Requirements

Worker nodes that run XDP/eBPF filters have specific requirements:

| Requirement | Specification |
|-------------|---------------|
| Linux Kernel | 5.15+ with BTF (BPF Type Format) support |
| Network Interface | XDP-compatible drivers (Intel i40e/ice, Mellanox mlx5, etc.) |
| Capabilities | CAP_BPF, CAP_NET_ADMIN, CAP_SYS_ADMIN |
| Filesystem | BPF filesystem mounted at /sys/fs/bpf |

### Hardware Recommendations

| Component | Development | Production |
|-----------|-------------|------------|
| Control Plane CPU | 2 cores | 4+ cores |
| Control Plane RAM | 4 GB | 8+ GB |
| Worker CPU | 2 cores | 8+ cores |
| Worker RAM | 2 GB | 8+ GB |
| Worker Network | 1 Gbps | 10+ Gbps |
| Storage | 20 GB SSD | 100+ GB NVMe |

### Verify Prerequisites

```bash
# Check Kubernetes version
kubectl version --short

# Check Helm version
helm version --short

# Verify kernel version on nodes (must be 5.15+)
kubectl get nodes -o wide

# Check for BTF support (run on each worker node)
ls /sys/kernel/btf/vmlinux

# Verify BPF filesystem
mount | grep bpf

# Check available XDP modes on network interface
ip link show eth0  # Look for xdp capability
```

---

## Quick Start

For development and testing environments.

### Step 1: Install Cilium CNI

PistonProtection requires Cilium as the CNI with specific configuration:

```bash
# Install Cilium CLI
CILIUM_CLI_VERSION=$(curl -s https://raw.githubusercontent.com/cilium/cilium-cli/main/stable.txt)
curl -L --fail --remote-name-all \
    https://github.com/cilium/cilium-cli/releases/download/${CILIUM_CLI_VERSION}/cilium-linux-amd64.tar.gz
sudo tar xzvfC cilium-linux-amd64.tar.gz /usr/local/bin
rm cilium-linux-amd64.tar.gz

# Get API server IP (adjust for your cluster)
API_SERVER_IP=$(kubectl get endpoints kubernetes -o jsonpath='{.subsets[0].addresses[0].ip}')

# Install Cilium with required settings
cilium install --version "1.16.0" \
    --set kubeProxyReplacement=true \
    --set k8sServiceHost="${API_SERVER_IP}" \
    --set k8sServicePort=6443 \
    --set hubble.enabled=true \
    --set hubble.relay.enabled=true \
    --set hubble.ui.enabled=true \
    --set l2announcements.enabled=true \
    --set encryption.enabled=true \
    --set encryption.type=wireguard \
    --set encryption.nodeEncryption=true

# Wait for Cilium to be ready
cilium status --wait
```

### Step 2: Add Helm Repository

```bash
helm repo add pistonprotection https://charts.pistonprotection.io
helm repo update
```

### Step 3: Create Namespace

```bash
kubectl create namespace pistonprotection
```

### Step 4: Install PistonProtection

```bash
# Install with default configuration
helm install pistonprotection pistonprotection/pistonprotection \
    --namespace pistonprotection \
    --wait --timeout 10m

# Monitor deployment progress
kubectl get pods -n pistonprotection -w
```

### Step 5: Access the Dashboard

```bash
# Port-forward the frontend service
kubectl port-forward -n pistonprotection svc/pistonprotection-frontend 3000:3000 &

# Open http://localhost:3000 in your browser
echo "Dashboard available at: http://localhost:3000"
```

---

## Production Deployment

For production environments, follow these comprehensive steps.

### Step 1: Create Production Values File

Create `values-production.yaml`:

```yaml
# =============================================================================
# PistonProtection Production Configuration
# =============================================================================

# Global settings
global:
  imagePullSecrets:
    - name: registry-credentials
  storageClass: "fast-ssd"

# -----------------------------------------------------------------------------
# Gateway Service - API Gateway and Proxy
# -----------------------------------------------------------------------------
gateway:
  enabled: true
  replicaCount: 3
  image:
    repository: ghcr.io/pistonprotection/gateway
    pullPolicy: IfNotPresent
  service:
    type: ClusterIP
    port: 8080
    grpcPort: 9090
  resources:
    requests:
      cpu: 500m
      memory: 512Mi
    limits:
      cpu: 2000m
      memory: 2Gi
  autoscaling:
    enabled: true
    minReplicas: 3
    maxReplicas: 20
    targetCPUUtilization: 70
  affinity:
    podAntiAffinity:
      requiredDuringSchedulingIgnoredDuringExecution:
        - labelSelector:
            matchExpressions:
              - key: app.kubernetes.io/component
                operator: In
                values:
                  - gateway
          topologyKey: kubernetes.io/hostname

# -----------------------------------------------------------------------------
# Worker Service - eBPF/XDP Packet Filtering
# -----------------------------------------------------------------------------
worker:
  enabled: true
  kind: DaemonSet
  image:
    repository: ghcr.io/pistonprotection/worker
    pullPolicy: IfNotPresent
  hostNetwork: true
  dnsPolicy: ClusterFirstWithHostNet
  securityContext:
    privileged: true
    capabilities:
      add:
        - NET_ADMIN
        - SYS_ADMIN
        - BPF
  resources:
    requests:
      cpu: 1000m
      memory: 1Gi
    limits:
      cpu: 4000m
      memory: 4Gi
  nodeSelector:
    pistonprotection.io/worker: "true"
  tolerations:
    - key: "pistonprotection.io/worker"
      operator: "Exists"
      effect: "NoSchedule"

# -----------------------------------------------------------------------------
# Config Manager - Configuration Distribution
# -----------------------------------------------------------------------------
configMgr:
  enabled: true
  replicaCount: 2
  resources:
    requests:
      cpu: 200m
      memory: 256Mi
    limits:
      cpu: 500m
      memory: 512Mi

# -----------------------------------------------------------------------------
# Metrics Collector - Metrics Aggregation
# -----------------------------------------------------------------------------
metrics:
  enabled: true
  replicaCount: 2
  resources:
    requests:
      cpu: 500m
      memory: 512Mi
    limits:
      cpu: 2000m
      memory: 2Gi

# -----------------------------------------------------------------------------
# Kubernetes Operator
# -----------------------------------------------------------------------------
operator:
  enabled: true
  replicaCount: 1
  resources:
    requests:
      cpu: 200m
      memory: 256Mi
    limits:
      cpu: 500m
      memory: 512Mi
  installCRDs: true

# -----------------------------------------------------------------------------
# Frontend Dashboard
# -----------------------------------------------------------------------------
frontend:
  enabled: true
  replicaCount: 3
  resources:
    requests:
      cpu: 100m
      memory: 128Mi
    limits:
      cpu: 500m
      memory: 512Mi

# -----------------------------------------------------------------------------
# Ingress Configuration
# -----------------------------------------------------------------------------
ingress:
  enabled: true
  className: "nginx"
  annotations:
    cert-manager.io/cluster-issuer: "letsencrypt-prod"
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
    nginx.ingress.kubernetes.io/proxy-body-size: "100m"
    nginx.ingress.kubernetes.io/proxy-read-timeout: "3600"
    nginx.ingress.kubernetes.io/proxy-send-timeout: "3600"
  hosts:
    - host: protection.yourdomain.com
      paths:
        - path: /
          pathType: Prefix
          service: frontend
        - path: /api
          pathType: Prefix
          service: gateway
        - path: /grpc
          pathType: Prefix
          service: gateway
  tls:
    - secretName: pistonprotection-tls
      hosts:
        - protection.yourdomain.com

# -----------------------------------------------------------------------------
# Database Configuration
# -----------------------------------------------------------------------------
# External PostgreSQL (recommended for production)
postgresql:
  enabled: false
  external:
    host: "your-postgres.database.example.com"
    port: 5432
    database: pistonprotection
    username: pistonprotection
    existingSecret: "postgres-credentials"

# External Redis (recommended for production)
redis:
  enabled: false
  external:
    host: "your-redis.cache.example.com"
    port: 6379
    existingSecret: "redis-credentials"

# -----------------------------------------------------------------------------
# Observability
# -----------------------------------------------------------------------------
observability:
  prometheus:
    enabled: true
    serviceMonitor:
      enabled: true
      namespace: monitoring
      interval: 15s
      scrapeTimeout: 10s
  grafana:
    enabled: true
    dashboards:
      enabled: true
  loki:
    enabled: true

# -----------------------------------------------------------------------------
# Security Settings
# -----------------------------------------------------------------------------
serviceAccount:
  create: true
  annotations:
    # For AWS IRSA
    # eks.amazonaws.com/role-arn: arn:aws:iam::ACCOUNT:role/pistonprotection

rbac:
  create: true

networkPolicy:
  enabled: true

# -----------------------------------------------------------------------------
# Protection Defaults
# -----------------------------------------------------------------------------
protection:
  defaultLevel: 3
  rateLimit:
    ppsPerIp: 5000
    burst: 10000
    globalPps: 10000000
  protocols:
    minecraft:
      enabled: true
      minVersion: 47
      maxVersion: 769
    quic:
      enabled: true
    synCookies:
      enabled: true
```

### Step 2: Create Required Secrets

```bash
# Create namespace
kubectl create namespace pistonprotection

# Create PostgreSQL credentials secret
kubectl create secret generic postgres-credentials \
    --namespace pistonprotection \
    --from-literal=password='your-secure-database-password'

# Create Redis credentials secret
kubectl create secret generic redis-credentials \
    --namespace pistonprotection \
    --from-literal=password='your-secure-redis-password'

# Create registry credentials (if using private registry)
kubectl create secret docker-registry registry-credentials \
    --namespace pistonprotection \
    --docker-server=ghcr.io \
    --docker-username=your-username \
    --docker-password=your-github-token \
    --docker-email=your-email@example.com
```

### Step 3: Label and Taint Worker Nodes

```bash
# List available nodes
kubectl get nodes

# Label nodes that will run XDP workers
kubectl label nodes node-worker-1 pistonprotection.io/worker=true
kubectl label nodes node-worker-2 pistonprotection.io/worker=true
kubectl label nodes node-worker-3 pistonprotection.io/worker=true

# Optional: Taint nodes for dedicated workloads
kubectl taint nodes node-worker-1 pistonprotection.io/worker=true:NoSchedule
kubectl taint nodes node-worker-2 pistonprotection.io/worker=true:NoSchedule
kubectl taint nodes node-worker-3 pistonprotection.io/worker=true:NoSchedule
```

### Step 4: Install PistonProtection

```bash
# Install with production values
helm install pistonprotection pistonprotection/pistonprotection \
    --namespace pistonprotection \
    --values values-production.yaml \
    --wait --timeout 15m

# Verify deployment
kubectl get pods -n pistonprotection
kubectl get svc -n pistonprotection
kubectl get ingress -n pistonprotection
```

---

## Cloud Provider Guides

### AWS EKS

#### Create EKS Cluster

```bash
# Create cluster configuration
cat <<EOF > eks-cluster.yaml
apiVersion: eksctl.io/v1alpha5
kind: ClusterConfig

metadata:
  name: pistonprotection-cluster
  region: us-west-2
  version: "1.29"

vpc:
  cidr: 10.0.0.0/16
  nat:
    gateway: HighlyAvailable

managedNodeGroups:
  # Control plane nodes
  - name: control-plane
    instanceType: m6i.xlarge
    desiredCapacity: 3
    minSize: 3
    maxSize: 5
    volumeSize: 100
    volumeType: gp3
    labels:
      role: control-plane
    privateNetworking: true

  # Worker nodes for XDP filtering
  - name: ddos-workers
    instanceType: c6i.2xlarge
    desiredCapacity: 3
    minSize: 3
    maxSize: 10
    volumeSize: 100
    volumeType: gp3
    labels:
      role: worker
      pistonprotection.io/worker: "true"
    taints:
      - key: pistonprotection.io/worker
        value: "true"
        effect: NoSchedule
    privateNetworking: true
    # ENA Express for enhanced networking
    amiFamily: AmazonLinux2
    preBootstrapCommands:
      - "yum install -y kernel-devel-\$(uname -r)"
EOF

# Create the cluster
eksctl create cluster -f eks-cluster.yaml

# Update kubeconfig
aws eks update-kubeconfig --name pistonprotection-cluster --region us-west-2
```

#### Install Cilium on EKS

```bash
# Delete AWS VPC CNI (Cilium will replace it)
kubectl delete daemonset -n kube-system aws-node

# Install Cilium with ENI mode
cilium install --version 1.16.0 \
    --set eni.enabled=true \
    --set ipam.mode=eni \
    --set egressMasqueradeInterfaces=eth0 \
    --set routingMode=native \
    --set kubeProxyReplacement=true \
    --set hubble.enabled=true \
    --set hubble.relay.enabled=true

# Verify Cilium
cilium status --wait
```

#### Create EKS-Specific Values

```yaml
# values-eks.yaml
global:
  storageClass: "gp3"

postgresql:
  enabled: true
  primary:
    persistence:
      storageClass: "gp3"
      size: 50Gi

redis:
  enabled: true
  master:
    persistence:
      storageClass: "gp3"
      size: 10Gi

ingress:
  enabled: true
  className: "alb"
  annotations:
    alb.ingress.kubernetes.io/scheme: internet-facing
    alb.ingress.kubernetes.io/target-type: ip
    alb.ingress.kubernetes.io/certificate-arn: arn:aws:acm:us-west-2:ACCOUNT:certificate/CERT-ID
    alb.ingress.kubernetes.io/listen-ports: '[{"HTTPS":443}]'
    alb.ingress.kubernetes.io/ssl-redirect: '443'
```

```bash
# Install PistonProtection on EKS
helm install pistonprotection pistonprotection/pistonprotection \
    --namespace pistonprotection \
    --create-namespace \
    --values values-eks.yaml
```

---

### Google GKE

#### Create GKE Cluster

```bash
# Set project and zone
export PROJECT_ID=your-project-id
export ZONE=us-central1-a

# Create GKE cluster with Dataplane V2 (Cilium-based)
gcloud container clusters create pistonprotection-cluster \
    --project ${PROJECT_ID} \
    --zone ${ZONE} \
    --machine-type e2-standard-4 \
    --num-nodes 3 \
    --enable-dataplane-v2 \
    --enable-ip-alias \
    --enable-autoscaling \
    --min-nodes 3 \
    --max-nodes 10 \
    --workload-pool=${PROJECT_ID}.svc.id.goog

# Create dedicated worker node pool
gcloud container node-pools create ddos-workers \
    --cluster pistonprotection-cluster \
    --zone ${ZONE} \
    --machine-type c2-standard-8 \
    --num-nodes 3 \
    --enable-autoscaling \
    --min-nodes 3 \
    --max-nodes 10 \
    --node-labels=pistonprotection.io/worker=true \
    --node-taints=pistonprotection.io/worker=true:NoSchedule

# Get credentials
gcloud container clusters get-credentials pistonprotection-cluster --zone ${ZONE}
```

#### Create GKE-Specific Values

```yaml
# values-gke.yaml
global:
  storageClass: "premium-rwo"

postgresql:
  enabled: true
  primary:
    persistence:
      storageClass: "premium-rwo"
      size: 50Gi

redis:
  enabled: true
  master:
    persistence:
      storageClass: "premium-rwo"
      size: 10Gi

serviceAccount:
  annotations:
    iam.gke.io/gcp-service-account: pistonprotection@PROJECT_ID.iam.gserviceaccount.com
```

```bash
# Install PistonProtection on GKE
helm install pistonprotection pistonprotection/pistonprotection \
    --namespace pistonprotection \
    --create-namespace \
    --values values-gke.yaml
```

---

### Azure AKS

#### Create AKS Cluster

```bash
# Set variables
RESOURCE_GROUP=pistonprotection-rg
CLUSTER_NAME=pistonprotection-cluster
LOCATION=eastus

# Create resource group
az group create --name ${RESOURCE_GROUP} --location ${LOCATION}

# Create AKS cluster with Azure CNI Overlay and Cilium
az aks create \
    --resource-group ${RESOURCE_GROUP} \
    --name ${CLUSTER_NAME} \
    --location ${LOCATION} \
    --network-plugin azure \
    --network-plugin-mode overlay \
    --network-dataplane cilium \
    --node-count 3 \
    --node-vm-size Standard_D4s_v3 \
    --enable-cluster-autoscaler \
    --min-count 3 \
    --max-count 10 \
    --enable-managed-identity \
    --generate-ssh-keys

# Add dedicated worker node pool
az aks nodepool add \
    --resource-group ${RESOURCE_GROUP} \
    --cluster-name ${CLUSTER_NAME} \
    --name ddosworkers \
    --node-count 3 \
    --node-vm-size Standard_F8s_v2 \
    --labels pistonprotection.io/worker=true \
    --node-taints pistonprotection.io/worker=true:NoSchedule \
    --enable-cluster-autoscaler \
    --min-count 3 \
    --max-count 10

# Get credentials
az aks get-credentials --resource-group ${RESOURCE_GROUP} --name ${CLUSTER_NAME}
```

#### Create AKS-Specific Values

```yaml
# values-aks.yaml
global:
  storageClass: "managed-premium"

postgresql:
  enabled: true
  primary:
    persistence:
      storageClass: "managed-premium"
      size: 50Gi

redis:
  enabled: true
  master:
    persistence:
      storageClass: "managed-premium"
      size: 10Gi
```

```bash
# Install PistonProtection on AKS
helm install pistonprotection pistonprotection/pistonprotection \
    --namespace pistonprotection \
    --create-namespace \
    --values values-aks.yaml
```

---

## Self-Hosted Kubernetes

### k0s Setup

k0s is a lightweight, certified Kubernetes distribution ideal for edge and bare-metal deployments.

#### Install k0s Controller

```bash
# Download and install k0s
curl -sSLf https://get.k0s.sh | sudo sh

# Create k0s configuration
cat <<EOF > /etc/k0s/k0s.yaml
apiVersion: k0s.k0sproject.io/v1beta1
kind: ClusterConfig
metadata:
  name: pistonprotection
spec:
  api:
    address: ${CONTROLLER_IP}
    port: 6443
    sans:
      - ${CONTROLLER_IP}
      - ${CONTROLLER_HOSTNAME}
  network:
    provider: custom  # We'll install Cilium manually
    kubeProxy:
      disabled: true  # Cilium replaces kube-proxy
  extensions:
    helm:
      repositories:
        - name: cilium
          url: https://helm.cilium.io
      charts:
        - name: cilium
          chartname: cilium/cilium
          version: "1.16.0"
          namespace: kube-system
          values: |
            kubeProxyReplacement: true
            k8sServiceHost: ${CONTROLLER_IP}
            k8sServicePort: 6443
            hubble:
              enabled: true
              relay:
                enabled: true
              ui:
                enabled: true
EOF

# Install and start k0s controller
sudo k0s install controller --config /etc/k0s/k0s.yaml
sudo k0s start

# Wait for k0s to be ready
sudo k0s status

# Get kubeconfig
sudo k0s kubeconfig admin > ~/.kube/config
chmod 600 ~/.kube/config
```

#### Join Worker Nodes

```bash
# On controller: Generate worker join token
sudo k0s token create --role worker > /tmp/worker-token

# Copy token to worker nodes, then on each worker:
curl -sSLf https://get.k0s.sh | sudo sh
sudo k0s install worker --token-file /path/to/worker-token
sudo k0s start

# On controller: Label worker nodes
kubectl label nodes worker-1 pistonprotection.io/worker=true
kubectl label nodes worker-2 pistonprotection.io/worker=true
```

#### Install PistonProtection on k0s

```bash
helm install pistonprotection pistonprotection/pistonprotection \
    --namespace pistonprotection \
    --create-namespace \
    --set global.storageClass=local-path
```

---

### k3s Setup

k3s is a lightweight Kubernetes distribution perfect for edge, IoT, and resource-constrained environments.

#### Install k3s Server

```bash
# Install k3s without default CNI and kube-proxy (Cilium will handle these)
curl -sfL https://get.k3s.io | INSTALL_K3S_EXEC="server \
    --flannel-backend=none \
    --disable-network-policy \
    --disable=traefik \
    --disable=servicelb \
    --disable-kube-proxy \
    --cluster-init" sh -

# Get the node token for workers
sudo cat /var/lib/rancher/k3s/server/node-token

# Copy kubeconfig
sudo cp /etc/rancher/k3s/k3s.yaml ~/.kube/config
sudo chown $(id -u):$(id -g) ~/.kube/config

# Get server IP for Cilium configuration
SERVER_IP=$(hostname -I | awk '{print $1}')
```

#### Install Cilium on k3s

```bash
# Install Cilium
cilium install --version 1.16.0 \
    --set kubeProxyReplacement=true \
    --set k8sServiceHost=${SERVER_IP} \
    --set k8sServicePort=6443 \
    --set hubble.enabled=true \
    --set hubble.relay.enabled=true

# Verify Cilium
cilium status --wait
```

#### Join k3s Workers

```bash
# On each worker node
curl -sfL https://get.k3s.io | K3S_URL="https://${SERVER_IP}:6443" \
    K3S_TOKEN="${NODE_TOKEN}" \
    INSTALL_K3S_EXEC="agent --flannel-backend=none" sh -

# On server: Label workers
kubectl label nodes k3s-worker-1 pistonprotection.io/worker=true
kubectl label nodes k3s-worker-2 pistonprotection.io/worker=true
```

#### Install PistonProtection on k3s

```yaml
# values-k3s.yaml
global:
  storageClass: "local-path"

gateway:
  replicaCount: 1
  resources:
    requests:
      cpu: 100m
      memory: 128Mi
    limits:
      cpu: 500m
      memory: 512Mi

postgresql:
  primary:
    persistence:
      storageClass: "local-path"
      size: 10Gi

redis:
  master:
    persistence:
      storageClass: "local-path"
      size: 2Gi
```

```bash
helm install pistonprotection pistonprotection/pistonprotection \
    --namespace pistonprotection \
    --create-namespace \
    --values values-k3s.yaml
```

---

## Verification

After installation, verify all components are running correctly.

### Check Pod Status

```bash
# List all pods
kubectl get pods -n pistonprotection -o wide

# Expected output (all pods Running):
# NAME                                              READY   STATUS    RESTARTS   AGE
# pistonprotection-gateway-xxxxxxxxxx-xxxxx         1/1     Running   0          5m
# pistonprotection-gateway-xxxxxxxxxx-xxxxx         1/1     Running   0          5m
# pistonprotection-config-mgr-xxxxxxxxxx-xxxxx      1/1     Running   0          5m
# pistonprotection-metrics-xxxxxxxxxx-xxxxx         1/1     Running   0          5m
# pistonprotection-operator-xxxxxxxxxx-xxxxx        1/1     Running   0          5m
# pistonprotection-frontend-xxxxxxxxxx-xxxxx        1/1     Running   0          5m
# pistonprotection-worker-xxxxx                     1/1     Running   0          5m  (one per worker node)
# pistonprotection-postgresql-0                     1/1     Running   0          5m
# pistonprotection-redis-master-0                   1/1     Running   0          5m
```

### Verify Worker XDP Attachment

```bash
# Check worker logs for XDP attachment
kubectl logs -n pistonprotection -l app.kubernetes.io/component=worker --tail=50 | grep -i xdp

# Expected output:
# [INFO] XDP program attached to eth0 in native mode
# [INFO] eBPF maps initialized successfully
# [INFO] Filter configuration loaded

# Verify XDP program is attached (from worker node)
kubectl exec -n pistonprotection -l app.kubernetes.io/component=worker -- ip link show eth0
# Look for: xdp/id:XX in the output
```

### Test API Health

```bash
# Port-forward the gateway
kubectl port-forward -n pistonprotection svc/pistonprotection-gateway 8080:8080 &

# Test health endpoint
curl -s http://localhost:8080/health | jq .
# Expected: {"status": "healthy", "version": "x.x.x"}

# Test readiness
curl -s http://localhost:8080/ready | jq .
```

### Verify CRDs

```bash
# Check CRDs are installed
kubectl get crd | grep pistonprotection

# Expected:
# backends.pistonprotection.io
# ddosprotections.pistonprotection.io
# filterrules.pistonprotection.io
```

### Verify Services

```bash
# List all services
kubectl get svc -n pistonprotection

# Check ingress (if enabled)
kubectl get ingress -n pistonprotection
```

---

## Upgrading

### Standard Upgrade

```bash
# Update Helm repository
helm repo update

# Check available versions
helm search repo pistonprotection --versions

# Review changes
helm diff upgrade pistonprotection pistonprotection/pistonprotection \
    --namespace pistonprotection \
    --values values.yaml

# Perform upgrade
helm upgrade pistonprotection pistonprotection/pistonprotection \
    --namespace pistonprotection \
    --values values.yaml \
    --wait --timeout 10m

# Verify upgrade
kubectl rollout status deployment/pistonprotection-gateway -n pistonprotection
kubectl rollout status daemonset/pistonprotection-worker -n pistonprotection
```

### Major Version Upgrades

For major version upgrades, review the [CHANGELOG](https://github.com/pistonprotection/app/blob/main/CHANGELOG.md) and follow the migration guide.

```bash
# Backup current configuration
helm get values pistonprotection -n pistonprotection > backup-values.yaml

# Backup CRD resources
kubectl get ddosprotections -A -o yaml > backup-ddosprotections.yaml
kubectl get filterrules -A -o yaml > backup-filterrules.yaml
kubectl get backends -A -o yaml > backup-backends.yaml
```

---

## Uninstallation

### Remove PistonProtection

```bash
# Uninstall Helm release
helm uninstall pistonprotection --namespace pistonprotection

# Wait for pods to terminate
kubectl wait --for=delete pod -l app.kubernetes.io/name=pistonprotection \
    -n pistonprotection --timeout=120s
```

### Remove CRDs (Optional)

**Warning**: This will delete all custom resources (backends, filter rules, etc.)

```bash
# Delete CRDs
kubectl delete crd backends.pistonprotection.io
kubectl delete crd ddosprotections.pistonprotection.io
kubectl delete crd filterrules.pistonprotection.io
```

### Clean Up Namespace

```bash
# Delete namespace and all remaining resources
kubectl delete namespace pistonprotection
```

### Remove Worker Node Labels

```bash
# Remove labels from worker nodes
kubectl label nodes --all pistonprotection.io/worker-

# Remove taints from worker nodes
kubectl taint nodes --all pistonprotection.io/worker-
```

---

## Troubleshooting

### Common Issues

#### Worker Pods Not Starting

**Symptom**: Worker pods stuck in `CrashLoopBackOff` or `Error` state

**Diagnosis**:
```bash
# Check pod events
kubectl describe pod -n pistonprotection -l app.kubernetes.io/component=worker

# Check logs
kubectl logs -n pistonprotection -l app.kubernetes.io/component=worker --previous
```

**Common Causes and Solutions**:

1. **Kernel too old**:
   ```bash
   # Check kernel version (must be 5.15+)
   kubectl debug node/<node-name> -it --image=ubuntu -- uname -r
   ```

2. **BTF not available**:
   ```bash
   # Check BTF support
   kubectl debug node/<node-name> -it --image=ubuntu -- ls /sys/kernel/btf/vmlinux
   ```

3. **XDP not supported**:
   ```bash
   # Check network driver
   kubectl debug node/<node-name> -it --image=ubuntu -- \
       ethtool -i eth0 | grep driver
   ```

4. **Missing capabilities**:
   - Ensure the worker pod has `privileged: true` or the required capabilities

#### Cilium Not Ready

**Symptom**: Cilium pods not becoming ready, network connectivity issues

**Diagnosis**:
```bash
# Check Cilium status
cilium status

# Check Cilium logs
kubectl logs -n kube-system -l k8s-app=cilium --tail=100
```

**Solutions**:
```bash
# Restart Cilium
kubectl rollout restart ds/cilium -n kube-system

# If persistent issues, reinstall Cilium
cilium uninstall
cilium install --version 1.16.0 --set kubeProxyReplacement=true
```

#### Database Connection Issues

**Symptom**: Services cannot connect to PostgreSQL or Redis

**Diagnosis**:
```bash
# Check database pods
kubectl get pods -n pistonprotection | grep -E 'postgresql|redis'

# Check secrets exist
kubectl get secrets -n pistonprotection | grep -E 'postgres|redis'

# Test database connection
kubectl run -n pistonprotection psql-test --rm -it --image=postgres:15 -- \
    psql -h pistonprotection-postgresql -U pistonprotection -d pistonprotection
```

#### Cannot Access Dashboard

**Symptom**: Frontend not accessible via ingress or port-forward

**Diagnosis**:
```bash
# Check frontend pods
kubectl get pods -n pistonprotection -l app.kubernetes.io/component=frontend

# Check ingress
kubectl get ingress -n pistonprotection
kubectl describe ingress -n pistonprotection pistonprotection

# Direct port-forward test
kubectl port-forward -n pistonprotection svc/pistonprotection-frontend 3000:3000
```

### Getting Help

If you continue to experience issues:

1. **Search existing issues**: [GitHub Issues](https://github.com/pistonprotection/app/issues)
2. **Join the community**: [Discord](https://discord.gg/pistonprotection)
3. **Read the operations guide**: [Operations Guide](operations.md)
4. **Check documentation**: [docs.pistonprotection.io](https://docs.pistonprotection.io)

---

## Next Steps

- [Configuration Reference](configuration.md) - Detailed configuration options
- [API Documentation](api.md) - REST and gRPC API reference
- [Protocol Filters](filters.md) - Set up DDoS protection rules
- [Architecture Overview](architecture.md) - Understand the system design
- [Operations Guide](operations.md) - Monitoring, alerting, and maintenance
