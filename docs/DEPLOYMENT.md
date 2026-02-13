# 🚀 Production Deployment Guide

Complete guide for deploying RMCE API to production environments.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Environment Configuration](#environment-configuration)
- [Docker Deployment](#docker-deployment)
- [Cloud Platforms](#cloud-platforms)
- [Security Checklist](#security-checklist)
- [Monitoring](#monitoring)
- [Troubleshooting](#troubleshooting)

## Prerequisites

- Docker and Docker Compose installed
- PostgreSQL database (or Docker for hosting)
- Domain name (optional but recommended)
- SSL/TLS certificate (Let's Encrypt recommended)

## Environment Configuration

### 1. Create Production Environment File

Create a `.env.prod` file (never commit this):

```bash
# Database
DATABASE_URL=postgresql://postgres:STRONG_PASSWORD@db:5432/rmce_db
POSTGRES_PASSWORD=STRONG_PASSWORD

# JWT Secret (generate with: openssl rand -base64 64)
JWT_SECRET=your-super-secret-jwt-key-minimum-64-characters-long

# Application
RUST_LOG=info
PORT=3000

# Optional: Monitoring
SENTRY_DSN=https://your-sentry-dsn
```

### 2. Generate Strong Secrets

```bash
# Generate JWT secret
openssl rand -base64 64

# Generate database password
openssl rand -base64 32
```

## Docker Deployment

### Option 1: Using Docker Compose (Recommended)

#### Build and Start

```bash
# Build the image
docker compose -f docker-compose.prod.yml build

# Start services
docker compose -f docker-compose.prod.yml up -d

# View logs
docker compose -f docker-compose.prod.yml logs -f api
```

#### Run Migrations

```bash
# Access the API container
docker compose -f docker-compose.prod.yml exec api /bin/bash

# Or run migrations from host
docker compose -f docker-compose.prod.yml exec api sqlx migrate run
```

#### Stop Services

```bash
docker compose -f docker-compose.prod.yml down
```

### Option 2: Manual Docker Deployment

#### Build Image

```bash
docker build -t rmce-api:latest .
```

#### Run PostgreSQL

```bash
docker run -d \
  --name rmce_db \
  -e POSTGRES_DB=rmce_db \
  -e POSTGRES_PASSWORD=your_password \
  -v postgres_data:/var/lib/postgresql/data \
  -p 5432:5432 \
  postgres:15-alpine
```

#### Run API

```bash
docker run -d \
  --name rmce_api \
  -e DATABASE_URL=postgresql://postgres:your_password@rmce_db:5432/rmce_db \
  -e JWT_SECRET=your_jwt_secret \
  -e RUST_LOG=info \
  -p 3000:3000 \
  --link rmce_db:db \
  rmce-api:latest
```

### Configure Reverse Proxy (Nginx)

Create `/etc/nginx/sites-available/rmce-api`:

```nginx
server {
    listen 80;
    server_name api.yourdomain.com;

    location / {
        proxy_pass http://localhost:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

Enable and configure SSL:

```bash
# Enable site
sudo ln -s /etc/nginx/sites-available/rmce-api /etc/nginx/sites-enabled/

# Get SSL certificate
sudo certbot --nginx -d api.yourdomain.com

# Reload Nginx
sudo nginx -t && sudo systemctl reload nginx
```

## Cloud Platforms

### Railway.app

1. **Install Railway CLI**:
   ```bash
   npm i -g @railway/cli
   ```

2. **Deploy**:
   ```bash
   railway login
   railway init
   railway up
   ```

3. **Add PostgreSQL**:
   ```bash
   railway add
   # Select PostgreSQL
   ```

4. **Set Environment Variables**:
   ```bash
   railway variables set JWT_SECRET=your_secret
   railway variables set RUST_LOG=info
   ```

### Render.com

1. **Connect GitHub Repository**
   - Go to Render.com dashboard
   - Click "New +" → "Web Service"
   - Connect your GitHub repository

2. **Configure Build**:
   - **Build Command**: `cargo build --release`
   - **Start Command**: `./target/release/rust-rmce-api`

3. **Add PostgreSQL**:
   - Click "New +" → "PostgreSQL"
   - Note the connection string

4. **Set Environment Variables**:
   - `DATABASE_URL`: (from PostgreSQL)
   - `JWT_SECRET`: your secret
   - `RUST_LOG`: info

### Heroku

1. **Create App**:
   ```bash
   heroku create rmce-api
   ```

2. **Add PostgreSQL**:
   ```bash
   heroku addons:create heroku-postgresql:mini
   ```

3. **Set Environment Variables**:
   ```bash
   heroku config:set JWT_SECRET=your_secret
   heroku config:set RUST_LOG=info
   ```

4. **Deploy**:
   ```bash
   git push heroku main
   ```

5. **Run Migrations**:
   ```bash
   heroku run sqlx migrate run
   ```

### AWS EC2

1. **Launch EC2 Instance**:
   - Ubuntu 22.04 LTS
   - t2.micro or larger
   - Configure security group (ports 22, 80, 443, 3000)

2. **Install Dependencies**:
   ```bash
   # Connect to instance
   ssh -i your-key.pem ubuntu@your-instance-ip

   # Install Docker
   curl -fsSL https://get.docker.com -o get-docker.sh
   sudo sh get-docker.sh

   # Install Docker Compose
   sudo apt-get update
   sudo apt-get install docker-compose-plugin
   ```

3. **Deploy Application**:
   ```bash
   # Clone repository
   git clone your-repo-url
   cd rust-rmce-api

   # Create .env.prod
   nano .env.prod
   # Add your configuration

   # Start services
   docker compose -f docker-compose.prod.yml up -d
   ```

### DigitalOcean

1. **Create Droplet**:
   - Ubuntu 22.04
   - Choose size (minimum 1GB RAM)
   - Add SSH key

2. **Setup Application**:
   ```bash
   # SSH into droplet
   ssh root@your-droplet-ip

   # Install Docker
   curl -fsSL https://get.docker.com | sh

   # Clone and deploy
   git clone your-repo
   cd rust-rmce-api
   docker compose -f docker-compose.prod.yml up -d
   ```

## Security Checklist

### Essential Security Measures

- [ ] **Change default passwords** - Never use default credentials
- [ ] **Use strong JWT secret** - Minimum 64 characters, randomly generated
- [ ] **Enable HTTPS/TLS** - Use Let's Encrypt or cloud provider SSL
- [ ] **Configure CORS** - Restrict to your frontend domain only
- [ ] **Enable rate limiting** - Implement in reverse proxy or application
- [ ] **Set up firewall** - Only open necessary ports (80, 443)
- [ ] **Regular updates** - Keep dependencies and OS updated
- [ ] **Database backups** - Automated daily backups with retention
- [ ] **Environment variables** - Never commit secrets to repository
- [ ] **Monitoring** - Set up error tracking and performance monitoring

### CORS Configuration

Add to your Nginx config:

```nginx
add_header 'Access-Control-Allow-Origin' 'https://yourfrontend.com' always;
add_header 'Access-Control-Allow-Methods' 'GET, POST, PUT, DELETE, OPTIONS' always;
add_header 'Access-Control-Allow-Headers' 'Authorization, Content-Type' always;
```

### Rate Limiting (Nginx)

```nginx
limit_req_zone $binary_remote_addr zone=api_limit:10m rate=10r/s;

server {
    location /api {
        limit_req zone=api_limit burst=20 nodelay;
        proxy_pass http://localhost:3000;
    }
}
```

## Database Management

### Backup PostgreSQL

```bash
# Backup
docker exec rmce_db pg_dump -U postgres rmce_db > backup.sql

# Restore
docker exec -i rmce_db psql -U postgres rmce_db < backup.sql
```

### Automated Backups (Cron)

```bash
# Add to crontab (crontab -e)
0 2 * * * docker exec rmce_db pg_dump -U postgres rmce_db | gzip > /backups/rmce_$(date +\%Y\%m\%d).sql.gz

# Keep only last 7 days
0 3 * * * find /backups -name "rmce_*.sql.gz" -mtime +7 -delete
```

## Monitoring

### Health Checks

```bash
# Check API health
curl http://localhost:3000/

# Check database connection
docker exec rmce_db pg_isready -U postgres -d rmce_db
```

### Logging

```bash
# View API logs
docker compose -f docker-compose.prod.yml logs -f api

# View database logs
docker compose -f docker-compose.prod.yml logs -f db

# View last 100 lines
docker compose -f docker-compose.prod.yml logs --tail=100 api
```

### Performance Monitoring

Consider integrating:
- **Sentry** - Error tracking
- **Datadog** - Application performance monitoring
- **Prometheus + Grafana** - Metrics and dashboards

## Troubleshooting

### API Won't Start

1. **Check logs**:
   ```bash
   docker compose -f docker-compose.prod.yml logs api
   ```

2. **Verify database connection**:
   ```bash
   docker compose -f docker-compose.prod.yml exec api /bin/bash
   psql $DATABASE_URL
   ```

3. **Check environment variables**:
   ```bash
   docker compose -f docker-compose.prod.yml exec api env
   ```

### Database Connection Issues

1. **Verify PostgreSQL is running**:
   ```bash
   docker compose -f docker-compose.prod.yml ps
   ```

2. **Check network connectivity**:
   ```bash
   docker compose -f docker-compose.prod.yml exec api ping db
   ```

3. **Test direct connection**:
   ```bash
   docker compose -f docker-compose.prod.yml exec db psql -U postgres -d rmce_db
   ```

### High Memory Usage

1. **Check container stats**:
   ```bash
   docker stats
   ```

2. **Increase memory limits** in docker-compose.prod.yml:
   ```yaml
   services:
     api:
       deploy:
         resources:
           limits:
             memory: 512M
   ```

### SSL Certificate Issues

1. **Renew Let's Encrypt certificate**:
   ```bash
   sudo certbot renew
   ```

2. **Test SSL configuration**:
   ```bash
   sudo nginx -t
   ```

## Scaling

### Horizontal Scaling

Use Docker Swarm or Kubernetes for multiple instances:

```bash
# Docker Swarm
docker swarm init
docker stack deploy -c docker-compose.prod.yml rmce

# Scale API service
docker service scale rmce_api=3
```

### Load Balancing

Configure Nginx for load balancing:

```nginx
upstream api_backend {
    least_conn;
    server api1:3000;
    server api2:3000;
    server api3:3000;
}

server {
    location / {
        proxy_pass http://api_backend;
    }
}
```

## Maintenance

### Update Application

```bash
# Pull latest changes
git pull origin main

# Rebuild and restart
docker compose -f docker-compose.prod.yml build
docker compose -f docker-compose.prod.yml up -d

# Run new migrations
docker compose -f docker-compose.prod.yml exec api sqlx migrate run
```

### Update Dependencies

```bash
# Update Cargo dependencies
cargo update

# Rebuild Docker image
docker compose -f docker-compose.prod.yml build --no-cache
```

## Support

- **Documentation**: Check `docs/` directory
- **Issues**: Report on GitHub
- **Logs**: Always check with `RUST_LOG=debug` for detailed information

---

**Last Updated**: February 13, 2026

