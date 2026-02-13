# 🏃 RMCE API - Running Chronometer API

A high-performance REST API built with Rust for a mobile running chronometer application. Features include route tracking, performance metrics, social features, challenges, and real-time sensor data collection.

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-9%2F9%20passing-brightgreen.svg)](tests/)

## 📋 Table of Contents

- [Features](#-features)
- [Tech Stack](#-tech-stack)
- [Prerequisites](#-prerequisites)
- [Quick Start](#-quick-start)
- [API Routes](#-api-routes)
- [Testing](#-testing)
- [Production Deployment](#-production-deployment)
- [Development](#-development)
- [Documentation](#-documentation)

## ✨ Features

- **🔐 JWT Authentication** - Secure token-based authentication
- **🗺️ Route Management** - Create, share, and manage running routes with GeoJSON support
- **📊 Performance Tracking** - Record times, speeds, G-force, inclination, and more
- **👥 Social Features** - Friend system with requests and acceptance
- **🏆 Leaderboards** - Route-specific and global speed rankings
- **🎯 Challenges** - Competitive features with 1v1 or open challenges
- **📱 Sensor Data** - Collect accelerometer, gyroscope, GPS, and environmental data
- **⚡ Real-time** - Async/await throughout for high performance
- **🛡️ Type-safe** - Rust's type system prevents entire categories of bugs

## 🛠 Tech Stack

- **Language**: Rust 1.70+
- **Web Framework**: Axum 0.8
- **Database**: PostgreSQL 12+
- **ORM**: SQLx with compile-time query verification
- **Authentication**: JWT (jsonwebtoken)
- **Password Hashing**: bcrypt
- **Async Runtime**: Tokio
- **Serialization**: Serde

## 📦 Prerequisites

Before you begin, ensure you have the following installed:

- **Rust** 1.70 or higher
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

- **Docker** and **Docker Compose** (for PostgreSQL)
  ```bash
  # Verify installation
  docker --version
  docker compose version
  ```

- **SQLx CLI** (for migrations)
  ```bash
  cargo install sqlx-cli --no-default-features --features postgres
  ```

## 🚀 Quick Start

### Option 1: Automated Setup (Recommended)

Run the automated setup script that handles everything:

```bash
bash scripts/setup-project.sh
```

This will:
1. ✅ Start PostgreSQL in Docker
2. ✅ Create the database
3. ✅ Run all migrations
4. ✅ Build the project

### Option 2: Manual Setup

If you prefer manual control:

#### 1. Clone and Configure

```bash
git clone <repository-url>
cd rust-rmce-api

# Copy environment configuration
cp .env.example .env
```

#### 2. Start PostgreSQL

```bash
docker compose up -d
```

#### 3. Run Migrations

```bash
sqlx migrate run
```

#### 4. Build and Run

```bash
# Development mode
cargo run

# Production build
cargo build --release
./target/release/rust-rmce-api
```

The API will be available at `http://localhost:3000`

## 📍 API Routes

### Authentication (Public)

| Method | Endpoint | Description |
|--------|----------|-------------|
| `POST` | `/auth/register` | Register a new user |
| `POST` | `/auth/login` | Login and receive JWT token |

#### Example: Register
```bash
curl -X POST http://localhost:3000/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "runner",
    "email": "runner@example.com",
    "password": "SecurePass123!"
  }'
```

#### Example: Login
```bash
curl -X POST http://localhost:3000/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "runner@example.com",
    "password": "SecurePass123!"
  }'

# Response includes JWT token:
# {
#   "token": "eyJ0eXAiOiJKV1QiLCJh...",
#   "user": { "id": 1, "username": "runner", "email": "runner@example.com" }
# }
```

### Routes Management (Protected)

All routes require `Authorization: Bearer <token>` header.

| Method | Endpoint | Description |
|--------|----------|-------------|
| `POST` | `/routes` | Create a new route |
| `GET` | `/routes` | List all routes (with filters) |
| `GET` | `/routes/{id}` | Get route details |
| `PUT` | `/routes/{id}` | Update route (owner only) |
| `DELETE` | `/routes/{id}` | Delete route (owner only) |
| `GET` | `/routes/user/{user_id}` | Get user's routes |
| `GET` | `/routes/public` | List public routes |
| `POST` | `/routes/{id}/score` | Submit time/score for route |

#### Example: Create Route
```bash
curl -X POST http://localhost:3000/routes \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Central Park Loop",
    "description": "5km loop around the park",
    "is_public": true,
    "path_data": {
      "type": "LineString",
      "coordinates": [[2.3522, 48.8566], [2.3523, 48.8567]]
    },
    "distance_meters": 5000.0
  }'
```

#### Example: Submit Score
```bash
curl -X POST http://localhost:3000/routes/1/score \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "time_seconds": 1800.5,
    "max_speed_kmh": 18.5,
    "avg_speed_kmh": 15.0,
    "max_g_force": 1.2,
    "max_inclination_degrees": 8.5,
    "max_sound_db": 85.0
  }'
```

### Friends Management (Protected)

| Method | Endpoint | Description |
|--------|----------|-------------|
| `POST` | `/friends/add/{friend_id}` | Send friend request |
| `GET` | `/friends` | List accepted friends |
| `GET` | `/friends/pending` | View pending requests |
| `PUT` | `/friends/accept/{friendship_id}` | Accept friend request |
| `PUT` | `/friends/reject/{friendship_id}` | Reject friend request |

### Challenges & Leaderboards (Protected)

| Method | Endpoint | Description |
|--------|----------|-------------|
| `POST` | `/api/challenges` | Create a challenge |
| `GET` | `/api/challenges/{id}` | Get challenge details |
| `POST` | `/api/challenges/{id}/accept` | Accept challenge |
| `POST` | `/api/challenges/{id}/complete` | Complete challenge |
| `GET` | `/api/challenges/available` | List open challenges |
| `GET` | `/api/leaderboard/route/{id}` | Route leaderboard |
| `GET` | `/api/leaderboard/global/speed` | Global speed leaderboard |

### Sensor Data (Protected)

| Method | Endpoint | Description |
|--------|----------|-------------|
| `POST` | `/sensor-data/{score_id}` | Upload single sensor data point |
| `POST` | `/sensor-data/bulk` | Bulk upload (transactional) |
| `GET` | `/sensor-data/score/{score_id}` | Retrieve sensor data |

#### Example: Bulk Sensor Upload
```bash
curl -X POST http://localhost:3000/sensor-data/bulk \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "score_id": 1,
    "data": [
      {
        "timestamp_offset_ms": 0,
        "accel_x": 0.1, "accel_y": 0.2, "accel_z": 9.8,
        "gyro_x": 0.01, "gyro_y": 0.01, "gyro_z": 0.0,
        "orientation_azimuth": 180.0,
        "orientation_pitch": 5.0,
        "orientation_roll": 0.0,
        "speed_kmh": 12.5,
        "g_force": 1.0,
        "inclination_degrees": 2.5,
        "sound_db": 70.0,
        "nearby_devices": 3,
        "latitude": 48.8566,
        "longitude": 2.3522,
        "altitude": 35.0
      }
    ]
  }'
```

## 🧪 Testing

### Run All Tests (Recommended)

```bash
bash scripts/run-all-tests.sh
```

This automated script:
- ✅ Checks PostgreSQL is running
- ✅ Verifies migrations are applied
- ✅ Runs all 9 integration tests

### Manual Testing

```bash
# All tests with single thread (prevents conflicts)
cargo test -- --test-threads=1

# Integration tests only
cargo test --test integration_tests -- --test-threads=1

# With debug output
RUST_LOG=debug cargo test -- --nocapture --test-threads=1

# Specific test
cargo test user_story_01 -- --nocapture
```

### Test Coverage

We have **9 comprehensive integration tests**:

**User Stories (6 tests)**
- ✅ Registration & Login with JWT
- ✅ Create & Manage Routes
- ✅ Submit Scores after Run
- ✅ Friend Management
- ✅ View Leaderboards
- ✅ Upload Sensor Data

**Security Tests (3 tests)**
- ✅ Unauthorized access blocked (401)
- ✅ Invalid token rejected (401)
- ✅ Route ownership enforced (403)

### Expected Output

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

test result: ok. 9 passed; 0 failed; 0 ignored
```

## 🚢 Production Deployment

### Environment Configuration

Create a `.env` file with production values:

```bash
DATABASE_URL=postgresql://user:password@db-host:5432/rmce_db
JWT_SECRET=your-super-secret-key-minimum-32-characters
RUST_LOG=info
```

⚠️ **Security Checklist**:
- [ ] Change `JWT_SECRET` to a strong, random value
- [ ] Use HTTPS/TLS in production
- [ ] Configure CORS for your frontend domain
- [ ] Enable rate limiting
- [ ] Set up database backups
- [ ] Use environment variables for secrets (never commit)

### Docker Deployment

#### Build Production Image

```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libpq5 ca-certificates
COPY --from=builder /app/target/release/rust-rmce-api /usr/local/bin/
CMD ["rust-rmce-api"]
```

#### Deploy with Docker Compose

```yaml
version: '3.8'
services:
  api:
    build: .
    ports:
      - "3000:3000"
    environment:
      - DATABASE_URL=postgresql://postgres:password@db:5432/rmce_db
      - JWT_SECRET=${JWT_SECRET}
    depends_on:
      - db
  
  db:
    image: postgres:15-alpine
    environment:
      POSTGRES_DB: rmce_db
      POSTGRES_PASSWORD: ${DB_PASSWORD}
    volumes:
      - postgres_data:/var/lib/postgresql/data

volumes:
  postgres_data:
```

### Cloud Platforms

#### Railway.app
```bash
# Install Railway CLI
npm i -g @railway/cli

# Deploy
railway login
railway init
railway up
```

#### Render.com
1. Connect your GitHub repository
2. Set environment variables in dashboard
3. Deploy automatically on push

#### Heroku
```bash
# Create app
heroku create rmce-api

# Add PostgreSQL
heroku addons:create heroku-postgresql:mini

# Deploy
git push heroku main
```

### Database Migrations

Always run migrations before starting:

```bash
# Production
sqlx migrate run

# Rollback if needed
sqlx migrate revert
```

### Health Check Endpoint

The API includes a health check at `/`:

```bash
curl http://localhost:3000/
# Response: "OK"
```

## 💻 Development

### Project Structure

```
rust-rmce-api/
├── src/
│   ├── main.rs              # Server entry point
│   ├── lib.rs               # Library exports
│   ├── db.rs                # Database pool
│   ├── middleware.rs        # JWT middleware
│   ├── models/              # Data models (8 files)
│   └── routes/              # API endpoints (8 handlers)
├── migrations/              # SQL migrations (9 files)
├── tests/                   # Integration tests
├── scripts/                 # Helper scripts
├── docs/                    # Documentation
└── Cargo.toml              # Dependencies
```

### Adding a New Migration

```bash
sqlx migrate add create_my_table
# Edit the generated file in migrations/
sqlx migrate run
```

### Running in Development

```bash
# With hot reload (install cargo-watch)
cargo install cargo-watch
cargo watch -x run

# With debug logs
RUST_LOG=debug cargo run

# Check compilation
cargo check

# Format code
cargo fmt

# Lint code
cargo clippy
```

### Database Management

```bash
# Connect to database
docker exec -it rmce_db psql -U postgres -d rmce_db

# View tables
\dt

# View specific data
SELECT * FROM users;
SELECT * FROM routes;

# Exit
\q
```

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection string | `postgresql://postgres:password@localhost:5432/rmce_db` |
| `JWT_SECRET` | Secret key for JWT signing | `your_jwt_secret_key` |
| `RUST_LOG` | Logging level | `info` |
| `PORT` | Server port | `3000` |

## 📚 Documentation

Additional documentation is available in the `docs/` directory:

- **[API_DOCUMENTATION.md](docs/API_DOCUMENTATION.md)** - Complete API reference
- **[TESTING_GUIDE.md](docs/TESTING_GUIDE.md)** - Detailed testing documentation
- **[JWT_IMPLEMENTATION.md](docs/JWT_IMPLEMENTATION.md)** - Authentication details
- **[QUICK_START.md](docs/QUICK_START.md)** - Quick reference guide
- **[RUNNING_TESTS.md](docs/RUNNING_TESTS.md)** - Test setup and troubleshooting

## 🤝 Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Run tests (`bash scripts/run-all-tests.sh`)
4. Commit your changes (`git commit -m 'Add amazing feature'`)
5. Push to the branch (`git push origin feature/amazing-feature`)
6. Open a Pull Request

## 📝 License

This project is licensed under the MIT License.

## 🆘 Support

- **Issues**: [GitHub Issues](https://github.com/your-repo/issues)
- **Documentation**: Check the `docs/` folder
- **Logs**: Run with `RUST_LOG=debug` for detailed logs

## 🎯 API Status

- ✅ **26+ Endpoints** - All functional
- ✅ **9/9 Tests Passing** - Full coverage
- ✅ **JWT Authentication** - Secure
- ✅ **Production Ready** - Deployed and tested

---

**Built with ❤️ using Rust and Axum**

Last Updated: February 13, 2026 | Version: 1.0.0
