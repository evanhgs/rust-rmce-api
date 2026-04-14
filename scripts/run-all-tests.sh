#!/bin/bash

# RMCE API - Run All Tests
# Ensures database is running and migrations are applied before running tests

set -e

echo "╔════════════════════════════════════════════════════════════════╗"
echo "║                                                                ║"
echo "║   🧪  RMCE API - Test Runner                                  ║"
echo "║                                                                ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# Check if .env exists and source it
if [ ! -f .env ]; then
    echo -e "${RED}❌ .env file not found${NC}"
    echo "   Run ./setup-project.sh first"
    exit 1
fi

export $(cat .env | grep -v '^#' | xargs)

# Check if Docker container is running
if ! docker ps | grep -q rmce_db; then
    echo -e "${YELLOW}⚠️  PostgreSQL container not running${NC}"
    echo "   Starting Docker containers..."
    docker compose up -d
    sleep 5
    echo -e "${GREEN}✓  PostgreSQL started${NC}"
fi

# Wait for PostgreSQL to be ready
echo -e "${BLUE}🔍 Checking PostgreSQL connection...${NC}"
MAX_RETRIES=10
RETRY_COUNT=0
until docker exec rmce_db pg_isready -U postgres -d rmce_db > /dev/null 2>&1; do
    RETRY_COUNT=$((RETRY_COUNT+1))
    if [ $RETRY_COUNT -ge $MAX_RETRIES ]; then
        echo -e "${RED}❌ Cannot connect to PostgreSQL${NC}"
        exit 1
    fi
    echo -e "${YELLOW}   Waiting for PostgreSQL... ($RETRY_COUNT/$MAX_RETRIES)${NC}"
    sleep 2
done
echo -e "${GREEN}✓  PostgreSQL is ready${NC}"
echo ""

# Verify migrations are applied
echo -e "${BLUE}🔄 Checking migrations...${NC}"
sqlx migrate info --source api/migrations || {
    echo -e "${YELLOW}⚠️  Running migrations...${NC}"
    sqlx migrate run --source api/migrations
    echo -e "${GREEN}✓  Migrations completed${NC}"
}
echo ""

# Run tests
echo -e "${BLUE}🧪 Running integration tests...${NC}"
echo ""

cargo test --test integration_tests -p rust-rmce-api -- --test-threads=1 --nocapture

TEST_RESULT=$?

echo ""
if [ $TEST_RESULT -eq 0 ]; then
    echo "╔════════════════════════════════════════════════════════════════╗"
    echo "║                                                                ║"
    echo "║   ✅  All Tests Passed!                                        ║"
    echo "║                                                                ║"
    echo "╚════════════════════════════════════════════════════════════════╝"
else
    echo "╔════════════════════════════════════════════════════════════════╗"
    echo "║                                                                ║"
    echo "║   ❌  Some Tests Failed                                        ║"
    echo "║                                                                ║"
    echo "╚════════════════════════════════════════════════════════════════╝"
    echo ""
    echo "To debug:"
    echo "  1. Check server logs:  ${BLUE}RUST_LOG=debug cargo run${NC}"
    echo "  2. Check database:     ${BLUE}docker exec -it rmce_db psql -U postgres -d rmce_db${NC}"
    echo "  3. View migrations:    ${BLUE}sqlx migrate info${NC}"
fi

exit $TEST_RESULT

