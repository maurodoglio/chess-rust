// Chess Game Frontend Application

class ChessApp {
    constructor() {
        this.apiUrl = window.chessConfig?.apiUrl || 'http://localhost:3000';
        this.gameId = null;
        this.playerId = this.generatePlayerId();
        this.playerColor = null;
        this.selectedSquare = null;
        this.gameState = null;
        this.pollInterval = null;
        this.lastTurnColor = null;
        this.turnBannerShown = false;
        this.statusTimeout = null;
        
        this.initializeUI();
        this.attachEventListeners();
    }

    generatePlayerId() {
        // Generate a unique player ID or retrieve from localStorage
        let playerId = localStorage.getItem('chess_player_id');
        if (!playerId) {
            playerId = 'player_' + Math.random().toString(36).substring(2, 11);
            localStorage.setItem('chess_player_id', playerId);
        }
        return playerId;
    }

    initializeUI() {
        this.createBoard();
        this.updateApiUrl();
    }

    createBoard() {
        const board = document.getElementById('chessBoard');
        board.innerHTML = '';
        
        // Determine board orientation based on player color
        // White sees row 7 at top, Black sees row 0 at top
        const isBlack = this.playerColor && this.playerColor.toLowerCase() === 'black';
        const startRow = isBlack ? 0 : 7;
        const endRow = isBlack ? 7 : 0;
        const rowStep = isBlack ? 1 : -1;
        
        // Create 8x8 grid with appropriate orientation
        for (let row = startRow; isBlack ? row <= endRow : row >= endRow; row += rowStep) {
            for (let col = 0; col < 8; col++) {
                const square = document.createElement('div');
                square.classList.add('square');
                square.classList.add((row + col) % 2 === 0 ? 'dark' : 'light');
                square.dataset.row = row;
                square.dataset.col = col;
                
                square.addEventListener('click', () => this.handleSquareClick(row, col));
                
                board.appendChild(square);
            }
        }
    }

    attachEventListeners() {
        document.getElementById('newGameBtn').addEventListener('click', () => this.createNewGame());
        document.getElementById('joinGameBtn').addEventListener('click', () => this.showJoinGameModal());
        document.getElementById('listGamesBtn').addEventListener('click', () => this.listGames());
        document.getElementById('joinGameSubmit').addEventListener('click', () => this.joinGameFromModal());
        document.getElementById('updateApiBtn').addEventListener('click', () => this.updateApiUrl());
        
        // Modal close buttons
        document.querySelectorAll('.close').forEach(closeBtn => {
            closeBtn.addEventListener('click', (e) => {
                e.target.closest('.modal').style.display = 'none';
            });
        });
        
        // Close modal when clicking outside
        window.addEventListener('click', (e) => {
            if (e.target.classList.contains('modal')) {
                e.target.style.display = 'none';
            }
        });
    }

    updateApiUrl() {
        const input = document.getElementById('apiUrlInput');
        this.apiUrl = input.value || 'http://localhost:3000';
        document.getElementById('apiUrl').textContent = this.apiUrl;
        this.showStatus('API URL updated to: ' + this.apiUrl, 'info');
    }

    async createNewGame() {
        try {
            this.showStatus('Creating new game...', 'info');
            const response = await fetch(`${this.apiUrl}/games`, {
                method: 'POST'
            });
            
            if (!response.ok) throw new Error('Failed to create game');
            
            const data = await response.json();
            this.gameId = data.game_id;
            
            this.showStatus('Game created! ID: ' + this.gameId, 'success');
            
            // Automatically join the game
            await this.joinGame(this.gameId);
        } catch (error) {
            this.showStatus('Error creating game: ' + error.message, 'error');
        }
    }

    showJoinGameModal() {
        document.getElementById('joinGameModal').style.display = 'flex';
    }

    async joinGameFromModal() {
        const gameId = document.getElementById('gameIdInput').value.trim();
        if (!gameId) {
            this.showStatus('Please enter a game ID', 'error');
            return;
        }
        
        document.getElementById('joinGameModal').style.display = 'none';
        await this.joinGame(gameId);
    }

    async joinGame(gameId) {
        try {
            this.showStatus('Joining game...', 'info');
            const response = await fetch(`${this.apiUrl}/games/${gameId}/join`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({
                    player_id: this.playerId
                })
            });
            
            if (!response.ok) {
                const error = await response.json();
                throw new Error(error.error || 'Failed to join game');
            }
            
            const data = await response.json();
            this.gameId = gameId;
            this.playerColor = data.color;
            
            document.getElementById('playerInfo').style.display = 'block';
            document.getElementById('gameId').textContent = this.gameId;
            document.getElementById('playerColor').textContent = this.playerColor;
            
            this.showStatus(`Joined as ${this.playerColor}!`, 'success');
            
            // Recreate the board with correct orientation for this player
            this.createBoard();
            
            // Start polling for game updates
            this.startPolling();
            
            // Load initial game state
            await this.loadGameState();
        } catch (error) {
            this.showStatus('Error joining game: ' + error.message, 'error');
        }
    }

    async listGames() {
        try {
            this.showStatus('Loading games...', 'info');
            const response = await fetch(`${this.apiUrl}/games/list`);
            
            if (!response.ok) throw new Error('Failed to load games');
            
            const data = await response.json();
            this.displayGamesList(data.games);
        } catch (error) {
            this.showStatus('Error loading games: ' + error.message, 'error');
        }
    }

    displayGamesList(games) {
        const modal = document.getElementById('gameListModal');
        const gamesListDiv = document.getElementById('gamesList');
        
        if (games.length === 0) {
            gamesListDiv.innerHTML = '<p>No games available. Create a new game!</p>';
        } else {
            gamesListDiv.innerHTML = games.map(game => `
                <div class="game-item" data-game-id="${this.escapeHtml(game.id)}">
                    <p class="game-id">Game ID: ${this.escapeHtml(game.id)}</p>
                    <p class="game-status">${game.is_full ? '🔴 Full' : '🟢 Available'}</p>
                </div>
            `).join('');
            
            // Use event delegation for game item clicks
            gamesListDiv.querySelectorAll('.game-item').forEach(item => {
                item.addEventListener('click', () => {
                    const gameId = item.dataset.gameId;
                    this.joinGame(gameId);
                });
            });
        }
        
        modal.style.display = 'flex';
    }

    escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }

    startPolling() {
        if (this.pollInterval) {
            clearInterval(this.pollInterval);
        }
        
        // Poll every 2 seconds for game updates
        this.pollInterval = setInterval(() => {
            this.loadGameState();
        }, 2000);
    }

    stopPolling() {
        if (this.pollInterval) {
            clearInterval(this.pollInterval);
            this.pollInterval = null;
        }
    }

    async loadGameState() {
        if (!this.gameId) return;
        
        try {
            const response = await fetch(`${this.apiUrl}/games/${this.gameId}`);
            
            if (!response.ok) throw new Error('Failed to load game state');
            
            const data = await response.json();
            this.gameState = data;
            this.renderBoard();
            this.updateGameInfo();
        } catch (error) {
            console.error('Error loading game state:', error);
            // Don't show error for polling failures to avoid spam
        }
    }

    updateGameInfo() {
        if (!this.gameState) return;
        
        const currentTurnColor = this.gameState.game.current_turn === 'White' ? 'White' : 'Black';
        document.getElementById('currentTurn').textContent = currentTurnColor;
        
        const status = this.gameState.game.status;
        document.getElementById('gameStatus').textContent = status;
        
        // Update status message based on turn
        if (this.playerColor) {
            const isMyTurn = currentTurnColor.toLowerCase() === this.playerColor.toLowerCase();
            
            // Show the turn banner when turn changes to the player's turn
            if (isMyTurn && status === 'Active') {
                if (this.lastTurnColor !== currentTurnColor || !this.turnBannerShown) {
                    this.showStatus("It's your turn!", 'success', true); // persistent banner
                    this.turnBannerShown = true;
                }
            } else {
                // Clear the turn banner when it's not the player's turn
                if (this.turnBannerShown) {
                    this.clearStatus();
                    this.turnBannerShown = false;
                }
            }
            
            this.lastTurnColor = currentTurnColor;
        }
    }

    renderBoard() {
        if (!this.gameState) return;
        
        const board = this.gameState.game.board.squares;
        const squares = document.querySelectorAll('.square');
        
        squares.forEach(square => {
            const row = parseInt(square.dataset.row);
            const col = parseInt(square.dataset.col);
            
            const piece = board[row][col];
            square.textContent = piece ? this.getPieceSymbol(piece) : '';
            
            // Add piece class for styling
            if (piece) {
                square.classList.add('piece');
            } else {
                square.classList.remove('piece');
            }
        });
    }

    getPieceSymbol(piece) {
        const symbols = {
            'White': {
                'Pawn': '♙',
                'Knight': '♘',
                'Bishop': '♗',
                'Rook': '♖',
                'Queen': '♕',
                'King': '♔'
            },
            'Black': {
                'Pawn': '♟',
                'Knight': '♞',
                'Bishop': '♝',
                'Rook': '♜',
                'Queen': '♛',
                'King': '♚'
            }
        };
        
        return symbols[piece.color][piece.piece_type] || '';
    }

    handleSquareClick(row, col) {
        if (!this.gameId || !this.playerColor || !this.gameState) {
            this.showStatus('Join a game first!', 'error');
            return;
        }
        
        // Check if it's player's turn
        const currentTurnColor = this.gameState.game.current_turn === 'White' ? 'white' : 'black';
        if (currentTurnColor !== this.playerColor.toLowerCase()) {
            this.showStatus("It's not your turn!", 'error');
            return;
        }
        
        // If no square is selected, select this square if it has a piece of the player's color
        if (!this.selectedSquare) {
            const piece = this.gameState.game.board.squares[row][col];
            if (piece && piece.color.toLowerCase() === this.playerColor.toLowerCase()) {
                this.selectedSquare = { row, col };
                this.highlightSquare(row, col, true);
                this.showStatus('Select destination square', 'info');
            }
        } else {
            // If a square is already selected, try to make a move
            this.makeMove(this.selectedSquare.row, this.selectedSquare.col, row, col);
        }
    }

    highlightSquare(row, col, selected) {
        // Clear previous highlights
        document.querySelectorAll('.square').forEach(sq => {
            sq.classList.remove('selected', 'valid-move');
        });
        
        if (selected) {
            const square = document.querySelector(`[data-row="${row}"][data-col="${col}"]`);
            if (square) {
                square.classList.add('selected');
            }
        }
    }

    async makeMove(fromRow, fromCol, toRow, toCol) {
        try {
            this.showStatus('Making move...', 'info');
            
            const response = await fetch(`${this.apiUrl}/games/${this.gameId}/move`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({
                    player_id: this.playerId,
                    chess_move: {
                        from_row: fromRow,
                        from_col: fromCol,
                        to_row: toRow,
                        to_col: toCol
                    }
                })
            });
            
            if (!response.ok) {
                const error = await response.json();
                throw new Error(error.error || 'Invalid move');
            }
            
            const data = await response.json();
            this.gameState = data;
            this.renderBoard();
            this.updateGameInfo();
            
            this.selectedSquare = null;
            this.highlightSquare(null, null, false);
            
            this.turnBannerShown = false; // Reset turn banner flag after move
            this.showStatus('Move made successfully!', 'success');
        } catch (error) {
            this.showStatus('Error: ' + error.message, 'error');
            this.selectedSquare = null;
            this.highlightSquare(null, null, false);
        }
    }

    showStatus(message, type = 'info', persistent = false) {
        const statusDiv = document.getElementById('statusMessage');
        statusDiv.textContent = message;
        statusDiv.className = 'status-message ' + type;
        
        // Clear any existing timeout
        if (this.statusTimeout) {
            clearTimeout(this.statusTimeout);
            this.statusTimeout = null;
        }
        
        // Auto-hide success and info messages after 3 seconds, unless persistent
        if (type !== 'error' && !persistent) {
            this.statusTimeout = setTimeout(() => {
                statusDiv.textContent = '';
                statusDiv.className = 'status-message';
                this.statusTimeout = null;
            }, 3000);
        }
    }

    clearStatus() {
        const statusDiv = document.getElementById('statusMessage');
        statusDiv.textContent = '';
        statusDiv.className = 'status-message';
        
        // Clear any existing timeout
        if (this.statusTimeout) {
            clearTimeout(this.statusTimeout);
            this.statusTimeout = null;
        }
    }
}

// Initialize the app when DOM is loaded
let chessApp;
document.addEventListener('DOMContentLoaded', () => {
    chessApp = new ChessApp();
});
