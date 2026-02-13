# 🚀 QUICK START GUIDE - RMCE API

## 📋 Table of Contents
1. [Installation](#installation)
2. [Running Tests](#running-tests)
3. [API Documentation](#api-documentation)
4. [Common Commands](#common-commands)

---

## 🔧 Installation

### Prerequisites
```bash
# Check Rust is installed
rustc --version  # Should be 1.70+
cargo --version

# Check PostgreSQL is running
psql --version
```

### First Time Setup
```bash
cd /home/evan/work/school/rust-rmce-api

# Create environment file
cat > .env << EOF
DATABASE_URL=postgresql://postgres:postgres@localhost/rust-rmce-api
RUST_LOG=info
EOF

# Create database
createdb rust-rmce-api

# Run migrations
sqlx migrate run

# Build project
cargo build

# Verify compilation
cargo check
```

### Using Docker (Alternative)
```bash
# Start PostgreSQL in Docker
docker-compose up -d

# Run migrations
sqlx migrate run

# Start the API
cargo run
```

---

## 🧪 Running Tests

### Quick Start
```bash
# Run all tests
./run-tests.sh all

# Run integration tests only
./run-tests.sh integration

# Run with debug logs
./run-tests.sh debug
```

### Individual Tests
```bash
# User Story 1 - Registration & Login
./run-tests.sh us1

# User Story 2 - Create Routes
./run-tests.sh us2

# User Story 3 - Submit Scores
./run-tests.sh us3

# User Story 4 - Friend Management
./run-tests.sh us4

# User Story 5 - View Leaderboard
./run-tests.sh us5

# User Story 6 - Upload Sensor Data
./run-tests.sh us6

# Security Tests
./run-tests.sh security
```

### Manual Commands
```bash
# All tests
cargo test -- --test-threads=1

# Integration tests only
cargo test --test integration_tests -- --test-threads=1

# Route tests only
cargo test --test routes -- --test-threads=1

# With logging
RUST_LOG=debug cargo test -- --nocapture --test-threads=1

# Specific test with output
cargo test user_story_01 -- --nocapture
```

---

## 🏃 Running the API Server

### Development
```bash
# Start with logging
RUST_LOG=debug cargo run

# Start normally
cargo run

# Server will be available at http://localhost:3000
```

### Release Build
```bash
# Optimized build
cargo build --release

# Run release binary
./target/release/rust-rmce-api
```

---

## 📚 API Documentation

### Available Documentation Files
```bash
# Full API reference
cat API_DOCUMENTATION.md

# Testing guide
cat TESTING_GUIDE.md

# JWT & Security details
cat JWT_IMPLEMENTATION.md

# Implementation details
cat IMPLEMENTATION_SUMMARY.md

# Completion report
cat FINAL_COMPLETION_REPORT.md
```

---

## 🔑 Authentication Example

### Register User
```bash
curl -X POST http://localhost:3000/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "alice",
    "email": "alice@example.com",
    "password": "SecurePass123!"
  }'
```

### Login
```bash
curl -X POST http://localhost:3000/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "alice@example.com",
    "password": "SecurePass123!"
  }'

# Response contains JWT token
# Copy the token value
```

### Use Token in Requests
```bash
TOKEN="your-jwt-token-here"

# Create a route
curl -X POST http://localhost:3000/routes \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "My Route",
    "description": "5km loop",
    "is_public": true,
    "path_data": {"type": "LineString", "coordinates": [[0,0], [1,1]]},
    "distance_meters": 5000.0
  }'
```

---

## 📊 Common Commands

### Project Management
```bash
# Format code
cargo fmt

# Check for issues
cargo clippy

# Check compilation
cargo check

# Build for debugging
cargo build

# Build for production
cargo build --release

# Clean build artifacts
cargo clean
```

### Database
```bash
# Show migrations status
sqlx migrate list

# Run pending migrations
sqlx migrate run

# Rollback last migration
sqlx migrate revert

# Connect to database
psql rust-rmce-api

# Show tables in database
\dt
```

### Testing
```bash
# Run tests in verbose mode
cargo test -- --nocapture --test-threads=1 --show-output

# Run a specific test function
cargo test test_name -- --nocapture

# List all available tests
cargo test -- --list

# Run with specific log level
RUST_LOG=trace cargo test -- --nocapture --test-threads=1
```

---

## 📱 Testing with Postman/Insomnia

### 1. Register
```
POST http://localhost:3000/auth/register
Content-Type: application/json

{
  "username": "testuser",
  "email": "test@example.com",
  "password": "TestPass123!"
}
```

### 2. Login
```
POST http://localhost:3000/auth/login
Content-Type: application/json

{
  "email": "test@example.com",
  "password": "TestPass123!"
}
```

### 3. Copy token from response

### 4. Create Route (with token)
```
POST http://localhost:3000/routes
Authorization: Bearer <YOUR_TOKEN>
Content-Type: application/json

{
  "name": "Test Route",
  "description": "A test running route",
  "is_public": true,
  "path_data": {
    "type": "LineString",
    "coordinates": [[2.3522, 48.8566], [2.3523, 48.8567]]
  },
  "distance_meters": 5000.0
}
```

### 5. Submit Score
```
POST http://localhost:3000/routes/1/score
Authorization: Bearer <YOUR_TOKEN>
Content-Type: application/json

{
  "time_seconds": 1800.0,
  "max_speed_kmh": 18.5,
  "avg_speed_kmh": 15.0,
  "max_g_force": 1.2,
  "max_inclination_degrees": 8.5,
  "max_sound_db": 85.0
}
```

---

## 🐛 Troubleshooting

### Issue: Tests keep getting skipped
**Solution:** Set DATABASE_URL
```bash
export DATABASE_URL=postgresql://postgres:postgres@localhost/rust-rmce-api
cargo test
```

### Issue: "connection refused" error
**Solution:** Make sure PostgreSQL is running
```bash
# Check if running
psql postgres -c "SELECT 1;"

# Or start with Docker
docker-compose up -d
```

### Issue: Tests run in parallel and conflict
**Solution:** Use single thread
```bash
cargo test -- --test-threads=1
```

### Issue: Can't see test output
**Solution:** Use nocapture flag
```bash
cargo test -- --nocapture --test-threads=1
```

### Issue: Port 3000 already in use
**Solution:** Kill process or use different port
```bash
# Find process on port 3000
lsof -i :3000

# Kill it
kill -9 <PID>
```

---

## ✅ Verification Checklist

Run this to verify everything is working:

```bash
#!/bin/bash

echo "1. Checking Rust..."
cargo --version

echo "2. Checking database..."
createdb rust-rmce-api 2>/dev/null || echo "   Database exists ✓"

echo "3. Building project..."
cargo build 2>&1 | tail -1

echo "4. Running tests..."
cargo test --test integration_tests -- --test-threads=1 2>&1 | tail -5

echo ""
echo "✅ System ready for development!"
```

---

## 📖 Documentation Map

| Document | Content |
|----------|---------|
| `API_DOCUMENTATION.md` | All endpoints, parameters, responses |
| `TESTING_GUIDE.md` | How tests work, user stories, coverage |
| `JWT_IMPLEMENTATION.md` | Authentication, security, examples |
| `IMPLEMENTATION_SUMMARY.md` | What was built, architecture |
| `FINAL_COMPLETION_REPORT.md` | Complete project summary |
| `README.md` | Project overview |
| `run-tests.sh` | Test automation script |

---

## 🎯 Development Workflow

### Daily Development
```bash
# Start server in one terminal
cargo run

# In another terminal, run tests
cargo test -- --test-threads=1 --nocapture

# Make changes, watch tests pass
cargo watch -x "test -- --test-threads=1"
```

### Before Committing
```bash
# Format code
cargo fmt

# Check for issues
cargo clippy

# Run all tests
cargo test -- --test-threads=1

# Build release
cargo build --release
```

### Debugging
```bash
# Run with full logging
RUST_LOG=debug cargo run

# Test with full output
RUST_LOG=debug cargo test user_story_01 -- --nocapture --test-threads=1
```

---

## 🚀 Deployment

### Build Optimized Binary
```bash
cargo build --release

# Binary location
./target/release/rust-rmce-api
```

### Docker Build
```bash
# Build image
docker build -t rmce-api:latest .

# Run container
docker run -p 3000:3000 -e DATABASE_URL=... rmce-api:latest
```

### Environment Variables Required
```bash
DATABASE_URL=postgresql://user:password@host/database
RUST_LOG=info
JWT_SECRET=your-secret-key
```

---

## 📞 Support

### Check Logs
```bash
# Server logs with timestamps
RUST_LOG=debug cargo run 2>&1 | tee server.log

# Test logs with output
RUST_LOG=debug cargo test -- --nocapture --test-threads=1
```

### Run Specific Tests
```bash
# User Story 1
cargo test user_story_01 -- --nocapture

# Security Test
cargo test security_test_unauthorized -- --nocapture

# All Route tests
cargo test --test routes -- --nocapture
```

### Database Inspection
```bash
# Connect to database
psql rust-rmce-api

# Show all tables
\dt

# Show users
SELECT * FROM users;

# Show routes
SELECT * FROM routes;

# Exit
\q
```

---

## 🎓 Learning Path

1. **Start Here**: Read `FINAL_COMPLETION_REPORT.md`
2. **Understand API**: Read `API_DOCUMENTATION.md`
3. **Learn Tests**: Read `TESTING_GUIDE.md`
4. **Study Security**: Read `JWT_IMPLEMENTATION.md`
5. **Explore Code**: Check `src/routes/` and `tests/`

---

## ✨ Quick Reference

| Task | Command |
|------|---------|
| Build | `cargo build` |
| Test All | `./run-tests.sh all` |
| Test User Stories | `./run-tests.sh us1-6` |
| Test Security | `./run-tests.sh security` |
| Format Code | `cargo fmt` |
| Lint Code | `cargo clippy` |
| Run Server | `cargo run` |
| Database | `psql rust-rmce-api` |
| Migrations | `sqlx migrate run` |

---

## 📅 Project Status

✅ **Complete & Production Ready**
- 26 endpoints implemented
- 9 integration tests
- 3 security tests
- JWT authentication
- Full documentation
- Ready for Flutter integration

---

**Last Updated:** February 13, 2026
**Version:** 1.0.0 (MVP)


