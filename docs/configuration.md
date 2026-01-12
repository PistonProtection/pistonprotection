# Configuration Reference

This document provides a comprehensive reference for all PistonProtection configuration options, including Helm values, environment variables, and service configurations.

## Table of Contents

- [Helm Values Reference](#helm-values-reference)
  - [Global Settings](#global-settings)
  - [Gateway Service](#gateway-service)
  - [Worker Service](#worker-service)
  - [Config Manager](#config-manager)
  - [Metrics Service](#metrics-service)
  - [Operator](#operator)
  - [Frontend](#frontend)
  - [Ingress](#ingress)
  - [PostgreSQL](#postgresql)
  - [Redis](#redis)
  - [Observability](#observability)
  - [Protection Settings](#protection-settings)
- [Environment Variables](#environment-variables)
- [Service Configurations](#service-configurations)
- [Custom Resource Definitions](#custom-resource-definitions)

---

## Helm Values Reference

### Global Settings

Global configuration applied to all components.

```yaml
global:
  # Image pull secrets for private registries
  imagePullSecrets: []
  #  - name: registry-credentials

  # Default storage class for persistent volumes
  storageClass: ""

  # Common labels applied to all resources
  labels: {}

  # Common annotations applied to all resources
  annotations: {}
```

| Parameter | Description | Default |
|-----------|-------------|---------|
| `global.imagePullSecrets` | List of image pull secret names | `[]` |
| `global.storageClass` | Default StorageClass for PVCs | `""` |
| `global.labels` | Labels applied to all resources | `{}` |
| `global.annotations` | Annotations applied to all resources | `{}` |

---

### Gateway Service

The API gateway handles REST/gRPC requests and proxies traffic.

```yaml
gateway:
  # Enable/disable the gateway
  enabled: true

  # Number of replicas
  replicaCount: 2

  # Container image configuration
  image:
    repository: pistonprotection/gateway
    tag: ""  # Defaults to Chart appVersion
    pullPolicy: IfNotPresent

  # Service configuration
  service:
    type: ClusterIP
    port: 8080
    grpcPort: 9090
    annotations: {}

  # Resource limits and requests
  resources:
    requests:
      cpu: 100m
      memory: 128Mi
    limits:
      cpu: 500m
      memory: 512Mi

  # Horizontal Pod Autoscaler
  autoscaling:
    enabled: false
    minReplicas: 2
    maxReplicas: 10
    targetCPUUtilization: 80
    targetMemoryUtilization: 80

  # Pod scheduling
  nodeSelector: {}
  tolerations: []
  affinity: {}

  # Additional environment variables
  extraEnv: []
  #  - name: LOG_LEVEL
  #    value: "debug"

  # Additional volumes and mounts
  extraVolumes: []
  extraVolumeMounts: []

  # Pod disruption budget
  podDisruptionBudget:
    enabled: false
    minAvailable: 1
    # maxUnavailable: 1

  # Health check configuration
  livenessProbe:
    enabled: true
    initialDelaySeconds: 10
    periodSeconds: 10
    timeoutSeconds: 5
    failureThreshold: 3

  readinessProbe:
    enabled: true
    initialDelaySeconds: 5
    periodSeconds: 5
    timeoutSeconds: 3
    failureThreshold: 3
```

| Parameter | Description | Default |
|-----------|-------------|---------|
| `gateway.enabled` | Enable gateway service | `true` |
| `gateway.replicaCount` | Number of gateway replicas | `2` |
| `gateway.image.repository` | Gateway image repository | `pistonprotection/gateway` |
| `gateway.image.tag` | Gateway image tag | `""` |
| `gateway.image.pullPolicy` | Image pull policy | `IfNotPresent` |
| `gateway.service.type` | Kubernetes service type | `ClusterIP` |
| `gateway.service.port` | HTTP port | `8080` |
| `gateway.service.grpcPort` | gRPC port | `9090` |
| `gateway.autoscaling.enabled` | Enable HPA | `false` |
| `gateway.autoscaling.minReplicas` | Minimum replicas | `2` |
| `gateway.autoscaling.maxReplicas` | Maximum replicas | `10` |

---

### Worker Service

The worker service runs on each node and performs eBPF/XDP packet filtering.

```yaml
worker:
  # Enable/disable the worker
  enabled: true

  # Deployment type (DaemonSet for network-level filtering)
  kind: DaemonSet

  # Container image configuration
  image:
    repository: pistonprotection/worker
    tag: ""
    pullPolicy: IfNotPresent

  # Host network is REQUIRED for XDP
  hostNetwork: true
  dnsPolicy: ClusterFirstWithHostNet

  # Security context - privileged mode REQUIRED for eBPF
  securityContext:
    privileged: true
    capabilities:
      add:
        - NET_ADMIN
        - SYS_ADMIN
        - BPF

  # Resource limits and requests
  resources:
    requests:
      cpu: 100m
      memory: 128Mi
    limits:
      cpu: 1000m
      memory: 512Mi

  # Node selector for worker nodes
  nodeSelector:
    pistonprotection.io/worker: "true"

  # Tolerations for worker taints
  tolerations:
    - key: "pistonprotection.io/worker"
      operator: "Exists"
      effect: "NoSchedule"

  # Volumes for eBPF maps
  volumes:
    - name: bpf-maps
      hostPath:
        path: /sys/fs/bpf
        type: DirectoryOrCreate
    - name: kernel-debug
      hostPath:
        path: /sys/kernel/debug
        type: Directory
    - name: cgroup
      hostPath:
        path: /sys/fs/cgroup
        type: Directory

  volumeMounts:
    - name: bpf-maps
      mountPath: /sys/fs/bpf
    - name: kernel-debug
      mountPath: /sys/kernel/debug
    - name: cgroup
      mountPath: /sys/fs/cgroup

  # XDP configuration
  xdp:
    # Preferred XDP mode: native, driver, generic
    mode: native
    # Network interfaces to attach (empty = auto-detect)
    interfaces: []
    # XDP flags
    flags: []

  # Additional environment variables
  extraEnv: []
```

| Parameter | Description | Default |
|-----------|-------------|---------|
| `worker.enabled` | Enable worker service | `true` |
| `worker.kind` | Kubernetes workload type | `DaemonSet` |
| `worker.hostNetwork` | Use host network (required) | `true` |
| `worker.securityContext.privileged` | Run as privileged (required) | `true` |
| `worker.nodeSelector` | Node selector for workers | `{pistonprotection.io/worker: "true"}` |
| `worker.xdp.mode` | XDP attachment mode | `native` |
| `worker.xdp.interfaces` | Network interfaces to attach | `[]` (auto-detect) |

---

### Config Manager

Manages and distributes configuration to workers.

```yaml
configMgr:
  enabled: true
  replicaCount: 1

  image:
    repository: pistonprotection/config-mgr
    tag: ""
    pullPolicy: IfNotPresent

  service:
    type: ClusterIP
    port: 8080

  resources:
    requests:
      cpu: 50m
      memory: 64Mi
    limits:
      cpu: 200m
      memory: 256Mi

  # Configuration sync interval (seconds)
  syncInterval: 30

  # Configuration cache TTL (seconds)
  cacheTTL: 300

  nodeSelector: {}
  tolerations: []
  affinity: {}
```

| Parameter | Description | Default |
|-----------|-------------|---------|
| `configMgr.enabled` | Enable config manager | `true` |
| `configMgr.replicaCount` | Number of replicas | `1` |
| `configMgr.syncInterval` | Config sync interval | `30` |
| `configMgr.cacheTTL` | Cache time-to-live | `300` |

---

### Metrics Service

Collects and aggregates metrics from all components.

```yaml
metrics:
  enabled: true
  replicaCount: 1

  image:
    repository: pistonprotection/metrics
    tag: ""
    pullPolicy: IfNotPresent

  service:
    type: ClusterIP
    port: 8080

  resources:
    requests:
      cpu: 100m
      memory: 128Mi
    limits:
      cpu: 500m
      memory: 512Mi

  # Metrics retention
  retention:
    raw: 24h      # Raw metrics retention
    aggregated: 30d  # Aggregated metrics retention

  # Aggregation intervals
  aggregation:
    enabled: true
    intervals:
      - 1m
      - 5m
      - 1h
      - 1d

  nodeSelector: {}
  tolerations: []
  affinity: {}
```

| Parameter | Description | Default |
|-----------|-------------|---------|
| `metrics.enabled` | Enable metrics service | `true` |
| `metrics.retention.raw` | Raw metrics retention | `24h` |
| `metrics.retention.aggregated` | Aggregated retention | `30d` |
| `metrics.aggregation.enabled` | Enable aggregation | `true` |

---

### Operator

Kubernetes operator for managing custom resources.

```yaml
operator:
  enabled: true
  replicaCount: 1

  image:
    repository: pistonprotection/operator
    tag: ""
    pullPolicy: IfNotPresent

  resources:
    requests:
      cpu: 100m
      memory: 128Mi
    limits:
      cpu: 500m
      memory: 512Mi

  # Install Custom Resource Definitions
  installCRDs: true

  # Leader election for HA
  leaderElection:
    enabled: true
    leaseDuration: 15s
    renewDeadline: 10s
    retryPeriod: 2s

  # Reconciliation settings
  reconcile:
    interval: 60s
    maxConcurrent: 5

  nodeSelector: {}
  tolerations: []
  affinity: {}
```

| Parameter | Description | Default |
|-----------|-------------|---------|
| `operator.enabled` | Enable operator | `true` |
| `operator.installCRDs` | Install CRDs | `true` |
| `operator.leaderElection.enabled` | Enable leader election | `true` |
| `operator.reconcile.interval` | Reconciliation interval | `60s` |

---

### Frontend

Dashboard web application.

```yaml
frontend:
  enabled: true
  replicaCount: 2

  image:
    repository: pistonprotection/frontend
    tag: ""
    pullPolicy: IfNotPresent

  service:
    type: ClusterIP
    port: 3000

  resources:
    requests:
      cpu: 50m
      memory: 64Mi
    limits:
      cpu: 200m
      memory: 256Mi

  # Environment configuration
  config:
    # API endpoint (auto-configured if empty)
    apiUrl: ""
    # Enable analytics
    analytics: false

  nodeSelector: {}
  tolerations: []
  affinity: {}
```

| Parameter | Description | Default |
|-----------|-------------|---------|
| `frontend.enabled` | Enable frontend | `true` |
| `frontend.replicaCount` | Number of replicas | `2` |
| `frontend.service.port` | Service port | `3000` |
| `frontend.config.apiUrl` | API URL | `""` (auto) |

---

### Ingress

Ingress configuration for external access.

```yaml
ingress:
  enabled: false
  className: ""
  annotations: {}
    # kubernetes.io/ingress.class: nginx
    # cert-manager.io/cluster-issuer: letsencrypt-prod
    # nginx.ingress.kubernetes.io/ssl-redirect: "true"

  hosts:
    - host: pistonprotection.local
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

  tls: []
  #  - secretName: pistonprotection-tls
  #    hosts:
  #      - pistonprotection.local
```

| Parameter | Description | Default |
|-----------|-------------|---------|
| `ingress.enabled` | Enable ingress | `false` |
| `ingress.className` | Ingress class name | `""` |
| `ingress.hosts` | Ingress host configuration | See above |
| `ingress.tls` | TLS configuration | `[]` |

---

### PostgreSQL

Database configuration.

```yaml
postgresql:
  # Use built-in PostgreSQL
  enabled: true

  auth:
    # PostgreSQL admin password
    postgresPassword: ""
    # Application username
    username: pistonprotection
    # Application password
    password: ""
    # Database name
    database: pistonprotection

  primary:
    # Persistence configuration
    persistence:
      enabled: true
      size: 10Gi
      storageClass: ""

    # Resource limits
    resources:
      requests:
        cpu: 100m
        memory: 256Mi
      limits:
        cpu: 500m
        memory: 512Mi

  # External PostgreSQL (when postgresql.enabled=false)
  external:
    host: ""
    port: 5432
    database: pistonprotection
    username: ""
    password: ""
    # Use existing secret for password
    existingSecret: ""
    existingSecretKey: "password"
    # SSL mode: disable, require, verify-ca, verify-full
    sslMode: "require"
```

| Parameter | Description | Default |
|-----------|-------------|---------|
| `postgresql.enabled` | Use built-in PostgreSQL | `true` |
| `postgresql.auth.username` | Database username | `pistonprotection` |
| `postgresql.auth.database` | Database name | `pistonprotection` |
| `postgresql.primary.persistence.size` | PVC size | `10Gi` |
| `postgresql.external.host` | External database host | `""` |
| `postgresql.external.sslMode` | SSL mode | `require` |

---

### Redis

Cache configuration.

```yaml
redis:
  # Use built-in Redis
  enabled: true

  architecture: standalone  # or 'replication'

  auth:
    enabled: true
    password: ""

  master:
    persistence:
      enabled: true
      size: 1Gi

    resources:
      requests:
        cpu: 50m
        memory: 64Mi
      limits:
        cpu: 200m
        memory: 256Mi

  # External Redis (when redis.enabled=false)
  external:
    host: ""
    port: 6379
    password: ""
    existingSecret: ""
    existingSecretKey: "password"
    # TLS configuration
    tls:
      enabled: false
```

| Parameter | Description | Default |
|-----------|-------------|---------|
| `redis.enabled` | Use built-in Redis | `true` |
| `redis.architecture` | Redis architecture | `standalone` |
| `redis.auth.enabled` | Enable authentication | `true` |
| `redis.master.persistence.size` | PVC size | `1Gi` |

---

### Observability

Monitoring and logging configuration.

```yaml
observability:
  # Prometheus metrics
  prometheus:
    enabled: true
    # Create ServiceMonitor for Prometheus Operator
    serviceMonitor:
      enabled: false
      namespace: ""  # If empty, uses release namespace
      interval: 30s
      scrapeTimeout: 10s
      labels: {}
      honorLabels: false

  # Grafana dashboards
  grafana:
    enabled: false
    dashboards:
      enabled: true
      # Dashboard ConfigMap labels
      labels:
        grafana_dashboard: "1"
      annotations: {}

  # Loki logging
  loki:
    enabled: false
    # Loki endpoint
    endpoint: ""

  # OpenTelemetry
  opentelemetry:
    enabled: false
    endpoint: ""
    protocol: "grpc"  # or "http/protobuf"
    insecure: false

  # Tracing
  tracing:
    enabled: false
    samplingRate: 0.1  # 10% of requests
```

| Parameter | Description | Default |
|-----------|-------------|---------|
| `observability.prometheus.enabled` | Enable Prometheus metrics | `true` |
| `observability.prometheus.serviceMonitor.enabled` | Create ServiceMonitor | `false` |
| `observability.grafana.dashboards.enabled` | Create Grafana dashboards | `true` |
| `observability.tracing.samplingRate` | Tracing sample rate | `0.1` |

---

### Protection Settings

Default protection configuration.

```yaml
protection:
  # Default protection level (1-5)
  # 1: Minimal - Basic filtering only
  # 2: Low - Standard rate limiting
  # 3: Medium - Enhanced protection (default)
  # 4: High - Aggressive filtering
  # 5: Under Attack - Maximum protection
  defaultLevel: 3

  # Default rate limits
  rateLimit:
    # Packets per second per IP
    ppsPerIp: 1000
    # Burst size
    burst: 2000
    # Global packets per second limit
    globalPps: 1000000
    # Connections per second per IP
    connPerIp: 100

  # Protocol-specific settings
  protocols:
    # Minecraft protocol validation
    minecraft:
      enabled: true
      # Minimum supported protocol version
      minVersion: 47  # 1.8
      # Maximum supported protocol version
      maxVersion: 769  # 1.21.x
      # Validate handshake
      validateHandshake: true
      # Rate limit status pings per IP
      statusRateLimit: 10

    # QUIC protocol settings
    quic:
      enabled: true
      # Allowed QUIC versions
      allowedVersions: [1, 2]
      # Validate initial packets
      validateInitial: true

    # HTTP settings
    http:
      enabled: true
      # Maximum request body size (bytes)
      maxBodySize: 10485760  # 10MB
      # Maximum header size (bytes)
      maxHeaderSize: 8192
      # Enable HTTP/2
      http2Enabled: true
      # Enable HTTP/3 (QUIC)
      http3Enabled: true

    # TCP SYN cookies
    synCookies:
      enabled: true
      # SYN flood threshold (SYN/s)
      threshold: 10000

  # GeoIP settings
  geoip:
    enabled: true
    # GeoIP database update interval
    updateInterval: 24h
    # Default action for blocked countries: drop, challenge
    defaultAction: drop

  # Challenge settings
  challenge:
    # Default challenge type: javascript, captcha, pow
    type: javascript
    # Challenge difficulty (1-10)
    difficulty: 5
    # Challenge validity (seconds)
    validitySeconds: 3600
```

| Parameter | Description | Default |
|-----------|-------------|---------|
| `protection.defaultLevel` | Default protection level | `3` |
| `protection.rateLimit.ppsPerIp` | Packets/second per IP | `1000` |
| `protection.rateLimit.burst` | Burst size | `2000` |
| `protection.rateLimit.globalPps` | Global PPS limit | `1000000` |
| `protection.protocols.minecraft.enabled` | Enable Minecraft filtering | `true` |
| `protection.protocols.quic.enabled` | Enable QUIC filtering | `true` |
| `protection.geoip.enabled` | Enable GeoIP filtering | `true` |

---

## Environment Variables

### Gateway Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `LOG_LEVEL` | Logging level (trace, debug, info, warn, error) | `info` |
| `DATABASE_URL` | PostgreSQL connection URL | Auto-configured |
| `REDIS_URL` | Redis connection URL | Auto-configured |
| `JWT_SECRET` | JWT signing secret | Auto-generated |
| `API_PORT` | HTTP API port | `8080` |
| `GRPC_PORT` | gRPC port | `9090` |
| `METRICS_PORT` | Prometheus metrics port | `9100` |
| `CORS_ORIGINS` | Allowed CORS origins | `*` |
| `RATE_LIMIT_ENABLED` | Enable API rate limiting | `true` |
| `RATE_LIMIT_RPS` | Requests per second | `100` |

### Worker Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `LOG_LEVEL` | Logging level | `info` |
| `CONFIG_MGR_URL` | Config manager endpoint | Auto-configured |
| `XDP_MODE` | XDP attachment mode | `native` |
| `XDP_INTERFACES` | Interfaces to attach (comma-separated) | Auto-detect |
| `METRICS_PORT` | Prometheus metrics port | `9100` |
| `HEARTBEAT_INTERVAL` | Heartbeat interval (seconds) | `30` |
| `MAP_SIZE_CONNECTIONS` | Connection tracking map size | `1000000` |
| `MAP_SIZE_RATELIMIT` | Rate limit map size | `100000` |
| `MAP_SIZE_BLOCKED` | Blocked IP map size | `100000` |

### Config Manager Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `LOG_LEVEL` | Logging level | `info` |
| `DATABASE_URL` | PostgreSQL connection URL | Auto-configured |
| `REDIS_URL` | Redis connection URL | Auto-configured |
| `SYNC_INTERVAL` | Configuration sync interval | `30s` |
| `CACHE_TTL` | Cache TTL | `300s` |

---

## Service Configurations

### Gateway Configuration File

The gateway can be configured via a YAML file mounted at `/etc/pistonprotection/gateway.yaml`:

```yaml
# Gateway configuration
server:
  http:
    port: 8080
    readTimeout: 30s
    writeTimeout: 30s
    idleTimeout: 120s
  grpc:
    port: 9090
    maxRecvMsgSize: 16777216  # 16MB
    maxSendMsgSize: 16777216

database:
  host: postgresql
  port: 5432
  database: pistonprotection
  username: pistonprotection
  maxConnections: 100
  minConnections: 10
  connectionTimeout: 30s

redis:
  host: redis-master
  port: 6379
  database: 0
  poolSize: 50

auth:
  jwtSecret: ""  # From environment/secret
  accessTokenTTL: 15m
  refreshTokenTTL: 7d
  sessionTTL: 24h

rateLimit:
  enabled: true
  requestsPerSecond: 100
  burst: 200
  keyPrefix: "ratelimit:"

logging:
  level: info
  format: json  # or "text"
  output: stdout

metrics:
  enabled: true
  port: 9100
  path: /metrics
```

### Worker Configuration File

The worker can be configured via `/etc/pistonprotection/worker.yaml`:

```yaml
# Worker configuration
xdp:
  mode: native  # native, driver, generic
  interfaces: []  # Empty = auto-detect
  flags: []

ebpf:
  maps:
    connections:
      maxEntries: 1000000
    rateLimit:
      maxEntries: 100000
    blocked:
      maxEntries: 100000
    geoip:
      maxEntries: 500000

controlPlane:
  configMgrUrl: "http://config-mgr:8080"
  heartbeatInterval: 30s
  configPollInterval: 60s

logging:
  level: info
  format: json

metrics:
  enabled: true
  port: 9100
  path: /metrics
```

---

## Custom Resource Definitions

### Backend CRD

```yaml
apiVersion: pistonprotection.io/v1
kind: Backend
metadata:
  name: my-backend
  namespace: default
spec:
  # Backend type: http, https, tcp, udp, minecraft-java, minecraft-bedrock, quic
  type: minecraft-java

  # Origin servers
  origins:
    - name: primary
      address: 192.168.1.100
      port: 25565
      weight: 100
      priority: 1
      enabled: true
      settings:
        connectTimeout: 5s
        readTimeout: 30s
        writeTimeout: 30s
        maxConnections: 1000
        proxyProtocol: 2  # 0=disabled, 1=v1, 2=v2

  # Load balancing configuration
  loadBalancer:
    algorithm: round-robin  # round-robin, least-connections, random, ip-hash, weighted
    stickySessions:
      enabled: false
      cookieName: "PISTON_SESSION"
      ttl: 3600

  # Health check configuration
  healthCheck:
    enabled: true
    interval: 30s
    timeout: 10s
    healthyThreshold: 2
    unhealthyThreshold: 3
    # Protocol-specific check
    minecraft:
      queryStatus: true

  # Protection settings
  protection:
    enabled: true
    level: 3
    rateLimit:
      perIp:
        requestsPerSecond: 1000
        burst: 2000
      global:
        requestsPerSecond: 100000
    geoip:
      mode: block-list  # allow-list, block-list, disabled
      countries: ["CN", "RU"]
    challenge:
      enabled: true
      type: javascript
      difficulty: 5

  # Associated domains
  domains:
    - mc.example.com
    - play.example.com
```

### FilterRule CRD

```yaml
apiVersion: pistonprotection.io/v1
kind: FilterRule
metadata:
  name: block-known-botnets
  namespace: default
spec:
  # Target backend (optional, applies globally if not specified)
  backendRef:
    name: my-backend

  # Rule priority (lower = higher priority)
  priority: 100

  # Rule enabled
  enabled: true

  # Match conditions
  match:
    # Source IP matching
    sourceIps:
      - 185.220.101.0/24
      - 45.155.205.0/24
    sourceIpBlacklist: []

    # GeoIP matching
    sourceCountries: []
    sourceCountryBlacklist:
      - CN
      - RU

    # ASN matching
    sourceAsns:
      - AS12345

    # Destination matching
    destinationPorts:
      - start: 25565
        end: 25565

    # Protocol matching
    protocols:
      - TCP
    l7Protocols:
      - MINECRAFT_JAVA

    # L7 specific matching (optional)
    l7Match:
      minecraftJava:
        validateHandshake: true
        maxConnectionsPerIp: 5

  # Action to take
  action: DROP  # ALLOW, DROP, RATE_LIMIT, CHALLENGE, LOG

  # Rate limit config (if action is RATE_LIMIT)
  rateLimit:
    requestsPerSecond: 100
    burst: 200
    window: 60s
```

### DDoSProtection CRD

```yaml
apiVersion: pistonprotection.io/v1
kind: DDoSProtection
metadata:
  name: global-protection
  namespace: pistonprotection
spec:
  # Protection level
  level: 3

  # Auto-escalation settings
  autoEscalation:
    enabled: true
    # Escalate when attack PPS exceeds threshold
    ppsThreshold: 100000
    # Escalate when attack BPS exceeds threshold
    bpsThreshold: 1000000000  # 1 Gbps
    # De-escalation delay (seconds)
    cooldownPeriod: 300

  # Global rate limits
  globalRateLimit:
    pps: 1000000
    bps: 10000000000  # 10 Gbps

  # Per-IP rate limits
  perIpRateLimit:
    pps: 1000
    connections: 100

  # Emergency mode
  emergencyMode:
    enabled: false
    # Auto-enable when attack exceeds threshold
    autoEnable: true
    ppsThreshold: 1000000
    # Drop all new connections in emergency
    dropNewConnections: false
```

---

## Configuration Precedence

Configuration is applied in the following order (later overrides earlier):

1. Default Helm values
2. User-provided values.yaml
3. Environment variables
4. ConfigMap/Secret values
5. CRD specifications

---

## Related Documentation

- [Installation Guide](installation.md) - Initial setup and deployment
- [API Documentation](api.md) - REST and gRPC API reference
- [Operations Guide](operations.md) - Monitoring and maintenance
