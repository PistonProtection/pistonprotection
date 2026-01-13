#!/usr/bin/env bash
# =============================================================================
# PistonProtection - Run All Tests
# =============================================================================
#
# Usage:
#   ./scripts/test-all.sh               # Run all tests
#   ./scripts/test-all.sh --unit        # Run only unit tests
#   ./scripts/test-all.sh --integration # Run only integration tests
#   ./scripts/test-all.sh --coverage    # Run with coverage
#   ./scripts/test-all.sh --help        # Show help
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
RUN_UNIT=true
RUN_INTEGRATION=true
RUN_E2E=false
RUN_COVERAGE=false
RUN_LINT=false
VERBOSE=false
FAIL_FAST=false

# Test results
TESTS_PASSED=0
TESTS_FAILED=0

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
PistonProtection Test Script

Usage: $0 [OPTIONS]

Options:
    --unit              Run only unit tests
    --integration       Run only integration tests
    --e2e               Run end-to-end tests (requires running services)
    --coverage          Generate code coverage report
    --lint              Run linters before tests
    --fail-fast         Stop on first failure
    --verbose, -v       Enable verbose output
    --help, -h          Show this help message

Components:
    --services          Test Rust services only
    --operator          Test operator only
    --frontend          Test frontend only

Examples:
    $0                          # Run all tests
    $0 --unit                   # Run unit tests only
    $0 --coverage --services    # Run service tests with coverage
EOF
}

# Parse arguments
parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            --unit)
                RUN_UNIT=true
                RUN_INTEGRATION=false
                shift
                ;;
            --integration)
                RUN_UNIT=false
                RUN_INTEGRATION=true
                shift
                ;;
            --e2e)
                RUN_E2E=true
                shift
                ;;
            --coverage)
                RUN_COVERAGE=true
                shift
                ;;
            --lint)
                RUN_LINT=true
                shift
                ;;
            --fail-fast)
                FAIL_FAST=true
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

# Check test environment
check_environment() {
    log_info "Checking test environment..."

    if ! command -v cargo &> /dev/null; then
        log_error "cargo not found. Please install Rust."
        exit 1
    fi

    if ! command -v pnpm &> /dev/null && ! command -v npm &> /dev/null; then
        log_warning "pnpm/npm not found. Frontend tests will be skipped."
    fi

    if $RUN_COVERAGE; then
        if ! cargo install --list | grep -q "cargo-llvm-cov"; then
            log_info "Installing cargo-llvm-cov..."
            cargo install cargo-llvm-cov
        fi
    fi

    log_success "Environment check passed"
}

# Run linters
run_linters() {
    log_info "Running linters..."

    local lint_failed=false

    # Rust formatting
    log_info "Checking Rust formatting..."
    if ! (cd "$ROOT_DIR/services" && cargo fmt --all -- --check); then
        log_error "Rust formatting check failed"
        lint_failed=true
    fi

    # Rust clippy
    log_info "Running clippy..."
    if ! (cd "$ROOT_DIR/services" && cargo clippy --all-targets --all-features -- -D warnings); then
        log_error "Clippy check failed"
        lint_failed=true
    fi

    # Frontend linting
    if command -v pnpm &> /dev/null || command -v npm &> /dev/null; then
        log_info "Running frontend linter..."
        local pkg_mgr="pnpm"
        [[ ! -x "$(command -v pnpm)" ]] && pkg_mgr="npm"

        if ! (cd "$ROOT_DIR/frontend" && $pkg_mgr run lint); then
            log_error "Frontend lint check failed"
            lint_failed=true
        fi
    fi

    if $lint_failed; then
        log_error "Linting failed"
        if $FAIL_FAST; then
            exit 1
        fi
        return 1
    fi

    log_success "All linters passed"
}

# Run Rust service tests
run_service_tests() {
    log_info "Running Rust service tests..."

    cd "$ROOT_DIR/services"

    local cargo_args=("--all-features")

    if $VERBOSE; then
        cargo_args+=("--verbose")
    fi

    if $FAIL_FAST; then
        cargo_args+=("--no-fail-fast")
    fi

    if $RUN_COVERAGE; then
        log_info "Running with coverage..."

        if $RUN_UNIT && ! $RUN_INTEGRATION; then
            cargo llvm-cov --lib "${cargo_args[@]}" --lcov --output-path "$ROOT_DIR/coverage-services.lcov"
        elif $RUN_INTEGRATION && ! $RUN_UNIT; then
            cargo llvm-cov --test '*' "${cargo_args[@]}" --lcov --output-path "$ROOT_DIR/coverage-services.lcov"
        else
            cargo llvm-cov "${cargo_args[@]}" --lcov --output-path "$ROOT_DIR/coverage-services.lcov"
        fi

        log_info "Coverage report: $ROOT_DIR/coverage-services.lcov"
    else
        if $RUN_UNIT; then
            log_info "Running unit tests..."
            if cargo test --lib "${cargo_args[@]}"; then
                ((TESTS_PASSED++))
            else
                ((TESTS_FAILED++))
                if $FAIL_FAST; then
                    return 1
                fi
            fi
        fi

        if $RUN_INTEGRATION; then
            log_info "Running integration tests..."
            # Integration test targets may not exist if disabled
            local integration_output
            if integration_output=$(cargo test --test '*' "${cargo_args[@]}" 2>&1); then
                ((TESTS_PASSED++))
            elif echo "$integration_output" | grep -q "no test target matches pattern"; then
                log_info "No integration test targets found (may be disabled)"
                ((TESTS_PASSED++))
            else
                echo "$integration_output"
                ((TESTS_FAILED++))
                if $FAIL_FAST; then
                    return 1
                fi
            fi
        fi

        # Run doc tests
        log_info "Running doc tests..."
        if cargo test --doc "${cargo_args[@]}"; then
            ((TESTS_PASSED++))
        else
            ((TESTS_FAILED++))
        fi
    fi
}

# Run operator tests
run_operator_tests() {
    log_info "Running operator tests..."

    cd "$ROOT_DIR/operator"

    local cargo_args=()

    if $VERBOSE; then
        cargo_args+=("--verbose")
    fi

    if $RUN_COVERAGE; then
        cargo llvm-cov "${cargo_args[@]}" --lcov --output-path "$ROOT_DIR/coverage-operator.lcov"
    else
        if cargo test "${cargo_args[@]}"; then
            ((TESTS_PASSED++))
        else
            ((TESTS_FAILED++))
        fi
    fi
}

# Run frontend tests
run_frontend_tests() {
    log_info "Running frontend tests..."

    cd "$ROOT_DIR/frontend"

    local pkg_mgr="pnpm"
    if ! command -v pnpm &> /dev/null; then
        if command -v npm &> /dev/null; then
            pkg_mgr="npm"
        else
            log_warning "No package manager found, skipping frontend tests"
            return 0
        fi
    fi

    # Install dependencies if needed
    if [[ ! -d "node_modules" ]]; then
        log_info "Installing frontend dependencies..."
        $pkg_mgr install
    fi

    # Type check (using biome check which includes type checking)
    log_info "Running code quality check..."
    if $pkg_mgr run check; then
        ((TESTS_PASSED++))
    else
        ((TESTS_FAILED++))
        if $FAIL_FAST; then
            return 1
        fi
    fi

    # Run tests if they exist
    if grep -q '"test"' package.json; then
        log_info "Running frontend unit tests..."
        if $pkg_mgr test -- --passWithNoTests; then
            ((TESTS_PASSED++))
        else
            ((TESTS_FAILED++))
        fi
    fi
}

# Run E2E tests
run_e2e_tests() {
    log_info "Running end-to-end tests..."

    # Check if services are running
    if ! curl -sf http://localhost:8080/health > /dev/null 2>&1; then
        log_error "Gateway service not running. Start services first with: docker compose up -d"
        return 1
    fi

    cd "$ROOT_DIR/tests"

    if [[ -f "package.json" ]]; then
        local pkg_mgr="pnpm"
        [[ ! -x "$(command -v pnpm)" ]] && pkg_mgr="npm"

        $pkg_mgr install
        $pkg_mgr run e2e
    else
        log_warning "No E2E test configuration found"
    fi
}

# Print test summary
print_summary() {
    echo ""
    echo "=============================================="
    echo "                TEST SUMMARY                  "
    echo "=============================================="
    echo -e "  Passed: ${GREEN}$TESTS_PASSED${NC}"
    echo -e "  Failed: ${RED}$TESTS_FAILED${NC}"
    echo "=============================================="

    if [[ $TESTS_FAILED -gt 0 ]]; then
        log_error "Some tests failed"
        return 1
    else
        log_success "All tests passed!"
        return 0
    fi
}

# Main function
main() {
    parse_args "$@"

    log_info "Starting PistonProtection test suite..."

    check_environment

    if $RUN_LINT; then
        run_linters || true
    fi

    # Run tests
    run_service_tests || true
    run_operator_tests || true
    run_frontend_tests || true

    if $RUN_E2E; then
        run_e2e_tests || true
    fi

    print_summary
}

main "$@"
