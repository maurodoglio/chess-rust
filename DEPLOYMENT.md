# Deployment Guide

This guide provides information on how to deploy and use the chess backend.

## Deployment Options

### Azure Deployment (Production)

For production deployments to Microsoft Azure, see the [Azure Deployment Guide](AZURE-DEPLOYMENT.md).

Quick start:
```bash
az login
docker context create aci azure-chess-rust
docker compose -f docker-compose.azure.yml up
```

For detailed instructions, see [AZURE-DEPLOYMENT.md](AZURE-DEPLOYMENT.md) and [AZURE-QUICKREF.md](AZURE-QUICKREF.md).

### Render Deployment (Quick Cloud Setup)

For quick cloud deployment with minimal setup, see the [Render Deployment Guide](RENDER-DEPLOYMENT.md).

### Docker Deployment (Local/Self-Hosted)

The easiest way to deploy the application locally or on your own servers is using Docker and Docker Compose.

### Quick Start

```bash
# Build and start all services
docker compose up --build

# Or run in background
docker compose up -d --build
```

This will start:
- Backend API on port 3000
- Frontend web UI on port 80

Access the game at `http://localhost`

### Docker Commands

```bash
# View logs
docker compose logs -f

# Stop services
docker compose down

# Rebuild after changes
docker compose up --build

# View running containers
docker compose ps
```

For detailed Docker deployment information including production setup, monitoring, and troubleshooting, see [DOCKER.md](DOCKER.md).

## Local Development

### Running the Server
```bash
cargo run
```

The server will start on `http://0.0.0.0:3000`

### Running Tests
```bash
cargo test
```

### Building for Release
```bash
cargo build --release
```

The optimized binary will be available at `target/release/chess-rust`

## Docker Deployment (Detailed)

### Architecture

The Docker setup uses two containers:
1. **Backend**: Rust application in a Debian slim container
2. **Frontend**: Static files served by nginx Alpine

### Building Docker Images

```bash
# Build backend
docker build -t chess-backend .

# Build frontend
docker build -t chess-frontend ./frontend
```

### Running with Docker Compose

The `docker-compose.yml` file defines the complete stack:

```bash
# Start all services
docker compose up

# Start in background
docker compose up -d

# View logs
docker compose logs -f backend
docker compose logs -f frontend

# Stop all services
docker compose down
```

### Environment Configuration

The frontend can be configured with the backend API URL:

```yaml
services:
  frontend:
    environment:
      - API_URL=http://localhost:3000
```

### Container Management

```bash
# Restart a service
docker compose restart backend

# Scale backend (if needed)
docker compose up --scale backend=2

# Remove all containers and volumes
docker compose down -v
```

## Alternative: Manual Docker Build

You can create a simple Dockerfile:

```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/chess-rust /usr/local/bin/chess-rust
EXPOSE 3000
CMD ["chess-rust"]
```

Build and run:
```bash
docker build -t chess-backend .
docker run -p 3000:3000 chess-backend
```

## Production Considerations

1. **Environment Variables**: Currently the server binds to `0.0.0.0:3000`. Consider making this configurable via environment variables:
   - `CHESS_HOST` (default: 0.0.0.0)
   - `CHESS_PORT` (default: 3000)

2. **Database**: The current implementation stores games in memory. For production:
   - Add database support (PostgreSQL, Redis, etc.)
   - Implement game persistence
   - Add game cleanup/expiration

3. **Authentication**: Add player authentication:
   - JWT tokens
   - OAuth integration
   - Session management

4. **WebSockets**: For real-time updates, consider adding WebSocket support

5. **Rate Limiting**: Add rate limiting to prevent abuse

6. **Monitoring**: Add metrics and monitoring:
   - Prometheus metrics
   - Health check endpoints
   - Logging aggregation

## API Client Examples

### JavaScript/TypeScript
```javascript
// Create a game
const response = await fetch('http://localhost:3000/games', {
  method: 'POST'
});
const { game_id } = await response.json();

// Join the game
await fetch(`http://localhost:3000/games/${game_id}/join`, {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ player_id: 'player1' })
});

// Make a move
await fetch(`http://localhost:3000/games/${game_id}/move`, {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    player_id: 'player1',
    chess_move: {
      from_row: 1,
      from_col: 4,
      to_row: 3,
      to_col: 4
    }
  })
});
```

### Python
```python
import requests

# Create a game
response = requests.post('http://localhost:3000/games')
game_id = response.json()['game_id']

# Join the game
requests.post(
    f'http://localhost:3000/games/{game_id}/join',
    json={'player_id': 'player1'}
)

# Make a move
requests.post(
    f'http://localhost:3000/games/{game_id}/move',
    json={
        'player_id': 'player1',
        'chess_move': {
            'from_row': 1,
            'from_col': 4,
            'to_row': 3,
            'to_col': 4
        }
    }
)
```

## Frontend Integration

To build a web frontend:

1. Create a chess board UI using HTML5 Canvas or SVG
2. Use the REST API to:
   - Create/join games
   - Poll for game state updates
   - Send moves to the server
3. Consider using a framework like React, Vue, or Angular
4. Display the 8x8 board with pieces
5. Handle click events to select and move pieces
6. Show current turn and game status

### Example Board Mapping
The API uses 0-indexed coordinates:
- Row 0 = Rank 1 (White's back rank)
- Row 7 = Rank 8 (Black's back rank)
- Col 0-7 = Files a-h

## Security Notes

- The current implementation has no authentication
- All games are public and anyone can join
- Consider adding:
  - Player authentication
  - Game passwords
  - Player verification for moves
  - Rate limiting on API endpoints
