#!/bin/bash

# RMCE API - Complete Setup Script
# Sets up Docker, database, and runs migrations

set -e

echo "╔════════════════════════════════════════════════════════════════╗"
echo "║                                                                ║"
echo "║   🚀  RMCE API - Complete Setup                               ║"
echo "║                                                                ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Check if .env exists
if [ ! -f .env ]; then
    echo -e "${YELLOW}⚠️  .env file not found. Creating from .env.example...${NC}"
    cp .env.example .env
    echo -e "${GREEN}✓  .env file created${NC}"
    echo ""
fi

# Source the .env file
export $(cat .env | grep -v '^#' | xargs)

echo -e "${BLUE}📦 Step 1: Starting Docker containers...${NC}"
docker compose up -d

# Wait for PostgreSQL to be ready
echo -e "${BLUE}⏳ Waiting for PostgreSQL to be ready...${NC}"
sleep 5

MAX_RETRIES=30
RETRY_COUNT=0
until docker exec rmce_db pg_isready -U postgres -d rmce_db > /dev/null 2>&1; do
    RETRY_COUNT=$((RETRY_COUNT+1))
    if [ $RETRY_COUNT -ge $MAX_RETRIES ]; then
        echo -e "${RED}❌ PostgreSQL failed to start after $MAX_RETRIES attempts${NC}"
        exit 1
    fi
    echo -e "${YELLOW}   Waiting for PostgreSQL... ($RETRY_COUNT/$MAX_RETRIES)${NC}"
    sleep 2
done

echo -e "${GREEN}✓  PostgreSQL is ready${NC}"
echo ""

echo -e "${BLUE}🔄 Step 2: Running database migrations...${NC}"
sqlx migrate run

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓  Migrations completed successfully${NC}"
else
    echo -e "${RED}❌ Migrations failed${NC}"
    exit 1
fi
echo ""

echo -e "${BLUE}🏗️  Step 3: Building project...${NC}"
cargo build

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓  Build successful${NC}"
else
    echo -e "${RED}❌ Build failed${NC}"
    exit 1
fi
echo ""

echo "╔════════════════════════════════════════════════════════════════╗"
echo "║                                                                ║"
echo "║   ✅  Setup Complete!                                          ║"
echo "║                                                                ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""
echo -e "${GREEN}Your RMCE API is ready to use!${NC}"
echo ""
echo "Next steps:"
echo "  1. Run the API:     ${BLUE}cargo run${NC}"
echo "  2. Run tests:       ${BLUE}./run-all-tests.sh${NC}"
echo "  3. View logs:       ${BLUE}docker compose logs -f${NC}"
echo "  4. Stop database:   ${BLUE}docker compose down${NC}"
echo ""
echo "API will be available at: http://localhost:3000"
echo "Database is running on:   localhost:5432"
echo ""

