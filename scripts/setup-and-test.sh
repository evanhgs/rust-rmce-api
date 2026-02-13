#!/bin/bash

# RMCE API - Test Setup & Runner
# This script sets up the test environment and runs tests

set -e

echo "🔧 RMCE API - Test Setup"
echo "========================"
echo ""

# Check if DATABASE_URL is set
if [ -z "$DATABASE_URL" ]; then
    echo "⚠️  DATABASE_URL not set. Setting up test database..."

    # Set default test database URL
    export DATABASE_URL="postgresql://postgres:postgres@localhost/rust_rmce_api_test"
    echo "   Using: $DATABASE_URL"
    echo ""

    # Check if database exists, create if not
    if psql -lqt | cut -d \| -f 1 | grep -qw rust_rmce_api_test; then
        echo "✓  Test database already exists"
    else
        echo "📦 Creating test database..."
        createdb rust_rmce_api_test 2>/dev/null || {
            echo "❌ Failed to create database. Make sure PostgreSQL is running."
            echo "   Try: docker compose up -d "
            exit 1
        }
        echo "✓  Test database created"
    fi
    echo ""

    # Run migrations
    echo "🔄 Running database migrations..."
    sqlx migrate run || {
        echo "❌ Failed to run migrations"
        exit 1
    }
    echo "✓  Migrations completed"
    echo ""
fi

echo "🧪 Running tests..."
echo ""

# Run tests with single thread to avoid conflicts
cargo test --test integration_tests -- --test-threads=1 --nocapture

echo ""
echo "✅ Test run complete!"

