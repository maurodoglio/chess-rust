# Docker Deployment Guide

This guide explains how to deploy the chess-rust application using Docker and Docker Compose.

## Prerequisites

- Docker 20.10 or later
- Docker Compose v2.0 or later

## Quick Start

The simplest way to run the entire application is using Docker Compose:

```bash
# Build and start both backend and frontend
docker compose up --build

# Or run in detached mode
docker compose up -d --build
```

This will:
- Build the Rust backend and run it on port 3000
- Build the frontend and serve it with nginx on port 80
- Create a shared network for the services to communicate

Access the application at `http://localhost`

## Architecture

The Docker setup consists of two services:

### Backend Service
- **Image**: Built from root Dockerfile
- **Port**: 3000
- **Technology**: Rust application using Axum web framework
- **Container**: Multi-stage build with Rust 1.82 for compilation and Debian slim for runtime

### Frontend Service
- **Image**: Built from frontend/Dockerfile
- **Port**: 80
- **Technology**: Static files served by nginx
- **Container**: Nginx Alpine with runtime environment configuration

## Building Individual Services

### Build Backend Only
```bash
docker build -t chess-backend .
```

### Build Frontend Only
```bash
docker build -t chess-frontend ./frontend
```

## Running Individual Services

### Run Backend Only
```bash
docker run -p 3000:3000 chess-backend
```

### Run Frontend Only
```bash
docker run -p 80:80 -e API_URL=http://localhost:3000 chess-frontend
```

## Environment Variables

### Frontend
- `API_URL`: The backend API URL (default: `http://localhost:3000`)

Example:
```bash
docker run -p 80:80 -e API_URL=http://backend:3000 chess-frontend
```

## Docker Compose Commands

```bash
# Start services
docker compose up

# Start services in background
docker compose up -d

# Build and start services
docker compose up --build

# Stop services
docker compose down

# View logs
docker compose logs

# View logs for a specific service
docker compose logs backend
docker compose logs frontend

# Restart a service
docker compose restart backend

# Scale services (if needed)
docker compose up --scale backend=2
```

## Configuration

### Customizing Ports

Edit `docker-compose.yml` to change port mappings:

```yaml
services:
  backend:
    ports:
      - "8080:3000"  # Map host port 8080 to container port 3000
  
  frontend:
    ports:
      - "8000:80"    # Map host port 8000 to container port 80
```

### Customizing API URL

Update the environment variable in `docker-compose.yml`:

```yaml
services:
  frontend:
    environment:
      - API_URL=http://localhost:8080  # Point to custom backend URL
```

## Production Considerations

### 1. Use Pre-built Images

For production, build images in CI/CD and push to a registry:

```bash
# Build
docker build -t myregistry/chess-backend:v1.0 .
docker build -t myregistry/chess-frontend:v1.0 ./frontend

# Push
docker push myregistry/chess-backend:v1.0
docker push myregistry/chess-frontend:v1.0

# Update docker-compose.yml to use pre-built images
```

### 2. Use Docker Secrets

For sensitive data, use Docker secrets instead of environment variables:

```yaml
services:
  backend:
    secrets:
      - db_password
```

### 3. Add Health Checks

Add health checks to ensure services are running correctly:

```yaml
services:
  backend:
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/health"]
      interval: 30s
      timeout: 10s
      retries: 3
```

### 4. Resource Limits

Set resource limits to prevent containers from consuming too many resources:

```yaml
services:
  backend:
    deploy:
      resources:
        limits:
          cpus: '0.5'
          memory: 512M
```

### 5. Logging Configuration

Configure logging drivers for better log management:

```yaml
services:
  backend:
    logging:
      driver: "json-file"
      options:
        max-size: "10m"
        max-file: "3"
```

### 6. Use Production-Ready Database

The current implementation stores games in memory. For production:
- Add a PostgreSQL or Redis service to docker-compose.yml
- Implement persistence in the backend
- Mount volumes for data persistence

### 7. HTTPS/TLS

For production, add HTTPS support:
- Use a reverse proxy (nginx, Traefik, or Caddy)
- Configure SSL certificates (Let's Encrypt)
- Update frontend API_URL to use HTTPS

## Troubleshooting

### Container Won't Start
```bash
# Check container logs
docker compose logs backend
docker compose logs frontend

# Check container status
docker compose ps
```

### Network Issues
```bash
# Inspect the network
docker network inspect chess-rust_chess-network

# Restart network
docker compose down
docker compose up
```

### Build Cache Issues
```bash
# Build without cache
docker compose build --no-cache

# Remove all containers, networks, and volumes
docker compose down -v
```

### Port Already in Use
```bash
# Check what's using the port
sudo lsof -i :3000
sudo lsof -i :80

# Kill the process or change port in docker-compose.yml
```

## Development vs Production

### Development Setup
The default docker-compose.yml is configured for development with:
- Hot-reload disabled (would need volume mounts)
- Debug logging enabled
- Services bound to localhost

### Production Setup
For production, create a `docker-compose.prod.yml`:

```yaml
version: '3.8'

services:
  backend:
    image: myregistry/chess-backend:latest
    restart: always
    # ... production settings

  frontend:
    image: myregistry/chess-frontend:latest
    restart: always
    # ... production settings
```

Run with:
```bash
docker compose -f docker-compose.prod.yml up -d
```

## Monitoring

Consider adding monitoring tools:

```yaml
services:
  prometheus:
    image: prom/prometheus
    ports:
      - "9090:9090"
  
  grafana:
    image: grafana/grafana
    ports:
      - "3001:3000"
```

## Backup and Restore

### Backup
```bash
# Export game data (if persistence is added)
docker compose exec backend backup-command

# Backup volumes
docker run --rm -v chess-rust_data:/data -v $(pwd):/backup ubuntu tar czf /backup/data-backup.tar.gz /data
```

### Restore
```bash
# Restore volumes
docker run --rm -v chess-rust_data:/data -v $(pwd):/backup ubuntu tar xzf /backup/data-backup.tar.gz -C /
```

## CI/CD Integration

Example GitHub Actions workflow:

```yaml
name: Build and Push Docker Images

on:
  push:
    branches: [main]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      
      - name: Build images
        run: docker compose build
      
      - name: Push to registry
        run: |
          docker tag chess-backend myregistry/chess-backend:latest
          docker push myregistry/chess-backend:latest
```

## Support

This document is the primary guide for Docker usage. For more information, see:
- [Main README](README.md)
- [Frontend README](frontend/README.md)
- [Docker Deployment Reference](DOCKER-DEPLOYMENT.md) - detailed production deployment scenarios using Docker and Docker Compose
- [Docker Quick Reference](DOCKER-QUICKREF.md) - concise list of common Docker and Docker Compose commands for this project
