# Development Guide

This guide covers local development setup, building from source, testing, and contributing to PistonProtection.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Local Development Setup](#local-development-setup)
- [Project Structure](#project-structure)
- [Building from Source](#building-from-source)
- [Running Locally](#running-locally)
- [Testing](#testing)
- [Contributing](#contributing)
- [Code Style Guide](#code-style-guide)

---

## Prerequisites

### Required Software

| Software | Version | Purpose |
|----------|---------|---------|
| Rust | 1.75+ stable, nightly | Backend services |
| Node.js | 20+ | Frontend |
| pnpm | 9+ | Package manager |
| Docker | 24+ | Containerization |
| Kubernetes | 1.27+ | Orchestration |
| Helm | 3.12+ | Package deployment |
| Go | 1.21+ | Some tooling |

### Installation

#### Rust Toolchain

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install stable and nightly toolchains
rustup install stable
rustup install nightly

# Install required components
rustup component add rustfmt clippy rust-src
rustup component add rust-src --toolchain nightly

# Install eBPF tooling
cargo install bpf-linker
cargo install cargo-generate
```

#### Node.js and pnpm

```bash
# Install Node.js (using nvm recommended)
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
nvm install 20
nvm use 20

# Install pnpm
npm install -g pnpm
```

#### Docker and Kubernetes

```bash
# Install Docker
curl -fsSL https://get.docker.com | sh

# Install kubectl
curl -LO "https://dl.k8s.io/release/$(curl -L -s https://dl.k8s.io/release/stable.txt)/bin/linux/amd64/kubectl"
sudo install -o root -g root -m 0755 kubectl /usr/local/bin/kubectl

# Install Helm
curl https://raw.githubusercontent.com/helm/helm/main/scripts/get-helm-3 | bash

# Install k3d (local Kubernetes)
curl -s https://raw.githubusercontent.com/k3d-io/k3d/main/install.sh | bash
```

---

## Local Development Setup

### Clone Repository

```bash
git clone https://github.com/pistonprotection/app.git
cd pistonprotection
```

### Install Dependencies

```bash
# Install Rust dependencies
make deps

# Install frontend dependencies
cd frontend && pnpm install && cd ..
```

### Start Local Kubernetes Cluster

```bash
# Create k3d cluster
k3d cluster create pistonprotection \
    --api-port 6443 \
    --port "8080:80@loadbalancer" \
    --port "3000:3000@loadbalancer" \
    --agents 2

# Verify cluster
kubectl cluster-info
```

### Install Local Dependencies

```bash
# Install PostgreSQL
helm install postgresql oci://registry-1.docker.io/bitnamicharts/postgresql \
    --namespace pistonprotection \
    --create-namespace \
    --set auth.postgresPassword=devpassword \
    --set auth.database=pistonprotection

# Install Redis
helm install redis oci://registry-1.docker.io/bitnamicharts/redis \
    --namespace pistonprotection \
    --set auth.password=devpassword \
    --set architecture=standalone
```

### Configure Environment

Create `.env` file in project root:

```bash
# Database
DATABASE_URL=postgres://postgres:devpassword@localhost:5432/pistonprotection

# Redis
REDIS_URL=redis://:devpassword@localhost:6379

# Logging
LOG_LEVEL=debug
RUST_LOG=debug

# JWT
JWT_SECRET=dev-secret-key-change-in-production

# Service ports
GATEWAY_PORT=8080
GATEWAY_GRPC_PORT=9090
FRONTEND_PORT=3000
```

### Start Development Environment

```bash
# Using docker-compose (recommended)
make dev

# Or start services individually
make dev-up
```

---

## Project Structure

```
pistonprotection/
├── frontend/                 # Dashboard (TanStack Start + shadcn/ui)
│   ├── app/                  # Application code
│   │   ├── routes/           # File-based routes
│   │   ├── components/       # React components
│   │   ├── hooks/            # Custom hooks
│   │   └── lib/              # Utilities
│   ├── public/               # Static assets
│   └── package.json
│
├── services/                 # Rust backend services
│   ├── common/               # Shared library
│   │   └── src/
│   │       ├── config.rs     # Configuration
│   │       ├── db.rs         # Database
│   │       ├── error.rs      # Error types
│   │       └── telemetry.rs  # Observability
│   │
│   ├── gateway/              # API Gateway
│   │   └── src/
│   │       ├── main.rs
│   │       ├── handlers/     # HTTP/gRPC handlers
│   │       ├── middleware/   # Auth, logging, etc.
│   │       └── services/     # Business logic
│   │
│   ├── worker/               # XDP/eBPF worker
│   │   └── src/
│   │       ├── main.rs
│   │       ├── ebpf/         # eBPF program interface
│   │       ├── handlers/     # Control plane handlers
│   │       └── protocol/     # Protocol parsers
│   │
│   ├── config-mgr/           # Configuration manager
│   ├── metrics/              # Metrics aggregator
│   └── auth/                 # Authentication service
│
├── ebpf/                     # eBPF/XDP programs
│   └── src/
│       ├── xdp_filter.rs     # Main XDP filter
│       ├── xdp_ratelimit.rs  # Rate limiting
│       ├── xdp_minecraft.rs  # Minecraft protocol
│       └── xdp_quic.rs       # QUIC protocol
│
├── operator/                 # Kubernetes operator
│   └── src/
│       ├── main.rs
│       ├── crd.rs            # CRD definitions
│       └── controllers/      # Reconcilers
│
├── proto/                    # Protobuf definitions
│   ├── common.proto
│   ├── backend.proto
│   ├── filter.proto
│   ├── metrics.proto
│   └── auth.proto
│
├── charts/                   # Helm charts
│   └── pistonprotection/
│       ├── Chart.yaml
│       ├── values.yaml
│       └── templates/
│
├── docker/                   # Dockerfiles
│   ├── gateway/
│   ├── worker/
│   └── frontend/
│
├── docs/                     # Documentation
├── tests/                    # Integration tests
├── scripts/                  # Build and utility scripts
├── Makefile                  # Build automation
└── docker-compose.yml        # Local development
```

---

## Building from Source

### Build All Components

```bash
# Debug build
make build

# Release build
make build-release
```

### Build Individual Components

```bash
# Build Rust services
make build-services

# Build eBPF programs
make build-ebpf

# Build frontend
make build-frontend

# Build operator
make build-operator
```

### Build Docker Images

```bash
# Build all images
make docker-build

# Build specific image
make docker-build-gateway
make docker-build-worker
make docker-build-frontend
```

### Build eBPF Programs

eBPF programs require special handling:

```bash
# Set up eBPF build environment
cd ebpf

# Build with nightly Rust
cargo +nightly build \
    --target bpfel-unknown-none \
    -Z build-std=core \
    --release

# Generated files in target/bpfel-unknown-none/release/
```

**Requirements for eBPF development:**
- Nightly Rust toolchain
- `rust-src` component
- `bpf-linker`
- Linux kernel headers (for local testing)

---

## Running Locally

### Using Docker Compose

```bash
# Start all services
docker-compose up -d

# View logs
docker-compose logs -f

# Stop services
docker-compose down
```

### Running Services Individually

#### Gateway Service

```bash
cd services/gateway
cargo run
# Listening on http://localhost:8080 (HTTP)
# Listening on http://localhost:9090 (gRPC)
```

#### Frontend

```bash
cd frontend
pnpm dev
# Dashboard at http://localhost:3000
```

#### Worker (requires root/capabilities)

```bash
cd services/worker
sudo cargo run
```

#### Config Manager

```bash
cd services/config-mgr
cargo run
```

### Accessing Services

| Service | URL |
|---------|-----|
| Dashboard | http://localhost:3000 |
| API Gateway | http://localhost:8080 |
| gRPC Gateway | localhost:9090 |
| Prometheus | http://localhost:9090 |
| Grafana | http://localhost:3001 |

---

## Testing

### Run All Tests

```bash
make test
```

### Unit Tests

```bash
# All unit tests
make test-unit

# Specific service
cd services/gateway && cargo test

# With coverage
make test-coverage
```

### Integration Tests

```bash
# Start test environment
./scripts/test-env.sh up

# Run integration tests
make test-integration

# Cleanup
./scripts/test-env.sh down
```

### eBPF Tests

```bash
# eBPF tests require root privileges
cd ebpf
sudo cargo test
```

### Frontend Tests

```bash
cd frontend

# Unit tests
pnpm test

# E2E tests
pnpm test:e2e
```

### Linting

```bash
# All linters
make lint

# Rust only
make lint-rust

# Frontend only
make lint-frontend

# Helm chart
make lint-helm
```

### Formatting

```bash
# Format all code
make format

# Check formatting
make format-check
```

---

## Contributing

### Getting Started

1. **Fork the repository**
2. **Create a feature branch**
   ```bash
   git checkout -b feature/my-feature
   ```
3. **Make your changes**
4. **Run tests and lints**
   ```bash
   make lint test
   ```
5. **Commit with conventional commits**
6. **Push and create a pull request**

### Branch Naming

| Prefix | Purpose |
|--------|---------|
| `feature/` | New features |
| `fix/` | Bug fixes |
| `docs/` | Documentation |
| `refactor/` | Code refactoring |
| `test/` | Test additions |
| `chore/` | Maintenance |

### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

**Types:**
- `feat` - New feature
- `fix` - Bug fix
- `docs` - Documentation
- `style` - Formatting
- `refactor` - Code restructuring
- `test` - Tests
- `chore` - Maintenance

**Examples:**

```bash
feat(worker): add QUIC protocol support

Implemented QUIC protocol detection and filtering in the XDP layer.
Supports connection ID validation and version negotiation.

Closes #123
```

```bash
fix(gateway): handle concurrent API requests

Fixed race condition in rate limiter that caused
incorrect rate limit enforcement under high load.

Fixes #456
```

### Pull Request Process

1. **Ensure CI passes**
2. **Update documentation** if needed
3. **Add tests** for new functionality
4. **Request review** from maintainers

**PR Template:**

```markdown
## Description
Brief description of changes.

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
- [ ] Unit tests added/updated
- [ ] Integration tests added/updated
- [ ] Manual testing performed

## Checklist
- [ ] Code follows style guidelines
- [ ] Self-reviewed code
- [ ] Comments added for complex logic
- [ ] Documentation updated
- [ ] No new warnings
```

---

## Code Style Guide

### Rust

Follow the [Rust Style Guide](https://doc.rust-lang.org/stable/style-guide/).

```bash
# Format code
cargo fmt

# Lint code
cargo clippy --all-targets --all-features -- -D warnings
```

**Key conventions:**

```rust
// Use meaningful names
fn calculate_rate_limit() -> RateLimitResult { ... }

// Document public APIs
/// Calculates the token bucket rate limit for the given IP.
///
/// # Arguments
/// * `ip` - The source IP address
/// * `config` - Rate limit configuration
///
/// # Returns
/// The rate limit decision (allow, drop, or limit)
pub fn calculate_rate_limit(
    ip: IpAddr,
    config: &RateLimitConfig,
) -> RateLimitResult {
    ...
}

// Use Result for fallible operations
fn parse_packet(data: &[u8]) -> Result<Packet, ParseError> {
    ...
}

// Prefer iterators over manual loops
let valid_ips: Vec<_> = ips
    .iter()
    .filter(|ip| is_valid(ip))
    .collect();

// Use structured logging
tracing::info!(
    ip = %source_ip,
    action = "drop",
    reason = "rate_limit",
    "Dropping packet"
);
```

### TypeScript/React

```bash
# Lint
pnpm lint

# Format
pnpm format
```

**Key conventions:**

```typescript
// Use TypeScript types
interface BackendProps {
  id: string;
  name: string;
  status: BackendStatus;
}

// Use functional components
function BackendCard({ id, name, status }: BackendProps) {
  return (
    <Card>
      <CardHeader>
        <CardTitle>{name}</CardTitle>
      </CardHeader>
      <CardContent>
        <StatusBadge status={status} />
      </CardContent>
    </Card>
  );
}

// Use hooks for state
const [backends, setBackends] = useState<Backend[]>([]);

// Use TanStack Query for data fetching
const { data, isLoading, error } = useQuery({
  queryKey: ['backends'],
  queryFn: fetchBackends,
});
```

### Protocol Buffers

```protobuf
// Use descriptive names
message BackendStatus {
  HealthStatus health = 1;
  uint32 healthy_origins = 2;
  uint32 total_origins = 3;
}

// Document fields
message FilterRule {
  // Unique identifier for the rule
  string id = 1;

  // Human-readable name
  string name = 2;

  // Rule priority (lower = higher priority)
  uint32 priority = 3;
}

// Use enums for fixed values
enum ProtectionLevel {
  PROTECTION_LEVEL_UNSPECIFIED = 0;
  PROTECTION_LEVEL_OFF = 1;
  PROTECTION_LEVEL_LOW = 2;
  PROTECTION_LEVEL_MEDIUM = 3;
  PROTECTION_LEVEL_HIGH = 4;
  PROTECTION_LEVEL_UNDER_ATTACK = 5;
}
```

### eBPF/XDP

```rust
// Keep eBPF programs simple and bounded
#[xdp]
pub fn xdp_filter(ctx: XdpContext) -> u32 {
    match try_filter(&ctx) {
        Ok(action) => action,
        Err(_) => xdp_action::XDP_PASS,
    }
}

// Use verifier-friendly patterns
fn try_filter(ctx: &XdpContext) -> Result<u32, ()> {
    // Check packet bounds explicitly
    let eth = ptr_at::<EthHdr>(ctx, 0)?;

    // Use bounded loops
    for i in 0..MAX_ITERATIONS {
        // ...
    }

    Ok(xdp_action::XDP_PASS)
}

// Document map usage
/// Rate limit map: IP -> (tokens, last_update)
#[map]
static RATE_LIMIT_MAP: HashMap<u32, RateLimitEntry> =
    HashMap::with_max_entries(100000, 0);
```

---

## Development Tips

### Debugging Rust Services

```bash
# Enable debug logging
RUST_LOG=debug cargo run

# Use tracing-subscriber for structured logs
RUST_LOG=pistonprotection_gateway=trace cargo run
```

### Debugging eBPF Programs

```bash
# View eBPF program output
sudo cat /sys/kernel/debug/tracing/trace_pipe

# Check loaded programs
sudo bpftool prog list

# Inspect maps
sudo bpftool map list
sudo bpftool map dump id <map_id>
```

### Database Migrations

```bash
# Run migrations
sqlx migrate run

# Create new migration
sqlx migrate add <name>

# Revert migration
sqlx migrate revert
```

### Hot Reloading

```bash
# Rust services
cargo watch -x run

# Frontend
pnpm dev  # Has built-in HMR
```

---

## Related Documentation

- [Installation Guide](installation.md) - Deployment instructions
- [Architecture Overview](architecture.md) - System design
- [API Documentation](api.md) - API reference
- [Operations Guide](operations.md) - Production operations
