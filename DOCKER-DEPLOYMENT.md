# Chess-Rust Docker Deployment

This document provides a complete guide for deploying the chess-rust application using Docker and Docker Compose.

## Prerequisites

- Docker 20.10+
- Docker Compose V2+

## Quick Start

Deploy both backend and frontend with a single command:

```bash
docker compose up --build
```

Access the application:
- **Frontend**: http://localhost
- **Backend API**: http://localhost:3000
- **Health Check**: http://localhost:3000/health

## What's Included

This Docker deployment includes:

### Backend Service
- **Base Image**: rust:1.82-bookworm (build), debian:bookworm-slim (runtime)
- **Port**: 3000
- **Features**:
  - Multi-stage build for optimized image size
  - Production-ready Rust binary
  - Health check endpoint at `/health`
  - CORS enabled for frontend communication

### Frontend Service
- **Base Image**: nginx:alpine
- **Port**: 80
- **Features**:
  - Optimized nginx configuration
  - Runtime environment variable injection
  - Configurable backend API URL
  - Static file serving with gzip compression

## Configuration

### Environment Variables

Create a `.env` file in the project root (see `.env.example`):

```env
BACKEND_PORT=3000
FRONTEND_PORT=80
API_URL=http://localhost:3000
```

### Custom API URL

To point the frontend to a different backend:

```bash
docker run -p 80:80 -e API_URL=http://your-backend:3000 chess-frontend
```

Or in docker-compose.yml:

```yaml
services:
  frontend:
    environment:
      - API_URL=http://your-backend:3000
```

## Commands

### Starting Services

```bash
# Start all services
docker compose up

# Start in background
docker compose up -d

# Build and start
docker compose up --build

# View logs
docker compose logs -f
```

### Stopping Services

```bash
# Stop all services
docker compose down

# Stop and remove volumes
docker compose down -v
```

### Building Individual Services

```bash
# Backend
docker build -t chess-backend .

# Frontend
docker build -t chess-frontend ./frontend
```

### Running Individual Services

```bash
# Backend only
docker run -p 3000:3000 chess-backend

# Frontend only
docker run -p 80:80 -e API_URL=http://localhost:3000 chess-frontend
```

## Testing the Deployment

Run the validation script to test the setup:

```bash
./validate-docker.sh
```

This script will:
- Check Docker and Docker Compose installation
- Verify all required files exist
- Build Docker images
- Test container functionality
- Verify environment variable injection

## File Structure

```
.
├── Dockerfile                     # Backend Dockerfile
├── .dockerignore                  # Backend build exclusions
├── docker-compose.yml             # Service orchestration
├── .env.example                   # Environment variable template
├── validate-docker.sh             # Validation script
│
├── frontend/
│   ├── Dockerfile                 # Frontend Dockerfile
│   ├── .dockerignore              # Frontend build exclusions
│   ├── nginx.conf                 # Nginx configuration
│   ├── docker-entrypoint.sh       # Environment injection script
│   ├── config.js                  # Runtime configuration
│   ├── index.html                 # Main HTML
│   ├── app.js                     # Application logic
│   └── styles.css                 # Styling
│
└── documentation/
    ├── DOCKER.md                  # Comprehensive Docker guide
    ├── DOCKER-QUICKREF.md         # Quick reference
    ├── README.md                  # Project README
    └── DEPLOYMENT.md              # General deployment guide
```

## Production Deployment

For production use, consider:

1. **Use Pre-built Images**
   - Build images in CI/CD
   - Push to a container registry
   - Pull images in production

2. **Add Health Checks**
   ```yaml
   healthcheck:
     test: ["CMD", "curl", "-f", "http://localhost:3000/health"]
     interval: 30s
     timeout: 10s
     retries: 3
   ```

3. **Set Resource Limits**
   ```yaml
   deploy:
     resources:
       limits:
         cpus: '0.5'
         memory: 512M
   ```

4. **Use HTTPS**
   - Add a reverse proxy (nginx, Traefik, Caddy)
   - Configure SSL certificates
   - Update API_URL to use HTTPS

5. **Add Persistence**
   - Integrate a database (PostgreSQL, Redis)
   - Mount volumes for data persistence
   - Implement backup strategies

See [DOCKER.md](DOCKER.md) for detailed production considerations.

## Troubleshooting

### Container Won't Start
```bash
docker compose logs backend
docker compose logs frontend
```

### Port Already in Use
```bash
# Check what's using the port
sudo lsof -i :3000
sudo lsof -i :80

# Or change ports in docker-compose.yml
```

### Build Issues
```bash
# Clean build without cache
docker compose build --no-cache

# Check Docker disk space
docker system df
docker system prune
```

### Network Issues
```bash
# Inspect network
docker network inspect chess-rust_chess-network

# Restart services
docker compose restart
```

## Additional Resources

- [DOCKER.md](DOCKER.md) - Comprehensive Docker deployment guide
- [DOCKER-QUICKREF.md](DOCKER-QUICKREF.md) - Quick command reference
- [README.md](README.md) - Project overview and API documentation
- [DEPLOYMENT.md](DEPLOYMENT.md) - General deployment information

## Support

For issues or questions:
1. Check the documentation files listed above
2. Run the validation script: `./validate-docker.sh`
3. Check container logs: `docker compose logs`
4. Review GitHub issues

## License

See LICENSE file for details.
