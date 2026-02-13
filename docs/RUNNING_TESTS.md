# 🧪 Running Tests - Quick Guide

## Prerequisites

1. **PostgreSQL** must be running:
```bash
# Check if PostgreSQL is running
psql --version

# If not running, start it:
sudo systemctl start postgresql   # Linux
brew services start postgresql    # macOS
```

2. **Database** must be created and migrations run

## Option 1: Automatic Setup (Recommended)

Run the automated setup script:

```bash
./setup-and-test.sh
```

This script will:
- ✅ Check if DATABASE_URL is set
- ✅ Create test database if needed
- ✅ Run all migrations
- ✅ Run all tests

## Option 2: Manual Setup

### Step 1: Set Database URL

```bash
export DATABASE_URL=postgresql://postgres:postgres@localhost/rust_rmce_api_test
```

### Step 2: Create Database

```bash
createdb rust_rmce_api_test
```

### Step 3: Run Migrations

```bash
sqlx migrate run
```

### Step 4: Run Tests

```bash
# All tests
cargo test --test integration_tests -- --test-threads=1

# Specific test
cargo test user_story_01 -- --nocapture

# With logs
RUST_LOG=debug cargo test --test integration_tests -- --nocapture --test-threads=1
```

## Understanding Test Failures

### "DATABASE_URL non définie"
- **Cause**: DATABASE_URL environment variable not set
- **Fix**: `export DATABASE_URL=postgresql://postgres:postgres@localhost/rust_rmce_api_test`

### "relation does not exist"
- **Cause**: Database tables not created
- **Fix**: Run `sqlx migrate run`

### "connection refused"
- **Cause**: PostgreSQL not running
- **Fix**: `sudo systemctl start postgresql`

### Tests conflict (409 Conflict)
- **Cause**: Tests creating duplicate usernames
- **Fix**: Tests now use unique timestamps - this should not happen anymore

### 500 Internal Server Error
- **Cause**: Database tables missing or server error
- **Fix**: Check server logs with `RUST_LOG=debug` and ensure migrations ran

## Test Structure

We have **9 tests total**:

### User Stories (6 tests)
1. `user_story_01` - Registration & Login ✅
2. `user_story_02` - Create & Manage Routes
3. `user_story_03` - Submit Scores
4. `user_story_04` - Friend Management
5. `user_story_05` - View Leaderboard
6. `user_story_06` - Upload Sensor Data

### Security Tests (3 tests)
7. `security_test_unauthorized_access_without_token` ✅
8. `security_test_invalid_token` ✅
9. `security_test_user_cannot_modify_others_route`

## Expected Output (All Passing)

```
running 9 tests
test security_test_invalid_token ... ok
test security_test_unauthorized_access_without_token ... ok
test security_test_user_cannot_modify_others_route ... ok
test user_story_01_registration_and_login ... ok
test user_story_02_create_and_manage_routes ... ok
test user_story_03_submit_score_after_run ... ok
test user_story_04_add_friend_and_manage_requests ... ok
test user_story_05_view_leaderboard ... ok
test user_story_06_upload_sensor_data ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured
```

## Cleaning Test Database

To reset the test database:

```bash
dropdb rust_rmce_api_test
createdb rust_rmce_api_test
sqlx migrate run
```

## Running Tests in CI/CD

For GitHub Actions or other CI:

```yaml
- name: Setup Database
  run: |
    createdb rust_rmce_api_test
    export DATABASE_URL=postgresql://postgres:postgres@localhost/rust_rmce_api_test
    sqlx migrate run

- name: Run Tests
  run: cargo test --test integration_tests -- --test-threads=1
  env:
    DATABASE_URL: postgresql://postgres:postgres@localhost/rust_rmce_api_test
```

## Troubleshooting

### Test hangs or times out
- Kill any running server: `pkill rust-rmce-api`
- Check for port conflicts: `lsof -i :3000`

### Permission denied on database
```bash
# Grant permissions
psql -c "ALTER USER postgres WITH PASSWORD 'postgres';"
```

### SQLx compile-time verification fails
```bash
# Generate sqlx-data.json offline mode
cargo sqlx prepare
```

## Quick Commands Reference

```bash
# Create database
createdb rust_rmce_api_test

# Drop database
dropdb rust_rmce_api_test

# Run migrations
sqlx migrate run

# Rollback last migration
sqlx migrate revert

# List migrations
sqlx migrate list

# Run all tests
./setup-and-test.sh

# Run specific test
cargo test user_story_01 -- --nocapture

# Run with debug logs
RUST_LOG=debug cargo test -- --nocapture --test-threads=1
```

## Need Help?

1. Check `TESTING_GUIDE.md` for detailed test documentation
2. Check `QUICK_START.md` for API setup
3. Check server logs: `RUST_LOG=debug cargo run`

**Last Updated**: February 13, 2026

