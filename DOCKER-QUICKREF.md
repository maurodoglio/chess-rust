# Docker Quick Reference

Quick commands for working with the chess-rust Docker deployment.

## Starting the Application

```bash
# Start all services
docker compose up

# Start in background
docker compose up -d

# Build and start
docker compose up --build
```

## Stopping the Application

```bash
# Stop all services
docker compose down

# Stop and remove volumes
docker compose down -v
```

## Viewing Logs

```bash
# All services
docker compose logs -f

# Specific service
docker compose logs -f backend
docker compose logs -f frontend
```

## Individual Services

```bash
# Build backend
docker build -t chess-backend .

# Build frontend
docker build -t chess-frontend ./frontend

# Run backend
docker run -p 3000:3000 chess-backend

# Run frontend
docker run -p 80:80 -e API_URL=http://localhost:3000 chess-frontend
```

## Accessing the Application

- Frontend: http://localhost
- Backend API: http://localhost:3000
- Health Check: http://localhost:3000/health

## Troubleshooting

```bash
# View running containers
docker compose ps

# Restart a service
docker compose restart backend

# View container logs
docker logs chess-backend
docker logs chess-frontend

# Shell into a container
docker exec -it chess-backend /bin/bash
docker exec -it chess-frontend /bin/sh

# Remove all containers and start fresh
docker compose down -v
docker compose up --build
```

## Customization

Edit `docker-compose.yml` to:
- Change ports
- Modify environment variables
- Add volumes for persistence
- Configure resource limits

See [DOCKER.md](DOCKER.md) for detailed documentation.
