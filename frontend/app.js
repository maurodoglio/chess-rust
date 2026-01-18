// Chess Game Frontend Application

class ChessApp {
    constructor() {
        this.apiUrl = window.chessConfig?.apiUrl || 'http://localhost:3000';
        this.gameId = null;
        this.playerId = this.generatePlayerId();
        this.playerColor = null;
        this.isSpectator = false;
        this.selectedSquare = null;
        this.gameState = null;
        this.pollInterval = null;
        this.lastTurnColor = null;
        this.turnBannerShown = false;
        this.statusTimeout = null;
        this.authToken = null;
        this.username = null;
        this.isLoginMode = true;
        
        this.loadAuthState();
        this.initializeUI();
        this.attachEventListeners();
        this.updateAuthUI();
        
        // Restore game session if one exists
        if (this.loadGameSession()) {
            this.restoreGameSession();
        }
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

    loadAuthState() {
        this.authToken = localStorage.getItem('chess_auth_token');
        this.username = localStorage.getItem('chess_username');
    }

    saveAuthState(token, username) {
        this.authToken = token;
        this.username = username;
        localStorage.setItem('chess_auth_token', token);
        localStorage.setItem('chess_username', username);
    }

    clearAuthState() {
        this.authToken = null;
        this.username = null;
        localStorage.removeItem('chess_auth_token');
        localStorage.removeItem('chess_username');
        // Clear game session when auth is cleared to prevent orphaned sessions
        this.clearGameSession();
    }

    loadGameSession() {
        const gameId = localStorage.getItem('chess_game_id');
        const playerColorStr = localStorage.getItem('chess_player_color');
        const isSpectator = localStorage.getItem('chess_is_spectator') === 'true';
        
        if (gameId && this.isAuthenticated()) {
            this.gameId = gameId;
            // Convert string 'null' or empty string to actual null
            this.playerColor = (playerColorStr && playerColorStr !== 'null') ? playerColorStr : null;
            this.isSpectator = isSpectator;
            return true;
        }
        return false;
    }

    saveGameSession(gameId, playerColor, isSpectator) {
        this.gameId = gameId;
        this.playerColor = playerColor;
        this.isSpectator = isSpectator;
        localStorage.setItem('chess_game_id', gameId);
        // Store null as the string 'null' for consistency
        localStorage.setItem('chess_player_color', playerColor === null ? 'null' : playerColor);
        localStorage.setItem('chess_is_spectator', isSpectator.toString());
    }

    clearGameSession() {
        this.gameId = null;
        this.playerColor = null;
        this.isSpectator = false;
        localStorage.removeItem('chess_game_id');
        localStorage.removeItem('chess_player_color');
        localStorage.removeItem('chess_is_spectator');
    }

    async restoreGameSession() {
        if (!this.gameId) {
            console.log('No game session to restore');
            return;
        }
        
        try {
            this.showStatus('Restoring game session...', 'info');
            
            // Update UI to show game info
            document.getElementById('playerInfo').style.display = 'block';
            document.getElementById('gameId').textContent = this.gameId;
            
            if (this.isSpectator) {
                document.getElementById('playerColorInfo').style.display = 'none';
                document.getElementById('spectatorInfo').style.display = 'block';
            } else if (this.playerColor) {
                document.getElementById('playerColor').textContent = this.playerColor;
                document.getElementById('playerColorInfo').style.display = 'block';
                document.getElementById('spectatorInfo').style.display = 'none';
            }
            
            // Recreate the board with correct orientation
            this.createBoard();
            
            // Load game state and start polling
            await this.loadGameState();
            this.startPolling();
            
            this.showStatus('Game session restored!', 'success');
        } catch (error) {
            this.showStatus('Could not restore game session: ' + error.message, 'error');
            // Clear invalid session
            this.clearGameSession();
        }
    }

    isAuthenticated() {
        return this.authToken !== null && this.username !== null;
    }

    getAuthHeaders() {
        if (this.isAuthenticated()) {
            return {
                'Authorization': `Bearer ${this.authToken}`,
                'Content-Type': 'application/json'
            };
        }
        return {
            'Content-Type': 'application/json'
        };
    }

    updateAuthUI() {
        const authStatus = document.getElementById('authStatus');
        if (this.isAuthenticated()) {
            authStatus.innerHTML = `
                <span class="username">👤 ${this.username}</span>
                <button id="logoutBtn">Logout</button>
            `;
            document.getElementById('logoutBtn').addEventListener('click', () => this.logout());
        } else {
            authStatus.innerHTML = `
                <button id="loginBtn">Login / Register</button>
            `;
            document.getElementById('loginBtn').addEventListener('click', () => this.showAuthModal());
        }
    }

    showAuthModal() {
        this.isLoginMode = true;
        this.updateAuthModalUI();
        document.getElementById('authModal').style.display = 'flex';
        document.getElementById('authError').style.display = 'none';
        document.getElementById('authUsername').value = '';
        document.getElementById('authPassword').value = '';
    }

    updateAuthModalUI() {
        const title = document.getElementById('authModalTitle');
        const submitBtn = document.getElementById('authSubmitBtn');
        const toggleBtn = document.getElementById('authToggleBtn');
        
        if (this.isLoginMode) {
            title.textContent = 'Login';
            submitBtn.textContent = 'Login';
            toggleBtn.textContent = 'Register Instead';
        } else {
            title.textContent = 'Register';
            submitBtn.textContent = 'Register';
            toggleBtn.textContent = 'Login Instead';
        }
    }

    async handleAuth() {
        const username = document.getElementById('authUsername').value.trim();
        const password = document.getElementById('authPassword').value;
        const errorDiv = document.getElementById('authError');

        // Validate input
        if (username.length < 3) {
            errorDiv.textContent = 'Username must be at least 3 characters long';
            errorDiv.style.display = 'block';
            return;
        }

        if (password.length < 6) {
            errorDiv.textContent = 'Password must be at least 6 characters long';
            errorDiv.style.display = 'block';
            return;
        }

        const endpoint = this.isLoginMode ? '/auth/login' : '/auth/register';
        
        try {
            const response = await fetch(`${this.apiUrl}${endpoint}`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({ username, password })
            });

            const data = await response.json();

            if (!response.ok) {
                errorDiv.textContent = data.error || 'Authentication failed';
                errorDiv.style.display = 'block';
                return;
            }

            // Save authentication state
            this.saveAuthState(data.token, data.username);
            
            // Close modal and update UI
            document.getElementById('authModal').style.display = 'none';
            this.updateAuthUI();
            
            const action = this.isLoginMode ? 'logged in' : 'registered';
            this.showStatus(`Successfully ${action} as ${data.username}!`, 'success');
        } catch (error) {
            errorDiv.textContent = 'Network error: ' + error.message;
            errorDiv.style.display = 'block';
        }
    }

    toggleAuthMode() {
        this.isLoginMode = !this.isLoginMode;
        this.updateAuthModalUI();
        document.getElementById('authError').style.display = 'none';
    }

    logout() {
        this.clearAuthState();
        this.clearGameSession();
        this.updateAuthUI();
        this.showStatus('Logged out successfully', 'info');
        
        // Clear game UI
        document.getElementById('playerInfo').style.display = 'none';
        if (this.pollInterval) {
            clearInterval(this.pollInterval);
            this.pollInterval = null;
        }
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
        document.getElementById('watchGameBtn').addEventListener('click', () => this.showWatchGameModal());
        document.getElementById('listGamesBtn').addEventListener('click', () => this.listGames());
        document.getElementById('joinGameSubmit').addEventListener('click', () => this.joinGameFromModal());
        document.getElementById('watchGameSubmit').addEventListener('click', () => this.watchGameFromModal());
        document.getElementById('updateApiBtn').addEventListener('click', () => this.updateApiUrl());
        
        // Auth modal listeners
        document.getElementById('authSubmitBtn').addEventListener('click', () => this.handleAuth());
        document.getElementById('authToggleBtn').addEventListener('click', () => this.toggleAuthMode());
        
        // Handle Enter key in auth form
        const authInputs = [document.getElementById('authUsername'), document.getElementById('authPassword')];
        authInputs.forEach(input => {
            input.addEventListener('keypress', (e) => {
                if (e.key === 'Enter') {
                    this.handleAuth();
                }
            });
        });
        
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
        if (!this.isAuthenticated()) {
            this.showStatus('Please login to create a game', 'error');
            this.showAuthModal();
            return;
        }
        
        try {
            this.showStatus('Creating new game...', 'info');
            const response = await fetch(`${this.apiUrl}/games`, {
                method: 'POST',
                headers: this.getAuthHeaders()
            });
            
            if (!response.ok) {
                if (response.status === 401) {
                    this.showStatus('Session expired. Please login again.', 'error');
                    this.clearAuthState();
                    this.updateAuthUI();
                    return;
                }
                throw new Error('Failed to create game');
            }
            
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

    showWatchGameModal() {
        document.getElementById('watchGameModal').style.display = 'flex';
    }

    async watchGameFromModal() {
        const gameId = document.getElementById('watchGameIdInput').value.trim();
        if (!gameId) {
            this.showStatus('Please enter a game ID', 'error');
            return;
        }
        
        document.getElementById('watchGameModal').style.display = 'none';
        await this.watchGame(gameId);
    }

    async watchGame(gameId) {
        if (!this.isAuthenticated()) {
            this.showStatus('Please login to watch games', 'error');
            this.showAuthModal();
            return;
        }
        
        try {
            this.showStatus('Loading game as spectator...', 'info');
            const response = await fetch(`${this.apiUrl}/games/${gameId}/spectate`, {
                headers: this.getAuthHeaders()
            });
            
            if (!response.ok) {
                if (response.status === 401) {
                    this.showStatus('Session expired. Please login again.', 'error');
                    this.clearAuthState();
                    this.updateAuthUI();
                    return;
                }
                const error = await response.json();
                throw new Error(error.error || 'Failed to load game');
            }
            
            const data = await response.json();
            
            // Save game session to localStorage
            this.saveGameSession(gameId, null, true);
            
            document.getElementById('playerInfo').style.display = 'block';
            document.getElementById('gameId').textContent = this.gameId;
            document.getElementById('playerColorInfo').style.display = 'none';
            document.getElementById('spectatorInfo').style.display = 'block';
            
            this.showStatus('Watching game as spectator!', 'success');
            
            // Create the board with default white orientation
            this.createBoard();
            
            // Start polling for game updates
            this.startPolling();
            
            // Load initial game state
            this.gameState = data;
            this.renderBoard();
            this.updateGameInfo();
        } catch (error) {
            this.showStatus('Error watching game: ' + error.message, 'error');
        }
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
        if (!this.isAuthenticated()) {
            this.showStatus('Please login to join games', 'error');
            this.showAuthModal();
            return;
        }
        
        try {
            this.showStatus('Joining game...', 'info');
            const response = await fetch(`${this.apiUrl}/games/${gameId}/join`, {
                method: 'POST',
                headers: this.getAuthHeaders(),
                body: JSON.stringify({
                    player_id: this.playerId
                })
            });
            
            if (!response.ok) {
                if (response.status === 401) {
                    this.showStatus('Session expired. Please login again.', 'error');
                    this.clearAuthState();
                    this.updateAuthUI();
                    return;
                }
                const error = await response.json();
                throw new Error(error.error || 'Failed to join game');
            }
            
            const data = await response.json();
            
            // Save game session to localStorage
            this.saveGameSession(gameId, data.color, false);
            
            document.getElementById('playerInfo').style.display = 'block';
            document.getElementById('gameId').textContent = this.gameId;
            document.getElementById('playerColor').textContent = this.playerColor;
            document.getElementById('playerColorInfo').style.display = 'block';
            document.getElementById('spectatorInfo').style.display = 'none';
            
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
        if (!this.isAuthenticated()) {
            this.showStatus('Please login to view games', 'error');
            this.showAuthModal();
            return;
        }
        
        try {
            this.showStatus('Loading games...', 'info');
            const response = await fetch(`${this.apiUrl}/games/list`, {
                headers: this.getAuthHeaders()
            });
            
            if (!response.ok) {
                if (response.status === 401) {
                    this.showStatus('Session expired. Please login again.', 'error');
                    this.clearAuthState();
                    this.updateAuthUI();
                    return;
                }
                throw new Error('Failed to load games');
            }
            
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
            const response = await fetch(`${this.apiUrl}/games/${this.gameId}`, {
                headers: this.getAuthHeaders()
            });
            
            if (!response.ok) {
                if (response.status === 401) {
                    this.showStatus('Session expired. Please login again.', 'error');
                    this.clearAuthState();
                    this.updateAuthUI();
                    this.stopPolling();
                    return;
                }
                throw new Error('Failed to load game state');
            }
            
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
        if (!this.gameId || !this.gameState) {
            this.showStatus('Join or watch a game first!', 'error');
            return;
        }
        
        // Prevent moves if spectator
        if (this.isSpectator) {
            this.showStatus('You are a spectator and cannot make moves', 'error');
            return;
        }
        
        if (!this.playerColor) {
            this.showStatus('Join a game first to make moves!', 'error');
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
                this.highlightSquareAndMoves(row, col);
                this.showStatus('Select destination square', 'info');
            }
        } else {
            // If clicking the same square, deselect it
            if (this.selectedSquare.row === row && this.selectedSquare.col === col) {
                this.selectedSquare = null;
                this.clearHighlights();
                this.showStatus('Piece deselected', 'info');
            } else {
                // If a square is already selected, try to make a move
                this.makeMove(this.selectedSquare.row, this.selectedSquare.col, row, col);
            }
        }
    }

    clearHighlights() {
        document.querySelectorAll('.square').forEach(sq => {
            sq.classList.remove('selected', 'valid-move');
        });
    }

    highlightSquareAndMoves(row, col) {
        // Clear previous highlights
        this.clearHighlights();
        
        // Highlight the selected square
        const selectedSquare = document.querySelector(`[data-row="${row}"][data-col="${col}"]`);
        if (selectedSquare) {
            selectedSquare.classList.add('selected');
        }
        
        // Calculate and highlight valid moves
        const validMoves = this.calculateValidMoves(row, col);
        validMoves.forEach(move => {
            const square = document.querySelector(`[data-row="${move.row}"][data-col="${move.col}"]`);
            if (square) {
                square.classList.add('valid-move');
            }
        });
    }

    calculateValidMoves(fromRow, fromCol) {
        const validMoves = [];
        const piece = this.gameState.game.board.squares[fromRow][fromCol];
        
        if (!piece) return validMoves;
        
        const pieceType = piece.piece_type;
        const color = piece.color;
        
        // Helper function to check if a square is on the board
        const isValidSquare = (row, col) => row >= 0 && row < 8 && col >= 0 && col < 8;
        
        // Helper function to check if a square is empty or has an opponent piece
        const canMoveTo = (row, col) => {
            if (!isValidSquare(row, col)) return false;
            const targetPiece = this.gameState.game.board.squares[row][col];
            return !targetPiece || targetPiece.color !== color;
        };
        
        // Helper function to add moves in a direction (for sliding pieces)
        const addSlidingMoves = (rowDir, colDir) => {
            let row = fromRow + rowDir;
            let col = fromCol + colDir;
            while (isValidSquare(row, col)) {
                const targetPiece = this.gameState.game.board.squares[row][col];
                if (!targetPiece) {
                    validMoves.push({ row, col });
                } else {
                    if (targetPiece.color !== color) {
                        validMoves.push({ row, col });
                    }
                    break;
                }
                row += rowDir;
                col += colDir;
            }
        };
        
        // Calculate moves based on piece type
        switch (pieceType) {
            case 'Pawn':
                const direction = color === 'White' ? 1 : -1;
                const startRow = color === 'White' ? 1 : 6;
                
                // Forward move
                if (isValidSquare(fromRow + direction, fromCol) && 
                    !this.gameState.game.board.squares[fromRow + direction][fromCol]) {
                    validMoves.push({ row: fromRow + direction, col: fromCol });
                    
                    // Double move from start
                    if (fromRow === startRow && 
                        !this.gameState.game.board.squares[fromRow + 2 * direction][fromCol]) {
                        validMoves.push({ row: fromRow + 2 * direction, col: fromCol });
                    }
                }
                
                // Diagonal captures
                [-1, 1].forEach(colOffset => {
                    const targetRow = fromRow + direction;
                    const targetCol = fromCol + colOffset;
                    if (isValidSquare(targetRow, targetCol)) {
                        const targetPiece = this.gameState.game.board.squares[targetRow][targetCol];
                        if (targetPiece && targetPiece.color !== color) {
                            validMoves.push({ row: targetRow, col: targetCol });
                        }
                    }
                });
                break;
                
            case 'Knight':
                const knightMoves = [
                    [-2, -1], [-2, 1], [-1, -2], [-1, 2],
                    [1, -2], [1, 2], [2, -1], [2, 1]
                ];
                knightMoves.forEach(([rowOffset, colOffset]) => {
                    const targetRow = fromRow + rowOffset;
                    const targetCol = fromCol + colOffset;
                    if (canMoveTo(targetRow, targetCol)) {
                        validMoves.push({ row: targetRow, col: targetCol });
                    }
                });
                break;
                
            case 'Bishop':
                addSlidingMoves(1, 1);
                addSlidingMoves(1, -1);
                addSlidingMoves(-1, 1);
                addSlidingMoves(-1, -1);
                break;
                
            case 'Rook':
                addSlidingMoves(1, 0);
                addSlidingMoves(-1, 0);
                addSlidingMoves(0, 1);
                addSlidingMoves(0, -1);
                break;
                
            case 'Queen':
                addSlidingMoves(1, 1);
                addSlidingMoves(1, -1);
                addSlidingMoves(-1, 1);
                addSlidingMoves(-1, -1);
                addSlidingMoves(1, 0);
                addSlidingMoves(-1, 0);
                addSlidingMoves(0, 1);
                addSlidingMoves(0, -1);
                break;
                
            case 'King':
                const kingMoves = [
                    [-1, -1], [-1, 0], [-1, 1],
                    [0, -1], [0, 1],
                    [1, -1], [1, 0], [1, 1]
                ];
                kingMoves.forEach(([rowOffset, colOffset]) => {
                    const targetRow = fromRow + rowOffset;
                    const targetCol = fromCol + colOffset;
                    if (canMoveTo(targetRow, targetCol)) {
                        validMoves.push({ row: targetRow, col: targetCol });
                    }
                });
                break;
        }
        
        return validMoves;
    }

    async makeMove(fromRow, fromCol, toRow, toCol) {
        try {
            this.showStatus('Making move...', 'info');
            
            const response = await fetch(`${this.apiUrl}/games/${this.gameId}/move`, {
                method: 'POST',
                headers: this.getAuthHeaders(),
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
                if (response.status === 401) {
                    this.showStatus('Session expired. Please login again.', 'error');
                    this.clearAuthState();
                    this.updateAuthUI();
                    this.stopPolling();
                    return;
                }
                const error = await response.json();
                throw new Error(error.error || 'Invalid move');
            }
            
            const data = await response.json();
            this.gameState = data;
            this.renderBoard();
            this.updateGameInfo();
            
            this.selectedSquare = null;
            this.clearHighlights();
            
            this.turnBannerShown = false; // Reset turn banner flag after move
            this.showStatus('Move made successfully!', 'success');
        } catch (error) {
            this.showStatus('Error: ' + error.message, 'error');
            this.selectedSquare = null;
            this.clearHighlights();
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
