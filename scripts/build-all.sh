#!/usr/bin/env bash
# =============================================================================
# PistonProtection - Build All Components
# =============================================================================
#
# Usage:
#   ./scripts/build-all.sh              # Build all components
#   ./scripts/build-all.sh --release    # Build in release mode
#   ./scripts/build-all.sh --docker     # Build Docker images
#   ./scripts/build-all.sh --help       # Show help
#
# =============================================================================

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"

# Default options
RELEASE_MODE=false
BUILD_DOCKER=false
BUILD_EBPF=true
BUILD_FRONTEND=true
BUILD_SERVICES=true
BUILD_OPERATOR=true
PARALLEL=true
VERBOSE=false

# Print colored message
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Print help
print_help() {
    cat << EOF
PistonProtection Build Script

Usage: $0 [OPTIONS]

Options:
    --release           Build in release mode (default: debug)
    --docker            Build Docker images instead of local binaries
    --no-ebpf           Skip eBPF programs build
    --no-frontend       Skip frontend build
    --no-services       Skip Rust services build
    --no-operator       Skip operator build
    --no-parallel       Disable parallel builds
    --verbose, -v       Enable verbose output
    --help, -h          Show this help message

Examples:
    $0                          # Build all in debug mode
    $0 --release                # Build all in release mode
    $0 --docker                 # Build all Docker images
    $0 --release --no-ebpf      # Build release without eBPF
EOF
}

# Parse arguments
parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            --release)
                RELEASE_MODE=true
                shift
                ;;
            --docker)
                BUILD_DOCKER=true
                shift
                ;;
            --no-ebpf)
                BUILD_EBPF=false
                shift
                ;;
            --no-frontend)
                BUILD_FRONTEND=false
                shift
                ;;
            --no-services)
                BUILD_SERVICES=false
                shift
                ;;
            --no-operator)
                BUILD_OPERATOR=false
                shift
                ;;
            --no-parallel)
                PARALLEL=false
                shift
                ;;
            --verbose|-v)
                VERBOSE=true
                shift
                ;;
            --help|-h)
                print_help
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                print_help
                exit 1
                ;;
        esac
    done
}

# Check dependencies
check_dependencies() {
    log_info "Checking dependencies..."

    local missing_deps=()

    if $BUILD_SERVICES || $BUILD_OPERATOR; then
        if ! command -v cargo &> /dev/null; then
            missing_deps+=("cargo (Rust)")
        fi
    fi

    if $BUILD_FRONTEND; then
        if ! command -v pnpm &> /dev/null; then
            if ! command -v npm &> /dev/null; then
                missing_deps+=("pnpm or npm (Node.js)")
            fi
        fi
    fi

    if $BUILD_EBPF; then
        if ! rustup show | grep -q "nightly"; then
            log_warning "Rust nightly not found, eBPF build may fail"
        fi
    fi

    if $BUILD_DOCKER; then
        if ! command -v docker &> /dev/null; then
            missing_deps+=("docker")
        fi
    fi

    if [[ ${#missing_deps[@]} -gt 0 ]]; then
        log_error "Missing dependencies:"
        for dep in "${missing_deps[@]}"; do
            echo "  - $dep"
        done
        exit 1
    fi

    log_success "All dependencies found"
}

# Build Rust services
build_services() {
    log_info "Building Rust services..."

    cd "$ROOT_DIR/services"

    local cargo_args=()
    if $RELEASE_MODE; then
        cargo_args+=("--release")
    fi
    if $VERBOSE; then
        cargo_args+=("-v")
    fi

    cargo build "${cargo_args[@]}"

    log_success "Rust services built successfully"
}

# Build Kubernetes operator
build_operator() {
    log_info "Building Kubernetes operator..."

    cd "$ROOT_DIR/operator"

    local cargo_args=()
    if $RELEASE_MODE; then
        cargo_args+=("--release")
    fi
    if $VERBOSE; then
        cargo_args+=("-v")
    fi

    cargo build "${cargo_args[@]}"

    log_success "Kubernetes operator built successfully"
}

# Build eBPF programs
build_ebpf() {
    log_info "Building eBPF programs..."

    cd "$ROOT_DIR/ebpf"

    # Check for nightly toolchain
    if ! rustup show | grep -q "nightly"; then
        log_warning "Installing Rust nightly toolchain..."
        rustup install nightly
        rustup component add rust-src --toolchain nightly
    fi

    # Install bpf-linker if not present
    if ! command -v bpf-linker &> /dev/null; then
        log_info "Installing bpf-linker..."
        cargo +nightly install bpf-linker
    fi

    local cargo_args=("--target" "bpfel-unknown-none" "-Z" "build-std=core")
    if $RELEASE_MODE; then
        cargo_args+=("--release")
    fi

    cargo +nightly build "${cargo_args[@]}"

    log_success "eBPF programs built successfully"
}

# Build frontend
build_frontend() {
    log_info "Building frontend..."

    cd "$ROOT_DIR/frontend"

    # Use pnpm if available, otherwise npm
    local pkg_mgr="pnpm"
    if ! command -v pnpm &> /dev/null; then
        pkg_mgr="npm"
        log_warning "pnpm not found, using npm"
    fi

    # Install dependencies
    log_info "Installing frontend dependencies..."
    $pkg_mgr install

    # Build
    log_info "Building frontend application..."
    $pkg_mgr run build

    log_success "Frontend built successfully"
}

# Build Docker images
build_docker_images() {
    log_info "Building Docker images..."

    cd "$ROOT_DIR"

    local components=("gateway" "worker" "config-mgr" "metrics" "auth" "operator" "frontend")
    local build_args=()

    if $VERBOSE; then
        build_args+=("--progress=plain")
    fi

    # Get version info
    local version="${VERSION:-0.0.0-dev}"
    local git_commit
    git_commit=$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")

    for component in "${components[@]}"; do
        log_info "Building Docker image: pistonprotection/$component"

        docker build \
            -f "docker/$component/Dockerfile" \
            -t "pistonprotection/$component:latest" \
            -t "pistonprotection/$component:$version" \
            --build-arg VERSION="$version" \
            --build-arg GIT_COMMIT="$git_commit" \
            "${build_args[@]}" \
            .

        log_success "Built pistonprotection/$component:$version"
    done

    log_success "All Docker images built successfully"
}

# Main build function
main() {
    parse_args "$@"

    log_info "Starting PistonProtection build..."
    log_info "Release mode: $RELEASE_MODE"
    log_info "Docker build: $BUILD_DOCKER"

    check_dependencies

    if $BUILD_DOCKER; then
        build_docker_images
    else
        # Build components
        local pids=()

        if $BUILD_SERVICES; then
            if $PARALLEL; then
                build_services &
                pids+=($!)
            else
                build_services
            fi
        fi

        if $BUILD_OPERATOR; then
            if $PARALLEL; then
                build_operator &
                pids+=($!)
            else
                build_operator
            fi
        fi

        if $BUILD_EBPF; then
            if $PARALLEL; then
                build_ebpf &
                pids+=($!)
            else
                build_ebpf
            fi
        fi

        if $BUILD_FRONTEND; then
            if $PARALLEL; then
                build_frontend &
                pids+=($!)
            else
                build_frontend
            fi
        fi

        # Wait for parallel builds
        if $PARALLEL && [[ ${#pids[@]} -gt 0 ]]; then
            log_info "Waiting for parallel builds to complete..."
            local failed=false
            for pid in "${pids[@]}"; do
                if ! wait "$pid"; then
                    failed=true
                fi
            done
            if $failed; then
                log_error "One or more builds failed"
                exit 1
            fi
        fi
    fi

    log_success "Build completed successfully!"
}

main "$@"
