# Chess Frontend

A modern, responsive web frontend for the chess-rust multiplayer chess game.

## Features

- **Beautiful UI**: Clean, modern interface with responsive design
- **Real-time Updates**: Automatic polling for game state changes
- **Easy Game Management**: Create, join, and list games with simple controls
- **Visual Feedback**: Color-coded squares, piece selection, and status messages
- **Unicode Pieces**: Uses Unicode chess symbols for a clean look
- **Configurable API**: Easy to change backend API URL

## Quick Start

### Option 1: Serve with Python HTTP Server

```bash
cd frontend
python3 -m http.server 8000
```

Then open `http://localhost:8000` in your browser.

### Option 2: Serve with Node.js

```bash
cd frontend
npx http-server -p 8000
```

Then open `http://localhost:8000` in your browser.

### Option 3: Open Directly in Browser

Simply open the `index.html` file directly in your web browser. Note that some browsers may have CORS restrictions when opening files directly.

## Usage

### Starting a New Game

1. Make sure the Rust backend is running on `http://localhost:3000`
2. Click the "New Game" button
3. The game will be created and you'll automatically join as the first player (White)
4. Share the Game ID with another player to join

### Joining an Existing Game

1. Click "Join Game" button
2. Enter the Game ID provided by the game creator
3. Click "Join" to join as the second player (Black)

### Listing Available Games

1. Click "List Games" to see all available games
2. Click on any game to join it

### Making Moves

1. Wait for your turn (indicated by the "Current Turn" display)
2. Click on one of your pieces to select it
3. Click on the destination square to move the piece
4. The board will update automatically after a valid move

### Coordinate System

The board uses the same coordinate system as the backend:
- Row 0 = White's back rank (a1-h1)
- Row 7 = Black's back rank (a8-h8)
- Col 0-7 = Files a-h

The display shows the board from White's perspective (White at bottom).

## Configuration

### Changing the API URL

1. Enter the new API URL in the input field at the bottom of the page
2. Click "Update" to apply the changes
3. The new URL will be used for all future API calls

Default: `http://localhost:3000`

## Architecture

The frontend is built with vanilla JavaScript, HTML, and CSS:

- **index.html**: Main HTML structure
- **styles.css**: Responsive CSS styling with modern design
- **app.js**: Game logic and API integration

### Key Components

- **ChessApp Class**: Main application class handling game logic
- **Board Rendering**: Dynamic 8x8 grid with Unicode piece symbols
- **API Integration**: Fetch-based communication with the backend
- **State Management**: Local game state with automatic polling
- **Player Management**: Persistent player ID stored in localStorage

## Browser Compatibility

The frontend works on all modern browsers:
- Chrome/Edge (recommended)
- Firefox
- Safari
- Opera

## Responsive Design

The interface adapts to different screen sizes:
- Desktop: Full-sized board (70px squares)
- Tablet: Medium board (45px squares)
- Mobile: Compact board (35px squares)

## Development

### File Structure

```
frontend/
├── index.html    # Main HTML file
├── styles.css    # Styling
├── app.js        # Application logic
└── README.md     # This file
```

### Customization

You can customize the appearance by editing `styles.css`:
- Colors: Modify the gradient and color variables
- Board size: Adjust the grid dimensions in `.chess-board`
- Piece symbols: Change Unicode characters in `getPieceSymbol()`

## Troubleshooting

### Cannot connect to backend

- Ensure the Rust backend is running: `cargo run` in the project root
- Check the API URL is correct (default: `http://localhost:3000`)
- Verify no firewall is blocking the connection

### CORS errors

- Make sure the backend has CORS enabled (it should be by default)
- If opening the file directly, try serving it with a local HTTP server

### Pieces not displaying

- Ensure your browser supports Unicode chess symbols
- Try using a different browser or updating your current browser

### Game state not updating

- The frontend polls every 2 seconds for updates
- Check the browser console for any network errors
- Verify the backend is responding to requests

## Future Enhancements

Possible improvements:
- WebSocket support for real-time updates
- Move validation preview (show valid moves)
- Move history display
- Captured pieces display
- Game timer/clock
- Sound effects
- Animations for piece movements
- Drag-and-drop piece movement
- Chess notation display (algebraic notation)
- Game analysis and replay

## License

See the main project LICENSE file for details.
