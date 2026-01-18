# chess-rust

A multiplayer web chess game backend implementation built in Rust. This server provides a REST API that enables players to create games, join games, and make moves from different devices.

## Features

- **Complete Chess Game Logic**: Full implementation of chess rules including piece movement validation
- **Check, Checkmate, and Stalemate Detection**: Automatically detects check, checkmate, and stalemate conditions
- **Move Validation**: Prevents illegal moves that would leave the king in check
- **Captured Pieces and Score Tracking**: Automatically tracks captured pieces and calculates running scores based on piece values (Pawn: 1, Knight/Bishop: 3, Rook: 5, Queen: 9)
- **Multiplayer Support**: Players can join games from different devices and play in real-time
- **Spectator Mode**: Watch games in progress without joining as a player
- **WebSocket Support**: Real-time game updates via WebSocket connections (no polling required)
- **REST API**: Simple HTTP endpoints for game management
- **Game State Management**: Maintains multiple simultaneous games with proper state tracking
- **CORS Enabled**: Ready for web frontend integration

## API Endpoints

### Health Check
```
GET /health
```
Returns: `OK`

### Create a New Game
```
POST /games
```
Returns:
```json
{
  "game_id": "uuid-here"
}
```

### List All Games
```
GET /games/list
```
Returns:
```json
{
  "games": [
    {
      "id": "game-uuid",
      "is_full": false
    }
  ]
}
```

### Get Game State
```
GET /games/:game_id
```
Returns the complete game state including:
- Board position with all pieces
- Current turn and game status
- Both players' information
- Move history
- **Captured pieces**: Lists all pieces captured by each player
- **Scores**: Running point totals based on captured piece values

### Spectate a Game
```
GET /games/:game_id/spectate
```
Watch a game in progress without joining as a player. Returns the complete game state including board, players, and move history. This endpoint is identical to the get game state endpoint but is semantically clearer for spectator functionality.

### Join a Game
```
POST /games/:game_id/join
Content-Type: application/json

{
  "player_id": "your-player-id"
}
```
Returns:
```json
{
  "color": "white"
}
```
The first player to join gets white, the second gets black.

### Make a Move
```
POST /games/:game_id/move
Content-Type: application/json

{
  "player_id": "your-player-id",
  "chess_move": {
    "from_row": 1,
    "from_col": 4,
    "to_row": 3,
    "to_col": 4
  }
}
```
Returns the updated game state.

### Resign from a Game
```
POST /games/:game_id/resign
Content-Type: application/json

{
  "player_id": "your-player-id"
}
```
Resigns the game on behalf of the player. The game status will be set to `Resigned`. Returns the updated game state.

### Offer a Draw
```
POST /games/:game_id/offer-draw
Content-Type: application/json

{
  "player_id": "your-player-id"
}
```
Offers a draw to the opponent. The game status will be set to `DrawOffered`. Returns the updated game state.

**Note**: You can only offer a draw on your turn.

### Accept a Draw
```
POST /games/:game_id/accept-draw
Content-Type: application/json

{
  "player_id": "your-player-id"
}
```
Accepts a draw offer from the opponent. The game status will be set to `Draw`. Returns the updated game state.

**Note**: You cannot accept your own draw offer.

### WebSocket Connection (Real-time Updates)
```
GET /games/:game_id/ws
Upgrade: websocket
```
Establishes a WebSocket connection for real-time game updates. Once connected, the client will:
- Immediately receive the current game state as JSON
- Automatically receive updates whenever the game state changes (moves, player joins, draws, resignations)

**WebSocket Message Format**: All messages sent from the server are JSON-encoded game states identical to the REST API responses.

**Example using JavaScript**:
```javascript
const ws = new WebSocket('ws://localhost:3000/games/GAME_ID/ws');

ws.onmessage = (event) => {
  const gameState = JSON.parse(event.data);
  console.log('Game updated:', gameState);
  // Update UI with new game state
};

ws.onerror = (error) => {
  console.error('WebSocket error:', error);
};
```

**Benefits**:
- Eliminates need for polling
- Instant updates when game state changes
- Lower server load and network traffic
- Better user experience with real-time updates

**Note**: The REST API endpoints remain fully functional. WebSocket support is optional and can be used alongside or instead of polling for game state updates.

**Coordinate System**: The board uses a 0-7 coordinate system where:
- Row 0 = White's back rank (a1-h1)
- Row 7 = Black's back rank (a8-h8)
- Col 0-7 = Files a-h

**Captured Pieces and Scoring**: The game automatically tracks:
- All pieces captured by each player in `captured_by_white` and `captured_by_black` arrays
- Running score totals in `white_score` and `black_score` based on standard chess piece values:
  - Pawn: 1 point
  - Knight: 3 points
  - Bishop: 3 points
  - Rook: 5 points
  - Queen: 9 points
  - King: 0 points (cannot be captured)

**Game Status**: The game automatically tracks and updates the status:
- `Active`: Normal play continues
- `Check`: Current player's king is in check
- `Checkmate`: Current player is checkmated (game over)
- `Stalemate`: Current player has no legal moves but is not in check (game over)
- `Draw`: Game has been declared a draw (by agreement)
- `Resigned`: A player has resigned (game over)
- `DrawOffered`: A draw has been offered and is awaiting response

The game prevents illegal moves that would leave the player's own king in check.

## Running the Server

### Option 1: Deploy to Azure (Recommended for Production)

Deploy to Microsoft Azure using Docker Compose with Azure Container Instances:

```bash
# Login to Azure
az login

# Create Azure context
docker context create aci azure-chess-rust

# Deploy with Docker Compose
docker compose -f docker-compose.azure.yml up
```

**📚 Documentation:**
- [Azure Deployment Guide](AZURE-DEPLOYMENT.md) - Complete Azure deployment guide with multiple deployment methods

### Option 2: Deploy to Render (Recommended for Quick Cloud Deployment)

Deploy both backend and frontend to the cloud with one click:

1. Fork this repository to your GitHub account
2. Sign up for a free [Render account](https://dashboard.render.com/register)
3. Create a new Blueprint and connect your forked repository
4. Render will automatically detect the `render.yaml` and deploy both services

**📚 Documentation:**
- [Quick Start Guide](RENDER-QUICKSTART.md) - Get started in 5 minutes
- [Detailed Guide](RENDER-DEPLOYMENT.md) - Complete deployment documentation

### Option 3: Using Docker (Recommended for Local Development)

The easiest way to run both backend and frontend together locally:

```bash
# Build and start both services
docker compose up --build

# Or run in detached mode
docker compose up -d --build
```

This will start:
- Backend API on `http://localhost:3000`
- Frontend UI on `http://localhost`

For detailed Docker deployment instructions, see [DOCKER.md](DOCKER.md)

### Option 4: Running Locally (for Development)

#### Prerequisites
- Rust 1.82 or later
- Cargo

#### Build and Run
```bash
# Build the project
cargo build

# Run the server
cargo run

# Run tests
cargo test
```

The server will start on `http://0.0.0.0:3000`

## Web Frontend

A complete web frontend is available in the `frontend/` directory. The frontend provides:

- Modern, responsive UI for playing chess
- Game creation and joining
- Real-time game state updates
- Visual feedback and move validation
- Easy-to-use interface with Unicode chess pieces

### Running the Frontend

#### With Docker (Recommended)
```bash
# Run both backend and frontend together
docker compose up
```
Access at `http://localhost`

#### Without Docker

1. Start the backend server:
   ```bash
   cargo run
   ```

2. Serve the frontend (choose one method):
   ```bash
   # Using Python
   cd frontend
   python3 -m http.server 8000
   
   # Using Node.js
   cd frontend
   npx http-server -p 8000
   ```

3. Open `http://localhost:8000` in your browser

For more details, see [frontend/README.md](frontend/README.md)

## Architecture

The project is organized into several modules:

- **chess**: Core chess game logic
  - `piece.rs`: Piece types and colors
  - `board.rs`: Board representation and manipulation
  - `game.rs`: Game rules and move validation
- **game**: Multiplayer session management
  - `session.rs`: Game session and player management
  - `state.rs`: Shared state for multiple games with WebSocket broadcasting
- **api**: REST API handlers
- **ws**: WebSocket connection handling for real-time updates

## Example Usage

```bash
# Create a new game
GAME_ID=$(curl -X POST http://localhost:3000/games | jq -r '.game_id')

# Player 1 joins
curl -X POST http://localhost:3000/games/$GAME_ID/join \
  -H "Content-Type: application/json" \
  -d '{"player_id": "player1"}'

# Player 2 joins
curl -X POST http://localhost:3000/games/$GAME_ID/join \
  -H "Content-Type: application/json" \
  -d '{"player_id": "player2"}'

# Player 1 makes a move (e2 to e4)
curl -X POST http://localhost:3000/games/$GAME_ID/move \
  -H "Content-Type: application/json" \
  -d '{
    "player_id": "player1",
    "chess_move": {
      "from_row": 1,
      "from_col": 4,
      "to_row": 3,
      "to_col": 4
    }
  }'

# Spectate the game (anyone can watch without joining)
curl http://localhost:3000/games/$GAME_ID/spectate
```

## Future Enhancements

Potential improvements for this backend:

- Game persistence (database integration)
- Authentication and user accounts
- Move history and game replay
- En passant and castling support
- Game timers and time controls
- Game ratings and statistics

## License

See LICENSE file for details.
