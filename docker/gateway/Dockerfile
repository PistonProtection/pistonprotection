# syntax=docker/dockerfile:1.7

# =============================================================================
# PistonProtection Gateway Service
# Multi-stage build for optimal image size and security
# =============================================================================

ARG RUST_VERSION=1.85
ARG DEBIAN_VERSION=bookworm

# -----------------------------------------------------------------------------
# Stage 1: Chef - Dependency caching
# -----------------------------------------------------------------------------
FROM rust:${RUST_VERSION}-${DEBIAN_VERSION} AS chef

RUN cargo install cargo-chef --locked
WORKDIR /app

# -----------------------------------------------------------------------------
# Stage 2: Planner - Create recipe for dependencies
# -----------------------------------------------------------------------------
FROM chef AS planner

COPY services/Cargo.toml services/Cargo.lock ./
COPY services/common ./common
COPY services/proto ./proto
COPY services/gateway ./gateway
COPY services/config-mgr ./config-mgr
COPY services/metrics ./metrics
COPY services/worker ./worker
COPY services/auth ./auth
COPY proto ../proto

RUN cargo chef prepare --recipe-path recipe.json

# -----------------------------------------------------------------------------
# Stage 3: Builder - Build dependencies and application
# -----------------------------------------------------------------------------
FROM chef AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    protobuf-compiler \
    libprotobuf-dev \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Build dependencies (cached)
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Copy source and build application
COPY services/Cargo.toml services/Cargo.lock ./
COPY services/common ./common
COPY services/proto ./proto
COPY services/gateway ./gateway
COPY services/config-mgr ./config-mgr
COPY services/metrics ./metrics
COPY services/worker ./worker
COPY services/auth ./auth
COPY proto ../proto

# Build arguments for versioning
ARG VERSION=0.0.0
ARG GIT_COMMIT=unknown

ENV CARGO_BUILD_RUSTFLAGS="-C link-arg=-Wl,--compress-debug-sections=zlib"

RUN cargo build --release --package pistonprotection-gateway && \
    strip /app/target/release/gateway

# -----------------------------------------------------------------------------
# Stage 4: Runtime - Minimal production image
# -----------------------------------------------------------------------------
FROM debian:${DEBIAN_VERSION}-slim AS runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Create non-root user
RUN groupadd -r pistonprotection --gid=1000 && \
    useradd -r -g pistonprotection --uid=1000 -d /app -s /sbin/nologin pistonprotection

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/gateway /usr/local/bin/

# Set ownership
RUN chown -R pistonprotection:pistonprotection /app

# Metadata
ARG VERSION=0.0.0
ARG GIT_COMMIT=unknown

LABEL org.opencontainers.image.title="PistonProtection Gateway" \
      org.opencontainers.image.description="API Gateway for PistonProtection DDoS protection platform" \
      org.opencontainers.image.version="${VERSION}" \
      org.opencontainers.image.revision="${GIT_COMMIT}" \
      org.opencontainers.image.vendor="PistonProtection" \
      org.opencontainers.image.licenses="Apache-2.0" \
      org.opencontainers.image.source="https://github.com/pistonprotection/pistonprotection"

# Expose ports
# 8080 - HTTP API
# 9090 - gRPC API
# 9091 - Metrics
EXPOSE 8080 9090 9091

# Health check
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Run as non-root user
USER pistonprotection

# Environment defaults
ENV RUST_LOG=info \
    RUST_BACKTRACE=0

ENTRYPOINT ["/usr/local/bin/gateway"]
