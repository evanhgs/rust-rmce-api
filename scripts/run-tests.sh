#!/bin/bash

# RMCE API - Test Runner Script
# Usage: ./run-tests.sh [test-type]

set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Print banner
print_banner() {
    echo -e "${BLUE}"
    echo "╔════════════════════════════════════════╗"
    echo "║  RMCE API - Test Runner                ║"
    echo "║  Running Chronometer API Tests         ║"
    echo "╚════════════════════════════════════════╝"
    echo -e "${NC}"
}

# Print usage
print_usage() {
    echo -e "${YELLOW}Usage:${NC}"
    echo "  ./run-tests.sh all          # Run all tests"
    echo "  ./run-tests.sh integration  # Run integration tests only"
    echo "  ./run-tests.sh routes       # Run route tests only"
    echo "  ./run-tests.sh us1          # Run specific user story (US1-US6)"
    echo "  ./run-tests.sh security     # Run security tests only"
    echo "  ./run-tests.sh debug        # Run with debug logging"
    echo ""
}

# Run all tests
run_all_tests() {
    echo -e "${GREEN}Running all tests...${NC}"
    cargo test -- --test-threads=1
    echo -e "${GREEN}✅ All tests completed!${NC}"
}

# Run integration tests
run_integration_tests() {
    echo -e "${GREEN}Running integration tests (9 user stories + security tests)...${NC}"
    cargo test --test integration_tests -- --test-threads=1 --nocapture
    echo -e "${GREEN}✅ Integration tests completed!${NC}"
}

# Run route tests
run_route_tests() {
    echo -e "${GREEN}Running route tests...${NC}"
    cargo test --test routes -- --test-threads=1 --nocapture
    echo -e "${GREEN}✅ Route tests completed!${NC}"
}

# Run specific user story
run_user_story() {
    local story=$1
    echo -e "${GREEN}Running user story: $story...${NC}"
    cargo test $story -- --nocapture --test-threads=1
    echo -e "${GREEN}✅ $story test completed!${NC}"
}

# Run security tests
run_security_tests() {
    echo -e "${GREEN}Running security tests...${NC}"
    cargo test security_test -- --nocapture --test-threads=1
    echo -e "${GREEN}✅ Security tests completed!${NC}"
}

# Run with debug logging
run_debug() {
    echo -e "${GREEN}Running tests with debug logging...${NC}"
    RUST_LOG=debug cargo test --test integration_tests -- --nocapture --test-threads=1
    echo -e "${GREEN}✅ Debug tests completed!${NC}"
}

# Check if database URL is set
check_database() {
    if [ -z "$DATABASE_URL" ]; then
        echo -e "${YELLOW}⚠️  Warning: DATABASE_URL not set${NC}"
        echo "    Either set DATABASE_URL or tests will be skipped"
        echo ""
    fi
}

# Main
print_banner
check_database

if [ $# -eq 0 ]; then
    print_usage
    exit 0
fi

case "$1" in
    all)
        run_all_tests
        ;;
    integration)
        run_integration_tests
        ;;
    routes)
        run_route_tests
        ;;
    us1|us2|us3|us4|us5|us6)
        run_user_story "$1"
        ;;
    security)
        run_security_tests
        ;;
    debug)
        run_debug
        ;;
    help|-h|--help)
        print_usage
        ;;
    *)
        echo -e "${RED}❌ Unknown command: $1${NC}"
        echo ""
        print_usage
        exit 1
        ;;
esac

echo ""

