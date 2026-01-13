# PistonProtection

[![Build Status](https://img.shields.io/github/actions/workflow/status/pistonprotection/pistonprotection/ci.yml?branch=main&style=flat-square)](https://github.com/pistonprotection/app/actions)
[![Release](https://img.shields.io/github/v/release/pistonprotection/pistonprotection?style=flat-square)](https://github.com/pistonprotection/app/releases)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue?style=flat-square)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75+-orange?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![Kubernetes](https://img.shields.io/badge/kubernetes-1.27+-326CE5?style=flat-square&logo=kubernetes&logoColor=white)](https://kubernetes.io/)
[![Documentation](https://img.shields.io/badge/docs-latest-brightgreen?style=flat-square)](https://docs.pistonprotection.io)

**Enterprise-Grade DDoS Protection Platform**

PistonProtection is a comprehensive, self-hostable DDoS protection solution built on modern cloud-native technologies. It provides advanced Layer 4 and Layer 7 filtering using eBPF/XDP for line-rate packet processing, capable of mitigating attacks at 100+ Gbps per node.

---

## Highlights

- **Line-Rate Filtering** - eBPF/XDP processes packets at NIC driver level before kernel network stack
- **Multi-Protocol Support** - HTTP/1.1, HTTP/2, HTTP/3 (QUIC), Minecraft Java/Bedrock, generic TCP/UDP
- **Kubernetes Native** - Deploys as Helm chart with custom CRDs and operator for seamless integration
- **Real-Time Dashboard** - Modern React UI with live attack visualization and traffic analytics
- **Full API** - REST and gRPC APIs for complete automation and integration

---

## Features

### Protection Capabilities
- **Layer 4 Protection**: TCP, UDP, QUIC flood mitigation
- **Layer 7 Protocol Filtering**:
  - HTTP/1.1, HTTP/2, HTTP/3 (QUIC)
  - Minecraft Java Edition
  - Minecraft Bedrock Edition (RakNet)
  - Generic TCP/UDP applications
- **Adaptive Rate Limiting**: Per-IP, per-subnet, and global rate limiting
- **GeoIP Blocking**: Block or allow traffic by country
- **Bot Detection**: Advanced challenge-response for L7 protocols

### Infrastructure
- **eBPF/XDP Filtering**: Line-rate packet processing at the NIC driver level
- **Kubernetes Native**: Built on Cilium with custom operators
- **Horizontal Scaling**: Automatic worker node scaling based on traffic
- **Multi-Region**: Support for anycast routing and global load balancing

### Management Dashboard
- **Real-Time Metrics**: Live attack visualization and traffic analytics
- **Configuration UI**: Easy-to-use protection rule management
- **Multi-Tenant**: Organization-based access control
- **API Access**: Full REST and gRPC API for automation

### Observability
- **Prometheus**: Metrics collection and alerting
- **Grafana**: Pre-built dashboards for traffic analysis
- **Loki**: Centralized log aggregation

## Architecture

```
┌──────────────────────────────────────────────────────────────────┐
│                        Internet Traffic                          │
└─────────────────────────────────┬────────────────────────────────┘
                                  │
                                  ▼
┌──────────────────────────────────────────────────────────────────┐
│                     Anycast Edge Network                         │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐              │
│  │   Worker    │  │   Worker    │  │   Worker    │    ...       │
│  │   Node 1    │  │   Node 2    │  │   Node 3    │              │
│  │  (XDP/eBPF) │  │  (XDP/eBPF) │  │  (XDP/eBPF) │              │
│  └─────────────┘  └─────────────┘  └─────────────┘              │
└─────────────────────────────────┬────────────────────────────────┘
                                  │
                                  ▼
┌──────────────────────────────────────────────────────────────────┐
│                    Control Plane (Kubernetes)                    │
│                                                                  │
│  ┌────────────────┐  ┌────────────────┐  ┌────────────────┐     │
│  │    Gateway     │  │  Config Mgr    │  │   Metrics      │     │
│  │    Service     │  │    Service     │  │   Collector    │     │
│  └────────────────┘  └────────────────┘  └────────────────┘     │
│                                                                  │
│  ┌────────────────┐  ┌────────────────┐  ┌────────────────┐     │
│  │   PostgreSQL   │  │     Redis      │  │   Prometheus   │     │
│  └────────────────┘  └────────────────┘  └────────────────┘     │
│                                                                  │
│  ┌────────────────┐  ┌────────────────┐  ┌────────────────┐     │
│  │    Grafana     │  │     Loki       │  │   Dashboard    │     │
│  └────────────────┘  └────────────────┘  └────────────────┘     │
└──────────────────────────────────────────────────────────────────┘
```

## Project Structure

```
pistonprotection/
├── frontend/           # Dashboard (TanStack Start + shadcn/ui)
├── services/           # Rust backend services
│   ├── gateway/        # API Gateway and proxy
│   ├── config-mgr/     # Configuration management
│   ├── metrics/        # Metrics collection and aggregation
│   ├── auth/           # Authentication service
│   └── common/         # Shared libraries
├── ebpf/               # eBPF/XDP programs
│   ├── filters/        # Protocol-specific filters
│   ├── maps/           # eBPF maps definitions
│   └── loader/         # Userspace loader
├── operator/           # Kubernetes operator
├── proto/              # Protobuf definitions
├── charts/             # Helm charts
├── docs/               # Documentation
└── deploy/             # Deployment configurations
```

## Quick Start

### Prerequisites

- Kubernetes 1.27+ (k0s, k3s, EKS, GKE, or AKS)
- Helm 3.12+
- Cilium CNI (required for eBPF functionality)

### 1. Install Cilium

```bash
cilium install --version "1.16.0" \
  --set kubeProxyReplacement=true \
  --set hubble.enabled=true \
  --set hubble.relay.enabled=true
```

### 2. Install PistonProtection

```bash
# Add Helm repository
helm repo add pistonprotection https://charts.pistonprotection.io
helm repo update

# Install with default configuration
helm install pistonprotection pistonprotection/pistonprotection \
  --namespace pistonprotection \
  --create-namespace

# Or install with custom values
helm install pistonprotection pistonprotection/pistonprotection \
  --namespace pistonprotection \
  --create-namespace \
  -f values.yaml
```

### 3. Access the Dashboard

```bash
# Port forward to access the dashboard
kubectl port-forward -n pistonprotection svc/pistonprotection-frontend 3000:3000

# Open http://localhost:3000 in your browser
```

### 4. Create Your First Backend

```bash
# Via API
curl -X POST http://localhost:8080/v1/backends \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "name": "my-web-app",
    "domain": "app.example.com",
    "origins": [{"address": "10.0.1.10", "port": 443}],
    "protocol": "HTTPS",
    "protection_level": "MEDIUM"
  }'

# Or via kubectl
kubectl apply -f - <<EOF
apiVersion: pistonprotection.io/v1alpha1
kind: Backend
metadata:
  name: my-web-app
spec:
  domain: app.example.com
  origins:
    - address: 10.0.1.10
      port: 443
  protectionLevel: medium
EOF
```

For detailed installation instructions, see the [Installation Guide](docs/installation.md).

---

## Documentation

| Document | Description |
|----------|-------------|
| [Installation Guide](docs/installation.md) | Deployment on various platforms (AWS, GCP, Azure, self-hosted) |
| [Configuration Reference](docs/configuration.md) | All Helm values and environment variables |
| [API Documentation](docs/api.md) | REST and gRPC API reference |
| [Protocol Filters](docs/filters.md) | TCP, UDP, HTTP, QUIC, Minecraft filtering |
| [Architecture Overview](docs/architecture.md) | System design and component details |
| [Development Guide](docs/development.md) | Local setup, building, testing, contributing |
| [Operations Guide](docs/operations.md) | Monitoring, troubleshooting, scaling, backup |
| [User Guide](docs/user-guide.md) | Dashboard walkthrough and feature usage |

---

## Requirements

### Minimum Requirements

| Component | Requirement |
|-----------|-------------|
| Kubernetes | 1.27+ |
| Cilium | 1.14+ |
| Helm | 3.12+ |
| Worker Nodes | 2+ cores, 2GB RAM |
| Control Plane | 4+ cores, 8GB RAM |

### Supported Platforms

| Platform | Status | Notes |
|----------|--------|-------|
| AWS EKS | Supported | With Cilium CNI |
| Google GKE | Supported | Dataplane v2 recommended |
| Azure AKS | Supported | With Cilium CNI |
| k0s | Supported | Recommended for self-hosted |
| k3s | Supported | Disable Traefik/Flannel |
| Bare Metal | Supported | Full XDP performance |

---

## Community and Support

- **Documentation**: [docs.pistonprotection.io](https://docs.pistonprotection.io)
- **Discord Community**: [discord.gg/pistonprotection](https://discord.gg/pistonprotection)
- **GitHub Issues**: [Report bugs or request features](https://github.com/pistonprotection/app/issues)
- **GitHub Discussions**: [Ask questions and share ideas](https://github.com/pistonprotection/app/discussions)

### Enterprise Support

For enterprise support options including SLAs, dedicated support, and custom development:
- Email: enterprise@pistonprotection.io
- Website: [pistonprotection.io/enterprise](https://pistonprotection.io/enterprise)

---

## Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for:

- Development environment setup
- Code style guidelines
- Pull request process
- Issue reporting

```bash
# Quick start for development
git clone https://github.com/pistonprotection/app.git
cd pistonprotection
make deps
make dev
```

---

## License

Apache License 2.0 - See [LICENSE](LICENSE) for details.

---

## Acknowledgments

PistonProtection is built on the shoulders of giants:

- [aya](https://github.com/aya-rs/aya) - eBPF library for Rust
- [Cilium](https://cilium.io/) - eBPF-based networking
- [TanStack](https://tanstack.com/) - Modern React tooling
- [Tokio](https://tokio.rs/) - Async runtime for Rust
