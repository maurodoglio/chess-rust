# Deployment Guide

This guide provides information on how to deploy and use the chess backend.

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

## Docker Deployment (Optional)

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
