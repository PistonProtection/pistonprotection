# Operations Guide

This guide covers the operational aspects of running PistonProtection in production, including monitoring, troubleshooting, backup/recovery, scaling, and security best practices.

## Table of Contents

- [Monitoring and Alerting](#monitoring-and-alerting)
- [Troubleshooting Guide](#troubleshooting-guide)
- [Backup and Recovery](#backup-and-recovery)
- [Scaling Recommendations](#scaling-recommendations)
- [Security Best Practices](#security-best-practices)
- [Maintenance Procedures](#maintenance-procedures)

---

## Monitoring and Alerting

### Metrics Overview

PistonProtection exposes comprehensive metrics for monitoring system health and attack patterns.

#### Key Metric Categories

| Category | Description | Key Metrics |
|----------|-------------|-------------|
| Traffic | Network throughput and packet rates | `traffic_bytes_total`, `traffic_packets_total` |
| Protection | Attack detection and mitigation | `attacks_detected_total`, `packets_dropped_total` |
| Performance | System performance metrics | `latency_histogram`, `processing_time_ms` |
| Health | Component health status | `component_health`, `backend_health` |
| Resources | Resource utilization | `cpu_usage`, `memory_usage`, `ebpf_map_usage` |

### Prometheus Integration

#### Scrape Configuration

```yaml
# prometheus.yaml
scrape_configs:
  - job_name: 'pistonprotection-gateway'
    kubernetes_sd_configs:
      - role: endpoints
        namespaces:
          names:
            - pistonprotection
    relabel_configs:
      - source_labels: [__meta_kubernetes_service_name]
        regex: pistonprotection-gateway
        action: keep
      - source_labels: [__meta_kubernetes_endpoint_port_name]
        regex: metrics
        action: keep

  - job_name: 'pistonprotection-worker'
    kubernetes_sd_configs:
      - role: pod
        namespaces:
          names:
            - pistonprotection
    relabel_configs:
      - source_labels: [__meta_kubernetes_pod_label_app]
        regex: pistonprotection-worker
        action: keep
```

#### Key Metrics to Monitor

```promql
# Traffic throughput (bytes/second)
rate(pistonprotection_traffic_bytes_total[5m])

# Packet rate (packets/second)
rate(pistonprotection_traffic_packets_total[5m])

# Attack detection rate
rate(pistonprotection_attacks_detected_total[5m])

# Drop rate percentage
rate(pistonprotection_packets_dropped_total[5m]) /
rate(pistonprotection_traffic_packets_total[5m]) * 100

# API latency (p99)
histogram_quantile(0.99, rate(pistonprotection_api_latency_bucket[5m]))

# Backend health
pistonprotection_backend_health{status="healthy"}

# eBPF map utilization
pistonprotection_ebpf_map_entries / pistonprotection_ebpf_map_max_entries * 100
```

### Grafana Dashboards

PistonProtection includes pre-built Grafana dashboards.

#### Dashboard Installation

```bash
# Import dashboards from ConfigMap
kubectl apply -f - <<EOF
apiVersion: v1
kind: ConfigMap
metadata:
  name: pistonprotection-dashboards
  namespace: monitoring
  labels:
    grafana_dashboard: "1"
data:
  overview.json: |
    $(cat dashboards/overview.json)
  attacks.json: |
    $(cat dashboards/attacks.json)
  backends.json: |
    $(cat dashboards/backends.json)
EOF
```

#### Available Dashboards

1. **Overview Dashboard** - System-wide health and traffic
2. **Attack Dashboard** - Attack patterns and mitigation stats
3. **Backend Dashboard** - Per-backend metrics and health
4. **Worker Dashboard** - XDP worker performance
5. **API Dashboard** - API request rates and latencies

### Alerting Rules

#### Critical Alerts

```yaml
# alerting-rules.yaml
groups:
  - name: pistonprotection-critical
    rules:
      # High attack volume
      - alert: HighAttackVolume
        expr: |
          rate(pistonprotection_attacks_detected_total[5m]) > 1000
        for: 2m
        labels:
          severity: critical
        annotations:
          summary: "High attack volume detected"
          description: "Attack rate is {{ $value }}/s on {{ $labels.backend }}"

      # Service down
      - alert: ServiceDown
        expr: |
          up{job=~"pistonprotection.*"} == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "PistonProtection service is down"
          description: "{{ $labels.job }} has been down for more than 1 minute"

      # All backends unhealthy
      - alert: AllBackendsUnhealthy
        expr: |
          sum(pistonprotection_backend_health{status="healthy"}) == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "All backends are unhealthy"
          description: "No healthy backends available"

      # High drop rate
      - alert: HighDropRate
        expr: |
          rate(pistonprotection_packets_dropped_total[5m]) /
          rate(pistonprotection_traffic_packets_total[5m]) > 0.5
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "High packet drop rate"
          description: "Drop rate is {{ $value | humanizePercentage }}"
```

#### Warning Alerts

```yaml
groups:
  - name: pistonprotection-warning
    rules:
      # Elevated latency
      - alert: ElevatedLatency
        expr: |
          histogram_quantile(0.99, rate(pistonprotection_api_latency_bucket[5m])) > 0.5
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "API latency elevated"
          description: "P99 latency is {{ $value }}s"

      # Backend degraded
      - alert: BackendDegraded
        expr: |
          pistonprotection_backend_healthy_origins /
          pistonprotection_backend_total_origins < 0.5
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Backend is degraded"
          description: "Backend {{ $labels.backend }} has only {{ $value | humanizePercentage }} healthy origins"

      # eBPF map near capacity
      - alert: EBPFMapNearCapacity
        expr: |
          pistonprotection_ebpf_map_entries / pistonprotection_ebpf_map_max_entries > 0.8
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "eBPF map near capacity"
          description: "Map {{ $labels.map_name }} is {{ $value | humanizePercentage }} full"

      # Certificate expiring soon
      - alert: CertificateExpiringSoon
        expr: |
          pistonprotection_certificate_expiry_seconds < 604800
        labels:
          severity: warning
        annotations:
          summary: "Certificate expiring soon"
          description: "Certificate for {{ $labels.domain }} expires in {{ $value | humanizeDuration }}"
```

### Alert Notification Channels

#### Slack Integration

```yaml
# alertmanager.yaml
receivers:
  - name: 'slack-critical'
    slack_configs:
      - api_url: 'https://hooks.slack.com/services/XXX/YYY/ZZZ'
        channel: '#alerts-critical'
        title: '{{ .Status | toUpper }}: {{ .CommonAnnotations.summary }}'
        text: '{{ .CommonAnnotations.description }}'
        send_resolved: true

  - name: 'slack-warning'
    slack_configs:
      - api_url: 'https://hooks.slack.com/services/XXX/YYY/ZZZ'
        channel: '#alerts-warning'
        send_resolved: true

route:
  receiver: 'slack-warning'
  routes:
    - match:
        severity: critical
      receiver: 'slack-critical'
```

#### PagerDuty Integration

```yaml
receivers:
  - name: 'pagerduty'
    pagerduty_configs:
      - service_key: '<pagerduty-service-key>'
        severity: '{{ .CommonLabels.severity }}'
        description: '{{ .CommonAnnotations.summary }}'
        details:
          description: '{{ .CommonAnnotations.description }}'
          runbook: '{{ .CommonAnnotations.runbook_url }}'
```

---

## Troubleshooting Guide

### Common Issues

#### 1. Traffic Not Being Filtered

**Symptoms:**
- Attack traffic reaching backend servers
- No drop metrics showing
- Backends reporting high load

**Diagnosis:**

```bash
# Check if XDP programs are loaded
kubectl exec -n pistonprotection deploy/pistonprotection-worker -- \
  bpftool prog list

# Check worker logs
kubectl logs -n pistonprotection -l app=pistonprotection-worker --tail=100

# Verify filter rules are synced
kubectl exec -n pistonprotection deploy/pistonprotection-worker -- \
  bpftool map dump name filter_rules

# Check interface attachment
kubectl exec -n pistonprotection deploy/pistonprotection-worker -- \
  ip link show | grep xdp
```

**Solutions:**

```bash
# Restart worker to reload XDP programs
kubectl rollout restart daemonset/pistonprotection-worker -n pistonprotection

# Force rule sync
kubectl exec -n pistonprotection deploy/pistonprotection-gateway -- \
  curl -X POST http://localhost:8080/internal/sync-rules

# Check for Cilium conflicts
kubectl get ciliumnetworkpolicy -A
```

#### 2. High Latency

**Symptoms:**
- API responses slow
- Dashboard loading slowly
- Backend timeout errors

**Diagnosis:**

```bash
# Check API latency
kubectl exec -n pistonprotection deploy/pistonprotection-gateway -- \
  curl -w "@curl-format.txt" http://localhost:8080/health

# Check database performance
kubectl exec -n pistonprotection deploy/postgresql -- \
  psql -U postgres -c "SELECT * FROM pg_stat_activity WHERE state = 'active';"

# Check Redis latency
kubectl exec -n pistonprotection deploy/redis-master -- \
  redis-cli --latency

# Check pod resource usage
kubectl top pods -n pistonprotection
```

**Solutions:**

```bash
# Scale gateway horizontally
kubectl scale deployment pistonprotection-gateway -n pistonprotection --replicas=5

# Increase resource limits
kubectl patch deployment pistonprotection-gateway -n pistonprotection --patch '
spec:
  template:
    spec:
      containers:
      - name: gateway
        resources:
          limits:
            memory: 4Gi
            cpu: 2000m'

# Optimize database
kubectl exec -n pistonprotection deploy/postgresql -- \
  psql -U postgres -c "VACUUM ANALYZE;"
```

#### 3. Backend Health Check Failures

**Symptoms:**
- Backends showing as unhealthy
- Origin servers marked down
- Traffic not being proxied

**Diagnosis:**

```bash
# Check backend status via API
curl -H "Authorization: Bearer $TOKEN" \
  https://api.example.com/v1/backends/BACKEND_ID

# Check health check logs
kubectl logs -n pistonprotection deploy/pistonprotection-gateway \
  --tail=100 | grep "health_check"

# Test connectivity to origin
kubectl exec -n pistonprotection deploy/pistonprotection-gateway -- \
  curl -v http://origin-server:8080/health
```

**Solutions:**

```bash
# Adjust health check settings
curl -X PATCH \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "health_check": {
      "interval": "30s",
      "timeout": "10s",
      "unhealthy_threshold": 5
    }
  }' \
  https://api.example.com/v1/backends/BACKEND_ID

# Force health check
curl -X POST \
  -H "Authorization: Bearer $TOKEN" \
  https://api.example.com/v1/backends/BACKEND_ID/health-check
```

#### 4. Authentication Issues

**Symptoms:**
- 401 Unauthorized errors
- Token refresh failures
- Session timeouts

**Diagnosis:**

```bash
# Check auth service logs
kubectl logs -n pistonprotection deploy/pistonprotection-auth --tail=100

# Verify JWT secret
kubectl get secret pistonprotection-jwt -n pistonprotection -o jsonpath='{.data.secret}' | base64 -d

# Check token validity
curl -X POST \
  -H "Authorization: Bearer $TOKEN" \
  https://api.example.com/v1/auth/validate
```

**Solutions:**

```bash
# Regenerate JWT secret
kubectl create secret generic pistonprotection-jwt \
  -n pistonprotection \
  --from-literal=secret=$(openssl rand -base64 32) \
  --dry-run=client -o yaml | kubectl apply -f -

# Restart auth service
kubectl rollout restart deployment/pistonprotection-auth -n pistonprotection

# Clear session cache
kubectl exec -n pistonprotection deploy/redis-master -- \
  redis-cli KEYS "session:*" | xargs redis-cli DEL
```

#### 5. eBPF Map Full

**Symptoms:**
- New connections being dropped
- Rate limiting not working correctly
- Warning logs about map capacity

**Diagnosis:**

```bash
# Check map usage
kubectl exec -n pistonprotection deploy/pistonprotection-worker -- \
  bpftool map show name rate_limit_map

# Get map entry count
kubectl exec -n pistonprotection deploy/pistonprotection-worker -- \
  bpftool map dump name rate_limit_map | wc -l

# Check for stale entries
kubectl exec -n pistonprotection deploy/pistonprotection-worker -- \
  cat /proc/$(pgrep -f pistonprotection-worker)/status
```

**Solutions:**

```bash
# Increase map size (requires worker restart)
kubectl patch daemonset pistonprotection-worker -n pistonprotection --patch '
spec:
  template:
    spec:
      containers:
      - name: worker
        env:
        - name: EBPF_MAP_SIZE
          value: "200000"'

# Clear stale entries
kubectl exec -n pistonprotection deploy/pistonprotection-worker -- \
  curl -X POST http://localhost:9091/internal/gc-maps

# Reduce entry TTL
kubectl patch configmap pistonprotection-worker-config -n pistonprotection --patch '
data:
  rate_limit_ttl: "300"'
```

### Diagnostic Commands

#### Cluster Health Check

```bash
#!/bin/bash
# cluster-health.sh

echo "=== PistonProtection Health Check ==="

echo -e "\n--- Pod Status ---"
kubectl get pods -n pistonprotection -o wide

echo -e "\n--- Service Status ---"
kubectl get svc -n pistonprotection

echo -e "\n--- Recent Events ---"
kubectl get events -n pistonprotection --sort-by='.lastTimestamp' | tail -20

echo -e "\n--- Resource Usage ---"
kubectl top pods -n pistonprotection

echo -e "\n--- PVC Status ---"
kubectl get pvc -n pistonprotection

echo -e "\n--- Gateway Logs (last 20 lines) ---"
kubectl logs -n pistonprotection deploy/pistonprotection-gateway --tail=20

echo -e "\n--- Worker Logs (last 20 lines) ---"
kubectl logs -n pistonprotection -l app=pistonprotection-worker --tail=20

echo -e "\n=== Health Check Complete ==="
```

#### Traffic Analysis

```bash
#!/bin/bash
# traffic-analysis.sh

NAMESPACE=${1:-pistonprotection}

echo "=== Traffic Analysis ==="

# Get traffic metrics
kubectl exec -n $NAMESPACE deploy/pistonprotection-metrics -- \
  curl -s localhost:9090/metrics | grep pistonprotection_traffic

# Get attack metrics
kubectl exec -n $NAMESPACE deploy/pistonprotection-metrics -- \
  curl -s localhost:9090/metrics | grep pistonprotection_attack

# Get top attacked backends
kubectl exec -n $NAMESPACE deploy/pistonprotection-gateway -- \
  curl -s localhost:8080/internal/stats | jq '.top_attacked_backends'

# Get top attack sources
kubectl exec -n $NAMESPACE deploy/pistonprotection-gateway -- \
  curl -s localhost:8080/internal/stats | jq '.top_attack_sources[:10]'
```

### Log Analysis

#### Log Queries

```bash
# Find all errors in the last hour
kubectl logs -n pistonprotection -l app.kubernetes.io/name=pistonprotection \
  --since=1h | grep -i error

# Find authentication failures
kubectl logs -n pistonprotection deploy/pistonprotection-auth \
  --since=1h | grep "authentication_failed"

# Find dropped packets by reason
kubectl logs -n pistonprotection -l app=pistonprotection-worker \
  --since=1h | grep "packet_dropped" | jq -r '.reason' | sort | uniq -c | sort -rn

# Find slow API requests
kubectl logs -n pistonprotection deploy/pistonprotection-gateway \
  --since=1h | jq 'select(.latency_ms > 1000)'
```

#### Log Aggregation with Loki

```yaml
# loki-query examples

# All errors
{namespace="pistonprotection"} |= "error"

# Attack detections
{namespace="pistonprotection", app="pistonprotection-worker"} |= "attack_detected"

# Rate limited requests
{namespace="pistonprotection"} |= "rate_limited" | json | rate_limited="true"

# High latency requests (>500ms)
{namespace="pistonprotection", app="pistonprotection-gateway"} | json | latency_ms > 500
```

---

## Backup and Recovery

### What to Backup

| Component | Data | Priority | Method |
|-----------|------|----------|--------|
| PostgreSQL | All application data | Critical | pg_dump |
| Redis | Session cache | Medium | RDB snapshot |
| Secrets | JWT keys, API keys | Critical | Kubernetes backup |
| ConfigMaps | Configuration | High | Kubernetes backup |
| CRDs | Backend definitions | High | Kubernetes backup |

### Database Backup

#### Manual Backup

```bash
# Create database backup
kubectl exec -n pistonprotection deploy/postgresql -- \
  pg_dump -U postgres pistonprotection | gzip > backup-$(date +%Y%m%d).sql.gz

# Upload to S3
aws s3 cp backup-$(date +%Y%m%d).sql.gz \
  s3://my-backups/pistonprotection/db/
```

#### Automated Backup with CronJob

```yaml
apiVersion: batch/v1
kind: CronJob
metadata:
  name: postgres-backup
  namespace: pistonprotection
spec:
  schedule: "0 2 * * *"  # Daily at 2 AM
  concurrencyPolicy: Forbid
  jobTemplate:
    spec:
      template:
        spec:
          containers:
          - name: backup
            image: postgres:15
            command:
            - /bin/bash
            - -c
            - |
              BACKUP_FILE="backup-$(date +%Y%m%d-%H%M%S).sql.gz"
              pg_dump -h postgresql -U postgres pistonprotection | gzip > /backup/$BACKUP_FILE
              # Upload to S3 (requires aws-cli)
              aws s3 cp /backup/$BACKUP_FILE s3://my-backups/pistonprotection/db/
              # Keep only last 7 local backups
              ls -t /backup/*.gz | tail -n +8 | xargs rm -f
            env:
            - name: PGPASSWORD
              valueFrom:
                secretKeyRef:
                  name: postgresql
                  key: postgres-password
            - name: AWS_ACCESS_KEY_ID
              valueFrom:
                secretKeyRef:
                  name: aws-credentials
                  key: access-key-id
            - name: AWS_SECRET_ACCESS_KEY
              valueFrom:
                secretKeyRef:
                  name: aws-credentials
                  key: secret-access-key
            volumeMounts:
            - name: backup-storage
              mountPath: /backup
          volumes:
          - name: backup-storage
            persistentVolumeClaim:
              claimName: backup-pvc
          restartPolicy: OnFailure
```

### Kubernetes Resource Backup

#### Using Velero

```bash
# Install Velero
velero install \
  --provider aws \
  --plugins velero/velero-plugin-for-aws:v1.7.0 \
  --bucket pistonprotection-backups \
  --secret-file ./credentials-velero \
  --backup-location-config region=us-east-1

# Create backup
velero backup create pistonprotection-backup \
  --include-namespaces pistonprotection \
  --include-cluster-resources=true

# Schedule regular backups
velero schedule create pistonprotection-daily \
  --schedule="0 3 * * *" \
  --include-namespaces pistonprotection \
  --ttl 168h
```

#### Manual Resource Export

```bash
#!/bin/bash
# export-resources.sh

BACKUP_DIR="backup-$(date +%Y%m%d)"
mkdir -p $BACKUP_DIR

# Export all PistonProtection resources
for resource in secrets configmaps deployments services daemonsets; do
  kubectl get $resource -n pistonprotection -o yaml > $BACKUP_DIR/$resource.yaml
done

# Export CRDs and CRs
kubectl get crd -o yaml | grep -A1000 'pistonprotection' > $BACKUP_DIR/crds.yaml
kubectl get backends.pistonprotection.io -A -o yaml > $BACKUP_DIR/backends.yaml
kubectl get filterrules.pistonprotection.io -A -o yaml > $BACKUP_DIR/filterrules.yaml

# Create archive
tar -czvf $BACKUP_DIR.tar.gz $BACKUP_DIR/
```

### Recovery Procedures

#### Database Recovery

```bash
# Download backup
aws s3 cp s3://my-backups/pistonprotection/db/backup-20240115.sql.gz .

# Restore database
gunzip -c backup-20240115.sql.gz | kubectl exec -i -n pistonprotection deploy/postgresql -- \
  psql -U postgres pistonprotection

# Verify restoration
kubectl exec -n pistonprotection deploy/postgresql -- \
  psql -U postgres pistonprotection -c "SELECT COUNT(*) FROM backends;"
```

#### Full Cluster Recovery

```bash
# Using Velero
velero restore create --from-backup pistonprotection-backup

# Manual recovery
kubectl apply -f backup-20240115/crds.yaml
kubectl create namespace pistonprotection
kubectl apply -f backup-20240115/secrets.yaml
kubectl apply -f backup-20240115/configmaps.yaml
kubectl apply -f backup-20240115/deployments.yaml
kubectl apply -f backup-20240115/services.yaml
kubectl apply -f backup-20240115/daemonsets.yaml

# Restore database
# (follow database recovery steps)

# Restore custom resources
kubectl apply -f backup-20240115/backends.yaml
kubectl apply -f backup-20240115/filterrules.yaml

# Verify
kubectl get pods -n pistonprotection
kubectl get backends.pistonprotection.io -A
```

### Disaster Recovery Plan

#### RTO/RPO Targets

| Component | RPO | RTO |
|-----------|-----|-----|
| Configuration | 24 hours | 1 hour |
| Database | 1 hour | 2 hours |
| Traffic Protection | 0 | 15 minutes |

#### DR Runbook

1. **Detection** (0-5 minutes)
   - Automated alerts trigger
   - On-call engineer paged

2. **Assessment** (5-15 minutes)
   - Identify scope of failure
   - Determine recovery approach

3. **Recovery** (15-60 minutes)
   - Spin up new cluster (if needed)
   - Restore from backup
   - Verify functionality

4. **Validation** (60-90 minutes)
   - Run integration tests
   - Verify metrics collection
   - Confirm attack mitigation working

5. **Post-Incident** (within 24 hours)
   - Document incident
   - Update runbooks
   - Implement preventive measures

---

## Scaling Recommendations

### Horizontal Scaling

#### Gateway Service

```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: pistonprotection-gateway-hpa
  namespace: pistonprotection
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: pistonprotection-gateway
  minReplicas: 3
  maxReplicas: 20
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
  - type: Pods
    pods:
      metric:
        name: requests_per_second
      target:
        type: AverageValue
        averageValue: "1000"
  behavior:
    scaleUp:
      stabilizationWindowSeconds: 60
      policies:
      - type: Percent
        value: 100
        periodSeconds: 60
    scaleDown:
      stabilizationWindowSeconds: 300
      policies:
      - type: Percent
        value: 10
        periodSeconds: 60
```

#### Worker DaemonSet

Workers scale automatically with nodes. To scale workers:

```bash
# Add more nodes to the cluster
# AWS EKS
eksctl scale nodegroup --cluster=pistonprotection --nodes=10 --name=worker-nodes

# GKE
gcloud container clusters resize pistonprotection --node-pool=worker-pool --num-nodes=10
```

### Vertical Scaling

#### Resource Recommendations by Load

| Traffic Level | Gateway CPU | Gateway Memory | Worker CPU | Worker Memory |
|---------------|-------------|----------------|------------|---------------|
| Low (<1 Gbps) | 500m | 512Mi | 500m | 256Mi |
| Medium (1-10 Gbps) | 2000m | 2Gi | 1000m | 512Mi |
| High (10-40 Gbps) | 4000m | 4Gi | 2000m | 1Gi |
| Very High (>40 Gbps) | 8000m | 8Gi | 4000m | 2Gi |

```bash
# Update resource limits
kubectl patch deployment pistonprotection-gateway -n pistonprotection --patch '
spec:
  template:
    spec:
      containers:
      - name: gateway
        resources:
          requests:
            cpu: 2000m
            memory: 2Gi
          limits:
            cpu: 4000m
            memory: 4Gi'
```

### Database Scaling

#### PostgreSQL Read Replicas

```yaml
# values.yaml
postgresql:
  architecture: replication
  primary:
    resources:
      requests:
        memory: 4Gi
        cpu: 2000m
  readReplicas:
    replicaCount: 3
    resources:
      requests:
        memory: 2Gi
        cpu: 1000m
```

#### Redis Cluster Mode

```yaml
# values.yaml
redis:
  architecture: replication
  master:
    resources:
      requests:
        memory: 2Gi
  replica:
    replicaCount: 3
    resources:
      requests:
        memory: 1Gi
```

### Capacity Planning

#### Estimating Requirements

```python
# capacity-calculator.py

def estimate_resources(
    traffic_gbps: float,
    attack_ratio: float,  # Expected attack traffic percentage
    num_backends: int,
    num_rules: int
) -> dict:
    """Estimate resource requirements for PistonProtection."""

    # Base requirements
    base_gateway_cpu = 500  # millicores
    base_gateway_mem = 512  # MB
    base_worker_cpu = 500
    base_worker_mem = 256

    # Traffic-based scaling
    traffic_multiplier = max(1, traffic_gbps / 1)  # 1 Gbps baseline

    # Attack traffic increases processing
    attack_multiplier = 1 + (attack_ratio * 2)

    # Rule complexity
    rule_multiplier = 1 + (num_rules / 100) * 0.1

    # Calculate requirements
    gateway_cpu = int(base_gateway_cpu * traffic_multiplier * attack_multiplier * rule_multiplier)
    gateway_mem = int(base_gateway_mem * traffic_multiplier * 1.5)
    worker_cpu = int(base_worker_cpu * traffic_multiplier * attack_multiplier)
    worker_mem = int(base_worker_mem * traffic_multiplier)

    # Replica counts
    gateway_replicas = max(3, int(traffic_gbps / 5))  # 1 replica per 5 Gbps
    worker_nodes = max(2, int(traffic_gbps / 10))  # 1 node per 10 Gbps

    return {
        "gateway": {
            "replicas": gateway_replicas,
            "cpu_millicores": gateway_cpu,
            "memory_mb": gateway_mem
        },
        "worker": {
            "nodes": worker_nodes,
            "cpu_millicores": worker_cpu,
            "memory_mb": worker_mem
        },
        "database": {
            "cpu_millicores": 1000 + (num_backends * 10),
            "memory_mb": 2048 + (num_backends * 5),
            "storage_gb": 20 + (num_backends * 0.5)
        }
    }

# Example usage
requirements = estimate_resources(
    traffic_gbps=25,
    attack_ratio=0.3,
    num_backends=100,
    num_rules=500
)
print(requirements)
```

---

## Security Best Practices

### Network Security

#### Network Policies

```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: pistonprotection-gateway
  namespace: pistonprotection
spec:
  podSelector:
    matchLabels:
      app: pistonprotection-gateway
  policyTypes:
  - Ingress
  - Egress
  ingress:
  - from:
    - namespaceSelector:
        matchLabels:
          name: ingress-nginx
    ports:
    - port: 8080
    - port: 9090
  - from:
    - podSelector:
        matchLabels:
          app: pistonprotection-frontend
    ports:
    - port: 8080
  egress:
  - to:
    - podSelector:
        matchLabels:
          app: postgresql
    ports:
    - port: 5432
  - to:
    - podSelector:
        matchLabels:
          app: redis
    ports:
    - port: 6379
  - to:
    - podSelector:
        matchLabels:
          app: pistonprotection-worker
    ports:
    - port: 9091
```

#### Cilium Network Policies

```yaml
apiVersion: cilium.io/v2
kind: CiliumNetworkPolicy
metadata:
  name: pistonprotection-strict
  namespace: pistonprotection
spec:
  endpointSelector:
    matchLabels:
      app.kubernetes.io/name: pistonprotection
  ingress:
  - fromEndpoints:
    - matchLabels:
        app.kubernetes.io/name: pistonprotection
  - fromEntities:
    - world
    toPorts:
    - ports:
      - port: "8080"
        protocol: TCP
  egress:
  - toEndpoints:
    - matchLabels:
        app.kubernetes.io/name: pistonprotection
  - toEntities:
    - world
    toPorts:
    - ports:
      - port: "443"
        protocol: TCP
```

### Secret Management

#### External Secrets Operator

```yaml
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: pistonprotection-secrets
  namespace: pistonprotection
spec:
  refreshInterval: 1h
  secretStoreRef:
    name: aws-secrets-manager
    kind: SecretStore
  target:
    name: pistonprotection-secrets
    creationPolicy: Owner
  data:
  - secretKey: jwt-secret
    remoteRef:
      key: pistonprotection/jwt-secret
  - secretKey: db-password
    remoteRef:
      key: pistonprotection/db-password
```

#### Secret Rotation

```bash
#!/bin/bash
# rotate-secrets.sh

# Generate new JWT secret
NEW_JWT_SECRET=$(openssl rand -base64 32)

# Update in secrets manager
aws secretsmanager update-secret \
  --secret-id pistonprotection/jwt-secret \
  --secret-string "$NEW_JWT_SECRET"

# Trigger ExternalSecret refresh
kubectl annotate externalsecret pistonprotection-secrets \
  -n pistonprotection \
  force-sync=$(date +%s) \
  --overwrite

# Rolling restart to pick up new secret
kubectl rollout restart deployment/pistonprotection-gateway -n pistonprotection
kubectl rollout restart deployment/pistonprotection-auth -n pistonprotection
```

### RBAC Configuration

#### Service Accounts

```yaml
apiVersion: v1
kind: ServiceAccount
metadata:
  name: pistonprotection-gateway
  namespace: pistonprotection
---
apiVersion: rbac.authorization.k8s.io/v1
kind: Role
metadata:
  name: pistonprotection-gateway-role
  namespace: pistonprotection
rules:
- apiGroups: [""]
  resources: ["secrets", "configmaps"]
  verbs: ["get", "list", "watch"]
- apiGroups: ["pistonprotection.io"]
  resources: ["backends", "filterrules"]
  verbs: ["get", "list", "watch"]
---
apiVersion: rbac.authorization.k8s.io/v1
kind: RoleBinding
metadata:
  name: pistonprotection-gateway-binding
  namespace: pistonprotection
subjects:
- kind: ServiceAccount
  name: pistonprotection-gateway
roleRef:
  kind: Role
  name: pistonprotection-gateway-role
  apiGroup: rbac.authorization.k8s.io
```

#### Pod Security Standards

```yaml
apiVersion: v1
kind: Namespace
metadata:
  name: pistonprotection
  labels:
    pod-security.kubernetes.io/enforce: restricted
    pod-security.kubernetes.io/audit: restricted
    pod-security.kubernetes.io/warn: restricted
---
# Worker requires privileged for eBPF
apiVersion: v1
kind: Namespace
metadata:
  name: pistonprotection-workers
  labels:
    pod-security.kubernetes.io/enforce: privileged
```

### Audit Logging

#### Enable Kubernetes Audit Logging

```yaml
# audit-policy.yaml
apiVersion: audit.k8s.io/v1
kind: Policy
rules:
- level: RequestResponse
  namespaces: ["pistonprotection"]
  verbs: ["create", "update", "patch", "delete"]
  resources:
  - group: ""
    resources: ["secrets", "configmaps"]
  - group: "pistonprotection.io"
    resources: ["*"]
- level: Metadata
  namespaces: ["pistonprotection"]
  verbs: ["get", "list", "watch"]
```

#### Application Audit Logging

```bash
# Query audit logs
kubectl logs -n pistonprotection deploy/pistonprotection-auth | \
  jq 'select(.event_type == "audit")'

# Export audit logs
kubectl logs -n pistonprotection deploy/pistonprotection-auth \
  --since=24h | jq 'select(.event_type == "audit")' > audit-$(date +%Y%m%d).json
```

### Security Scanning

#### Container Image Scanning

```yaml
# .github/workflows/security-scan.yaml
name: Security Scan
on:
  push:
    branches: [main]
  schedule:
    - cron: '0 0 * * *'

jobs:
  trivy-scan:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    - name: Run Trivy vulnerability scanner
      uses: aquasecurity/trivy-action@master
      with:
        image-ref: 'ghcr.io/pistonprotection/gateway:latest'
        format: 'sarif'
        output: 'trivy-results.sarif'
        severity: 'CRITICAL,HIGH'
    - name: Upload Trivy scan results
      uses: github/codeql-action/upload-sarif@v2
      with:
        sarif_file: 'trivy-results.sarif'
```

---

## Maintenance Procedures

### Upgrade Procedures

#### Pre-Upgrade Checklist

```bash
#!/bin/bash
# pre-upgrade-check.sh

echo "=== Pre-Upgrade Checklist ==="

# 1. Backup current state
echo "1. Creating backup..."
velero backup create pre-upgrade-$(date +%Y%m%d) \
  --include-namespaces pistonprotection

# 2. Check cluster health
echo "2. Checking cluster health..."
kubectl get pods -n pistonprotection | grep -v Running

# 3. Verify current version
echo "3. Current version:"
helm list -n pistonprotection

# 4. Check for pending migrations
echo "4. Checking migrations..."
kubectl exec -n pistonprotection deploy/pistonprotection-gateway -- \
  ./migrate status

# 5. Verify backup completed
echo "5. Backup status:"
velero backup describe pre-upgrade-$(date +%Y%m%d)

echo "=== Pre-Upgrade Check Complete ==="
```

#### Upgrade Process

```bash
# 1. Update Helm repo
helm repo update pistonprotection

# 2. Review changes
helm diff upgrade pistonprotection pistonprotection/pistonprotection \
  -n pistonprotection \
  -f values.yaml \
  --version 2.0.0

# 3. Perform upgrade
helm upgrade pistonprotection pistonprotection/pistonprotection \
  -n pistonprotection \
  -f values.yaml \
  --version 2.0.0 \
  --wait \
  --timeout 10m

# 4. Verify upgrade
kubectl rollout status deployment/pistonprotection-gateway -n pistonprotection
kubectl rollout status daemonset/pistonprotection-worker -n pistonprotection

# 5. Run smoke tests
./scripts/smoke-test.sh
```

#### Rollback Procedure

```bash
# Quick rollback using Helm
helm rollback pistonprotection -n pistonprotection

# Or restore from Velero backup
velero restore create --from-backup pre-upgrade-20240115
```

### Certificate Management

#### Cert-Manager Integration

```yaml
apiVersion: cert-manager.io/v1
kind: Certificate
metadata:
  name: pistonprotection-tls
  namespace: pistonprotection
spec:
  secretName: pistonprotection-tls
  issuerRef:
    name: letsencrypt-prod
    kind: ClusterIssuer
  dnsNames:
  - api.pistonprotection.example.com
  - dashboard.pistonprotection.example.com
```

#### Certificate Monitoring

```bash
# Check certificate expiry
kubectl get certificates -n pistonprotection

# Detailed certificate info
kubectl describe certificate pistonprotection-tls -n pistonprotection

# Manual renewal
kubectl delete certificate pistonprotection-tls -n pistonprotection
kubectl apply -f certificate.yaml
```

### Log Rotation

```yaml
# Fluentd ConfigMap for log rotation
apiVersion: v1
kind: ConfigMap
metadata:
  name: fluentd-config
  namespace: logging
data:
  fluent.conf: |
    <source>
      @type tail
      path /var/log/containers/pistonprotection-*.log
      pos_file /var/log/fluentd-pistonprotection.log.pos
      tag pistonprotection.*
      read_from_head true
      <parse>
        @type json
      </parse>
    </source>

    <match pistonprotection.**>
      @type s3
      s3_bucket pistonprotection-logs
      s3_region us-east-1
      path logs/%Y/%m/%d/
      <buffer time>
        @type file
        path /var/log/fluentd-buffer
        timekey 1h
        timekey_wait 10m
        chunk_limit_size 256m
      </buffer>
    </match>
```

---

## Related Documentation

- [Installation Guide](installation.md) - Deployment instructions
- [Configuration Reference](configuration.md) - All configuration options
- [Architecture Overview](architecture.md) - System design
- [API Documentation](api.md) - API reference
- [Development Guide](development.md) - Contributing and development
