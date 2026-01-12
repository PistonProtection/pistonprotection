#!/usr/bin/env bash
# =============================================================================
# PistonProtection - Local Development Environment
# =============================================================================
#
# This script manages the local development environment using Docker Compose.
#
# Usage:
#   ./scripts/local-dev.sh up           # Start all services
#   ./scripts/local-dev.sh down         # Stop all services
#   ./scripts/local-dev.sh restart      # Restart all services
#   ./scripts/local-dev.sh logs         # View logs
#   ./scripts/local-dev.sh status       # Show service status
#   ./scripts/local-dev.sh clean        # Remove all data
#
# =============================================================================

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"

# Docker Compose options
COMPOSE_FILE="$ROOT_DIR/docker-compose.yml"
COMPOSE_PROJECT="pistonprotection"

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
PistonProtection Local Development Script

Usage: $0 COMMAND [OPTIONS]

Commands:
    up              Start all services
    down            Stop all services
    restart         Restart all services
    logs            View service logs
    status          Show service status
    build           Build Docker images
    rebuild         Rebuild and restart services
    clean           Stop services and remove all data
    shell SERVICE   Open shell in service container
    ps              Show running containers

Options:
    --build         Rebuild images when starting
    --tools         Include development tools (pgadmin, redis-commander)
    --no-cache      Build without cache
    -f, --follow    Follow log output
    -s, --service   Specify service(s)

Examples:
    $0 up                       # Start all services
    $0 up --build               # Start with rebuild
    $0 up --tools               # Start with dev tools
    $0 logs -f gateway          # Follow gateway logs
    $0 shell postgres           # Shell into postgres container
    $0 restart gateway worker   # Restart specific services

Access URLs:
    Frontend:    http://localhost:3000
    Gateway API: http://localhost:8080
    Grafana:     http://localhost:3001 (admin/admin)
    Prometheus:  http://localhost:9099
    pgAdmin:     http://localhost:5050 (when using --tools)
EOF
}

# Check Docker is available
check_docker() {
    if ! command -v docker &> /dev/null; then
        log_error "Docker not found. Please install Docker."
        exit 1
    fi

    if ! docker info &> /dev/null; then
        log_error "Docker daemon not running. Please start Docker."
        exit 1
    fi

    # Check for docker compose (v2) or docker-compose (v1)
    if docker compose version &> /dev/null; then
        DOCKER_COMPOSE="docker compose"
    elif command -v docker-compose &> /dev/null; then
        DOCKER_COMPOSE="docker-compose"
    else
        log_error "Docker Compose not found."
        exit 1
    fi
}

# Run docker compose command
dc() {
    $DOCKER_COMPOSE -f "$COMPOSE_FILE" -p "$COMPOSE_PROJECT" "$@"
}

# Start services
cmd_up() {
    local build_flag=""
    local profiles=""

    while [[ $# -gt 0 ]]; do
        case $1 in
            --build)
                build_flag="--build"
                shift
                ;;
            --tools)
                profiles="--profile tools"
                shift
                ;;
            *)
                break
                ;;
        esac
    done

    log_info "Starting PistonProtection development environment..."

    # Create init-db.sql if it doesn't exist
    if [[ ! -f "$ROOT_DIR/scripts/init-db.sql" ]]; then
        log_info "Creating database initialization script..."
        cat > "$ROOT_DIR/scripts/init-db.sql" << 'EOF'
-- PistonProtection Database Initialization

-- Create extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Create application user with limited privileges (optional)
-- CREATE USER pistonprotection_app WITH PASSWORD 'app_password';
-- GRANT CONNECT ON DATABASE pistonprotection TO pistonprotection_app;

-- Create schemas
CREATE SCHEMA IF NOT EXISTS auth;
CREATE SCHEMA IF NOT EXISTS config;
CREATE SCHEMA IF NOT EXISTS metrics;

-- Log initialization
DO $$
BEGIN
    RAISE NOTICE 'PistonProtection database initialized successfully';
END $$;
EOF
    fi

    dc up -d $build_flag $profiles "$@"

    log_info "Waiting for services to be healthy..."
    sleep 5

    # Check service health
    local services=("postgres" "redis" "gateway")
    for svc in "${services[@]}"; do
        local health
        health=$(docker inspect --format='{{.State.Health.Status}}' "pp-$svc" 2>/dev/null || echo "unknown")
        if [[ "$health" == "healthy" ]]; then
            echo -e "  ${GREEN}[ok]${NC} $svc"
        elif [[ "$health" == "starting" ]]; then
            echo -e "  ${YELLOW}[starting]${NC} $svc"
        else
            echo -e "  ${RED}[unhealthy]${NC} $svc"
        fi
    done

    echo ""
    log_success "Development environment started!"
    echo ""
    echo -e "${CYAN}Access URLs:${NC}"
    echo "  Frontend:    http://localhost:3000"
    echo "  Gateway API: http://localhost:8080"
    echo "  Grafana:     http://localhost:3001"
    echo "  Prometheus:  http://localhost:9099"
    echo ""
    echo "Run '$0 logs -f' to view logs"
}

# Stop services
cmd_down() {
    log_info "Stopping PistonProtection development environment..."
    dc down "$@"
    log_success "Environment stopped"
}

# Restart services
cmd_restart() {
    if [[ $# -eq 0 ]]; then
        log_info "Restarting all services..."
        dc restart
    else
        log_info "Restarting: $*"
        dc restart "$@"
    fi
    log_success "Restart complete"
}

# View logs
cmd_logs() {
    local follow=""
    local services=()

    while [[ $# -gt 0 ]]; do
        case $1 in
            -f|--follow)
                follow="-f"
                shift
                ;;
            *)
                services+=("$1")
                shift
                ;;
        esac
    done

    if [[ ${#services[@]} -eq 0 ]]; then
        dc logs $follow
    else
        dc logs $follow "${services[@]}"
    fi
}

# Show status
cmd_status() {
    echo -e "${CYAN}PistonProtection Service Status${NC}"
    echo ""
    dc ps --format "table {{.Name}}\t{{.Status}}\t{{.Ports}}"
}

# Build images
cmd_build() {
    local no_cache=""

    while [[ $# -gt 0 ]]; do
        case $1 in
            --no-cache)
                no_cache="--no-cache"
                shift
                ;;
            *)
                break
                ;;
        esac
    done

    log_info "Building Docker images..."
    dc build $no_cache "$@"
    log_success "Build complete"
}

# Rebuild and restart
cmd_rebuild() {
    log_info "Rebuilding and restarting services..."
    dc up -d --build "$@"
    log_success "Rebuild complete"
}

# Clean everything
cmd_clean() {
    log_warning "This will remove all containers, volumes, and data!"
    read -p "Are you sure? (y/N) " -n 1 -r
    echo

    if [[ $REPLY =~ ^[Yy]$ ]]; then
        log_info "Cleaning up..."
        dc down -v --remove-orphans

        # Remove any dangling images
        docker image prune -f --filter "label=com.docker.compose.project=$COMPOSE_PROJECT"

        log_success "Cleanup complete"
    else
        log_info "Cleanup cancelled"
    fi
}

# Open shell in container
cmd_shell() {
    if [[ $# -eq 0 ]]; then
        log_error "Please specify a service name"
        exit 1
    fi

    local service="$1"
    local container="pp-$service"

    # Check if container exists
    if ! docker ps -a --format '{{.Names}}' | grep -q "^$container$"; then
        log_error "Container $container not found"
        exit 1
    fi

    log_info "Opening shell in $container..."
    docker exec -it "$container" /bin/sh || docker exec -it "$container" /bin/bash
}

# Show running containers
cmd_ps() {
    dc ps "$@"
}

# Wait for service to be healthy
wait_healthy() {
    local service="$1"
    local timeout="${2:-60}"
    local container="pp-$service"

    log_info "Waiting for $service to be healthy..."

    local count=0
    while [[ $count -lt $timeout ]]; do
        local health
        health=$(docker inspect --format='{{.State.Health.Status}}' "$container" 2>/dev/null || echo "unknown")

        if [[ "$health" == "healthy" ]]; then
            log_success "$service is healthy"
            return 0
        fi

        sleep 1
        ((count++))
    done

    log_error "$service did not become healthy within ${timeout}s"
    return 1
}

# Main function
main() {
    if [[ $# -eq 0 ]]; then
        print_help
        exit 0
    fi

    local command="$1"
    shift

    check_docker

    case "$command" in
        up)
            cmd_up "$@"
            ;;
        down)
            cmd_down "$@"
            ;;
        restart)
            cmd_restart "$@"
            ;;
        logs)
            cmd_logs "$@"
            ;;
        status)
            cmd_status "$@"
            ;;
        build)
            cmd_build "$@"
            ;;
        rebuild)
            cmd_rebuild "$@"
            ;;
        clean)
            cmd_clean "$@"
            ;;
        shell)
            cmd_shell "$@"
            ;;
        ps)
            cmd_ps "$@"
            ;;
        help|--help|-h)
            print_help
            ;;
        *)
            log_error "Unknown command: $command"
            print_help
            exit 1
            ;;
    esac
}

main "$@"
