# syntax=docker/dockerfile:1.7

# =============================================================================
# PistonProtection Kubernetes Operator
# Multi-stage build for optimal image size and security
# =============================================================================

ARG DEBIAN_VERSION=bookworm

# -----------------------------------------------------------------------------
# Stage 1: Chef - Dependency caching with nightly Rust
# -----------------------------------------------------------------------------
FROM debian:${DEBIAN_VERSION} AS chef

# Install rustup and nightly toolchain
RUN apt-get update && apt-get install -y --no-install-recommends \
    curl \
    ca-certificates \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain nightly-2026-01-10 \
    && rustup component add rustfmt clippy

RUN cargo install cargo-chef --locked
WORKDIR /app

# -----------------------------------------------------------------------------
# Stage 2: Planner - Create recipe for dependencies
# -----------------------------------------------------------------------------
FROM chef AS planner

COPY operator/Cargo.toml operator/Cargo.lock ./
COPY operator/benches ./benches

# Create dummy source for dependency resolution
RUN mkdir src && \
    echo 'fn main() {}' > src/main.rs

RUN cargo chef prepare --recipe-path recipe.json

# -----------------------------------------------------------------------------
# Stage 3: Builder - Build dependencies and application
# -----------------------------------------------------------------------------
FROM chef AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Build dependencies (cached)
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Copy actual source and build
COPY operator/Cargo.toml operator/Cargo.lock ./
COPY operator/src ./src
COPY operator/benches ./benches

# Build arguments for versioning
ARG VERSION=0.0.0
ARG GIT_COMMIT=unknown

RUN cargo build --release && \
    strip /app/target/release/pistonprotection-operator

# -----------------------------------------------------------------------------
# Stage 4: Runtime - Minimal distroless-like image
# -----------------------------------------------------------------------------
FROM gcr.io/distroless/cc-debian12:nonroot AS runtime

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/pistonprotection-operator /usr/local/bin/

# Metadata
ARG VERSION=0.0.0
ARG GIT_COMMIT=unknown

LABEL org.opencontainers.image.title="PistonProtection Operator" \
      org.opencontainers.image.description="Kubernetes operator for PistonProtection DDoS protection platform" \
      org.opencontainers.image.version="${VERSION}" \
      org.opencontainers.image.revision="${GIT_COMMIT}" \
      org.opencontainers.image.vendor="PistonProtection" \
      org.opencontainers.image.licenses="Apache-2.0" \
      org.opencontainers.image.source="https://github.com/pistonprotection/app"

# Expose ports
# 8080 - HTTP webhook server
# 9091 - Metrics
EXPOSE 8080 9091

# Run as non-root (distroless nonroot user is 65532)
USER 65532:65532

# Environment defaults
ENV RUST_LOG=info \
    RUST_BACKTRACE=0

ENTRYPOINT ["/usr/local/bin/pistonprotection-operator"]
