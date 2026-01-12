# =============================================================================
# PistonProtection Makefile
# =============================================================================
#
# Usage:
#   make help          Show this help message
#   make build         Build all components
#   make test          Run all tests
#   make dev           Start local development environment
#
# =============================================================================

.PHONY: all help build build-release build-docker build-ebpf build-frontend \
        test test-unit test-integration test-coverage lint format \
        dev dev-up dev-down dev-logs dev-status dev-clean \
        docker-build docker-push docker-clean \
        helm-lint helm-package helm-install \
        clean clean-all release version

# Configuration
SHELL := /bin/bash
.DEFAULT_GOAL := help

# Version info
VERSION ?= $(shell git describe --tags --always --dirty 2>/dev/null || echo "0.0.0-dev")
GIT_COMMIT ?= $(shell git rev-parse --short HEAD 2>/dev/null || echo "unknown")
BUILD_TIME ?= $(shell date -u +"%Y-%m-%dT%H:%M:%SZ")

# Docker settings
DOCKER_REGISTRY ?= ghcr.io
DOCKER_REPO ?= pistonprotection/pistonprotection
DOCKER_TAG ?= $(VERSION)

# Rust settings
CARGO := cargo
CARGO_FLAGS ?=

# Colors
BLUE := \033[0;34m
GREEN := \033[0;32m
YELLOW := \033[1;33m
RED := \033[0;31m
NC := \033[0m

# =============================================================================
# Help
# =============================================================================

help: ## Show this help message
	@echo ""
	@echo "$(BLUE)PistonProtection$(NC) - Enterprise DDoS Protection Platform"
	@echo ""
	@echo "$(YELLOW)Usage:$(NC)"
	@echo "  make $(GREEN)<target>$(NC)"
	@echo ""
	@echo "$(YELLOW)Build Targets:$(NC)"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; /^[a-zA-Z_-]+:.*?## / {printf "  $(GREEN)%-20s$(NC) %s\n", $$1, $$2}'
	@echo ""
	@echo "$(YELLOW)Version:$(NC) $(VERSION)"
	@echo ""

# =============================================================================
# Build Targets
# =============================================================================

build: ## Build all components (debug mode)
	@echo "$(BLUE)[BUILD]$(NC) Building all components..."
	@./scripts/build-all.sh

build-release: ## Build all components (release mode)
	@echo "$(BLUE)[BUILD]$(NC) Building all components in release mode..."
	@./scripts/build-all.sh --release

build-services: ## Build Rust services
	@echo "$(BLUE)[BUILD]$(NC) Building Rust services..."
	@cd services && $(CARGO) build $(CARGO_FLAGS)

build-services-release: ## Build Rust services (release)
	@echo "$(BLUE)[BUILD]$(NC) Building Rust services (release)..."
	@cd services && $(CARGO) build --release $(CARGO_FLAGS)

build-operator: ## Build Kubernetes operator
	@echo "$(BLUE)[BUILD]$(NC) Building Kubernetes operator..."
	@cd operator && $(CARGO) build $(CARGO_FLAGS)

build-operator-release: ## Build Kubernetes operator (release)
	@echo "$(BLUE)[BUILD]$(NC) Building Kubernetes operator (release)..."
	@cd operator && $(CARGO) build --release $(CARGO_FLAGS)

build-ebpf: ## Build eBPF programs
	@echo "$(BLUE)[BUILD]$(NC) Building eBPF programs..."
	@./scripts/build-ebpf.sh

build-ebpf-release: ## Build eBPF programs (release)
	@echo "$(BLUE)[BUILD]$(NC) Building eBPF programs (release)..."
	@./scripts/build-ebpf.sh --release

build-frontend: ## Build frontend
	@echo "$(BLUE)[BUILD]$(NC) Building frontend..."
	@cd frontend && pnpm install && pnpm build

# =============================================================================
# Test Targets
# =============================================================================

test: ## Run all tests
	@echo "$(BLUE)[TEST]$(NC) Running all tests..."
	@./scripts/test-all.sh

test-unit: ## Run unit tests only
	@echo "$(BLUE)[TEST]$(NC) Running unit tests..."
	@./scripts/test-all.sh --unit

test-integration: ## Run integration tests only
	@echo "$(BLUE)[TEST]$(NC) Running integration tests..."
	@./scripts/test-all.sh --integration

test-coverage: ## Run tests with coverage report
	@echo "$(BLUE)[TEST]$(NC) Running tests with coverage..."
	@./scripts/test-all.sh --coverage

test-services: ## Run Rust service tests
	@echo "$(BLUE)[TEST]$(NC) Running service tests..."
	@cd services && $(CARGO) test $(CARGO_FLAGS)

test-operator: ## Run operator tests
	@echo "$(BLUE)[TEST]$(NC) Running operator tests..."
	@cd operator && $(CARGO) test $(CARGO_FLAGS)

test-frontend: ## Run frontend tests
	@echo "$(BLUE)[TEST]$(NC) Running frontend tests..."
	@cd frontend && pnpm test || true

# =============================================================================
# Lint & Format
# =============================================================================

lint: ## Run all linters
	@echo "$(BLUE)[LINT]$(NC) Running linters..."
	@$(MAKE) lint-rust
	@$(MAKE) lint-frontend
	@$(MAKE) lint-helm

lint-rust: ## Lint Rust code
	@echo "$(BLUE)[LINT]$(NC) Linting Rust code..."
	@cd services && $(CARGO) clippy --all-targets --all-features -- -D warnings
	@cd operator && $(CARGO) clippy --all-targets -- -D warnings

lint-frontend: ## Lint frontend code
	@echo "$(BLUE)[LINT]$(NC) Linting frontend..."
	@cd frontend && pnpm lint

lint-helm: ## Lint Helm chart
	@echo "$(BLUE)[LINT]$(NC) Linting Helm chart..."
	@helm lint charts/pistonprotection

format: ## Format all code
	@echo "$(BLUE)[FORMAT]$(NC) Formatting code..."
	@cd services && $(CARGO) fmt
	@cd operator && $(CARGO) fmt
	@cd frontend && pnpm exec prettier --write "src/**/*.{ts,tsx,js,jsx,css,json}" || true

format-check: ## Check code formatting
	@echo "$(BLUE)[FORMAT]$(NC) Checking code formatting..."
	@cd services && $(CARGO) fmt --check
	@cd operator && $(CARGO) fmt --check

# =============================================================================
# Development Environment
# =============================================================================

dev: dev-up ## Alias for dev-up

dev-up: ## Start local development environment
	@echo "$(BLUE)[DEV]$(NC) Starting development environment..."
	@./scripts/local-dev.sh up

dev-up-build: ## Start with rebuild
	@echo "$(BLUE)[DEV]$(NC) Starting with rebuild..."
	@./scripts/local-dev.sh up --build

dev-down: ## Stop development environment
	@echo "$(BLUE)[DEV]$(NC) Stopping development environment..."
	@./scripts/local-dev.sh down

dev-restart: ## Restart development environment
	@echo "$(BLUE)[DEV]$(NC) Restarting development environment..."
	@./scripts/local-dev.sh restart

dev-logs: ## View development logs
	@./scripts/local-dev.sh logs -f

dev-status: ## Show development status
	@./scripts/local-dev.sh status

dev-clean: ## Clean development environment
	@echo "$(BLUE)[DEV]$(NC) Cleaning development environment..."
	@./scripts/local-dev.sh clean

dev-shell-%: ## Open shell in container (e.g., make dev-shell-postgres)
	@./scripts/local-dev.sh shell $*

# =============================================================================
# Docker Targets
# =============================================================================

docker-build: ## Build all Docker images
	@echo "$(BLUE)[DOCKER]$(NC) Building Docker images..."
	@./scripts/build-all.sh --docker

docker-build-%: ## Build specific Docker image (e.g., make docker-build-gateway)
	@echo "$(BLUE)[DOCKER]$(NC) Building $* image..."
	@docker build -f docker/$*/Dockerfile \
		-t $(DOCKER_REPO)/$*:$(DOCKER_TAG) \
		-t $(DOCKER_REPO)/$*:latest \
		--build-arg VERSION=$(VERSION) \
		--build-arg GIT_COMMIT=$(GIT_COMMIT) \
		.

docker-push: ## Push all Docker images
	@echo "$(BLUE)[DOCKER]$(NC) Pushing Docker images..."
	@for component in gateway worker config-mgr metrics auth operator frontend; do \
		docker push $(DOCKER_REGISTRY)/$(DOCKER_REPO)/$$component:$(DOCKER_TAG); \
		docker push $(DOCKER_REGISTRY)/$(DOCKER_REPO)/$$component:latest; \
	done

docker-push-%: ## Push specific Docker image
	@echo "$(BLUE)[DOCKER]$(NC) Pushing $* image..."
	@docker push $(DOCKER_REGISTRY)/$(DOCKER_REPO)/$*:$(DOCKER_TAG)
	@docker push $(DOCKER_REGISTRY)/$(DOCKER_REPO)/$*:latest

docker-clean: ## Remove Docker images
	@echo "$(BLUE)[DOCKER]$(NC) Cleaning Docker images..."
	@docker images $(DOCKER_REPO)/* -q | xargs -r docker rmi -f

# =============================================================================
# Helm Targets
# =============================================================================

helm-lint: ## Lint Helm chart
	@echo "$(BLUE)[HELM]$(NC) Linting chart..."
	@helm lint charts/pistonprotection

helm-template: ## Render Helm templates
	@echo "$(BLUE)[HELM]$(NC) Rendering templates..."
	@helm template test charts/pistonprotection

helm-package: ## Package Helm chart
	@echo "$(BLUE)[HELM]$(NC) Packaging chart..."
	@helm package charts/pistonprotection --destination .

helm-deps: ## Update Helm dependencies
	@echo "$(BLUE)[HELM]$(NC) Updating dependencies..."
	@helm dependency update charts/pistonprotection

helm-install: ## Install Helm chart (dry-run)
	@echo "$(BLUE)[HELM]$(NC) Installing chart (dry-run)..."
	@helm install pistonprotection charts/pistonprotection --dry-run --debug

helm-upgrade: ## Upgrade Helm release
	@echo "$(BLUE)[HELM]$(NC) Upgrading release..."
	@helm upgrade pistonprotection charts/pistonprotection --install

# =============================================================================
# Clean Targets
# =============================================================================

clean: ## Clean build artifacts
	@echo "$(BLUE)[CLEAN]$(NC) Cleaning build artifacts..."
	@cd services && $(CARGO) clean
	@cd operator && $(CARGO) clean
	@cd ebpf && $(CARGO) clean || true
	@rm -rf frontend/.output frontend/node_modules
	@rm -f *.tgz coverage-*.lcov

clean-all: clean docker-clean ## Clean everything including Docker

# =============================================================================
# Release Targets
# =============================================================================

release: ## Create a new release
	@echo "$(BLUE)[RELEASE]$(NC) Creating release $(VERSION)..."
	@$(MAKE) lint
	@$(MAKE) test
	@$(MAKE) build-release
	@$(MAKE) docker-build
	@echo "$(GREEN)[SUCCESS]$(NC) Release $(VERSION) ready!"

version: ## Show version info
	@echo "Version:    $(VERSION)"
	@echo "Git Commit: $(GIT_COMMIT)"
	@echo "Build Time: $(BUILD_TIME)"

# =============================================================================
# CI/CD Helpers
# =============================================================================

ci-setup: ## Setup CI environment
	@echo "$(BLUE)[CI]$(NC) Setting up CI environment..."
	@rustup component add rustfmt clippy llvm-tools-preview
	@cargo install cargo-llvm-cov cargo-audit cargo-deny || true

ci-lint: format-check lint ## Run CI lint checks

ci-test: test-coverage ## Run CI tests with coverage

ci-security: ## Run security scans
	@echo "$(BLUE)[CI]$(NC) Running security scans..."
	@cd services && cargo audit || true
	@cd frontend && pnpm audit || true

# =============================================================================
# Utility Targets
# =============================================================================

deps: ## Install development dependencies
	@echo "$(BLUE)[DEPS]$(NC) Installing dependencies..."
	@rustup update
	@rustup component add rustfmt clippy rust-src
	@cd frontend && pnpm install

update-deps: ## Update all dependencies
	@echo "$(BLUE)[DEPS]$(NC) Updating dependencies..."
	@cd services && cargo update
	@cd operator && cargo update
	@cd frontend && pnpm update

.PHONY: docs
docs: ## Generate documentation
	@echo "$(BLUE)[DOCS]$(NC) Generating documentation..."
	@cd services && cargo doc --no-deps --open

watch: ## Watch for changes and rebuild
	@echo "$(BLUE)[WATCH]$(NC) Watching for changes..."
	@cd services && cargo watch -x build

watch-test: ## Watch for changes and run tests
	@echo "$(BLUE)[WATCH]$(NC) Watching for changes (tests)..."
	@cd services && cargo watch -x test
