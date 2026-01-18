use super::board::Board;
use super::piece::{Color, Piece, PieceType};
use serde::{Deserialize, Serialize};

const BOARD_SIZE: usize = 8;
const CASTLING_DISTANCE: i32 = 2;
const PAWN_DOUBLE_MOVE_DISTANCE: i32 = 2;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Move {
    pub from_row: usize,
    pub from_col: usize,
    pub to_row: usize,
    pub to_col: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameStatus {
    Active,
    Check,
    Checkmate,
    Stalemate,
    Draw,
    Resigned,
    DrawOffered,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChessGame {
    pub board: Board,
    pub current_turn: Color,
    pub status: GameStatus,
    pub move_history: Vec<Move>,
    pub captured_by_white: Vec<Piece>,
    pub captured_by_black: Vec<Piece>,
    pub white_score: u32,
    pub black_score: u32,
    pub draw_offered_by: Option<Color>,
    // En-passant tracking: Some((row, col)) if en-passant capture is possible
    pub en_passant_target: Option<(usize, usize)>,
    // Castling rights tracking
    pub white_king_moved: bool,
    pub white_rook_kingside_moved: bool,
    pub white_rook_queenside_moved: bool,
    pub black_king_moved: bool,
    pub black_rook_kingside_moved: bool,
    pub black_rook_queenside_moved: bool,
}

impl ChessGame {
    pub fn new() -> Self {
        Self {
            board: Board::new(),
            current_turn: Color::White,
            status: GameStatus::Active,
            move_history: Vec::new(),
            captured_by_white: Vec::new(),
            captured_by_black: Vec::new(),
            white_score: 0,
            black_score: 0,
            draw_offered_by: None,
            en_passant_target: None,
            white_king_moved: false,
            white_rook_kingside_moved: false,
            white_rook_queenside_moved: false,
            black_king_moved: false,
            black_rook_kingside_moved: false,
            black_rook_queenside_moved: false,
        }
    }

    pub fn make_move(&mut self, chess_move: Move) -> Result<(), String> {
        // Check if game is already over
        if self.status == GameStatus::Checkmate
            || self.status == GameStatus::Stalemate
            || self.status == GameStatus::Resigned
            || self.status == GameStatus::Draw
        {
            return Err("Game is already over".to_string());
        }

        // Clear any pending draw offer when a move is made
        self.draw_offered_by = None;
        if self.status == GameStatus::DrawOffered {
            self.status = GameStatus::Active;
        }

        // Basic validation
        if chess_move.from_row >= BOARD_SIZE
            || chess_move.from_col >= BOARD_SIZE
            || chess_move.to_row >= BOARD_SIZE
            || chess_move.to_col >= BOARD_SIZE
        {
            return Err("Invalid move: coordinates out of bounds".to_string());
        }

        // Check if there's a piece at the source position
        let piece = self
            .board
            .get(chess_move.from_row, chess_move.from_col)
            .ok_or_else(|| "No piece at source position".to_string())?;

        // Check if it's the correct player's turn
        if piece.color != self.current_turn {
            return Err("Not your turn".to_string());
        }

        // Validate the move is legal for the piece type
        if !self.is_valid_move(&chess_move, &piece) {
            return Err("Invalid move for this piece".to_string());
        }

        // Check if destination has a piece and whether it can be captured
        let captured_piece = self.board.get(chess_move.to_row, chess_move.to_col);
        if let Some(dest_piece) = captured_piece {
            if dest_piece.color == piece.color {
                return Err("Cannot capture your own piece".to_string());
            }
        }

        // Check if this is a castling move
        let is_castling = piece.piece_type == PieceType::King
            && (chess_move.to_col as i32 - chess_move.from_col as i32).abs() == CASTLING_DISTANCE;

        // Check if the move would leave the king in check
        if !is_castling && self.would_move_leave_king_in_check(&chess_move, piece.color) {
            return Err("Move would leave king in check".to_string());
        }

        // Handle castling separately
        if is_castling {
            return self.execute_castling(&chess_move, piece.color);
        }

        // Check for en-passant capture
        let is_en_passant = piece.piece_type == PieceType::Pawn
            && chess_move.from_col != chess_move.to_col
            && captured_piece.is_none()
            && Some((chess_move.to_row, chess_move.to_col)) == self.en_passant_target;

        // Make the move
        self.board.move_piece(
            chess_move.from_row,
            chess_move.from_col,
            chess_move.to_row,
            chess_move.to_col,
        );

        // Update castling rights based on piece movement
        self.update_castling_rights(&chess_move, &piece);

        // Handle en-passant capture
        let mut actual_captured = captured_piece;
        if is_en_passant {
            // Remove the pawn that was captured en-passant
            let captured_pawn_row = chess_move.from_row;
            let captured_pawn_col = chess_move.to_col;
            actual_captured = self.board.get(captured_pawn_row, captured_pawn_col);
            self.board.set(captured_pawn_row, captured_pawn_col, None);
        }

        // Update captured pieces and scores
        if let Some(captured) = actual_captured {
            let value = captured.piece_type.value();
            match piece.color {
                Color::White => {
                    self.captured_by_white.push(captured);
                    self.white_score += value;
                }
                Color::Black => {
                    self.captured_by_black.push(captured);
                    self.black_score += value;
                }
            }
        }

        // Update en-passant target square
        self.en_passant_target = None;
        if piece.piece_type == PieceType::Pawn
            && (chess_move.to_row as i32 - chess_move.from_row as i32).abs()
                == PAWN_DOUBLE_MOVE_DISTANCE
        {
            // A pawn moved two squares, set en-passant target
            let target_row = (chess_move.from_row + chess_move.to_row) / 2;
            self.en_passant_target = Some((target_row, chess_move.from_col));
        }

        // Record the move
        self.move_history.push(chess_move);

        // Switch turns
        self.current_turn = self.current_turn.opposite();

        // Update game status (check for check, checkmate, or stalemate)
        self.update_game_status();

        Ok(())
    }

    fn is_valid_move(&self, chess_move: &Move, piece: &Piece) -> bool {
        let from_row = chess_move.from_row as i32;
        let from_col = chess_move.from_col as i32;
        let to_row = chess_move.to_row as i32;
        let to_col = chess_move.to_col as i32;

        let row_diff = (to_row - from_row).abs();
        let col_diff = (to_col - from_col).abs();

        match piece.piece_type {
            PieceType::Pawn => {
                const WHITE_DIRECTION: i32 = 1;
                const BLACK_DIRECTION: i32 = -1;
                const WHITE_START_ROW: i32 = 1;
                const BLACK_START_ROW: i32 = 6;

                let direction = if piece.color == Color::White {
                    WHITE_DIRECTION
                } else {
                    BLACK_DIRECTION
                };
                let start_row = if piece.color == Color::White {
                    WHITE_START_ROW
                } else {
                    BLACK_START_ROW
                };

                // Forward move
                if from_col == to_col {
                    if to_row == from_row + direction {
                        // Single step forward
                        return self
                            .board
                            .get(chess_move.to_row, chess_move.to_col)
                            .is_none();
                    } else if from_row == start_row && to_row == from_row + (2 * direction) {
                        // Double step from start
                        let middle_row = (from_row + direction) as usize;
                        return self
                            .board
                            .get(chess_move.to_row, chess_move.to_col)
                            .is_none()
                            && self.board.get(middle_row, chess_move.from_col).is_none();
                    }
                }
                // Diagonal capture (regular or en-passant)
                else if col_diff == 1 && to_row == from_row + direction {
                    if let Some(target) = self.board.get(chess_move.to_row, chess_move.to_col) {
                        // Regular capture
                        return target.color != piece.color;
                    } else if Some((chess_move.to_row, chess_move.to_col)) == self.en_passant_target
                    {
                        // En-passant capture
                        return true;
                    }
                }
                false
            }
            PieceType::Knight => {
                (row_diff == 2 && col_diff == 1) || (row_diff == 1 && col_diff == 2)
            }
            PieceType::Bishop => {
                row_diff == col_diff && row_diff > 0 && self.is_path_clear(chess_move)
            }
            PieceType::Rook => {
                (row_diff == 0 || col_diff == 0)
                    && (row_diff + col_diff > 0)
                    && self.is_path_clear(chess_move)
            }
            PieceType::Queen => {
                ((row_diff == col_diff) || (row_diff == 0 || col_diff == 0))
                    && (row_diff + col_diff > 0)
                    && self.is_path_clear(chess_move)
            }
            PieceType::King => {
                // Normal king move (one square in any direction)
                if row_diff <= 1 && col_diff <= 1 && (row_diff + col_diff > 0) {
                    return true;
                }
                // Castling (king moves two squares horizontally)
                if row_diff == 0 && col_diff == CASTLING_DISTANCE {
                    return self.can_castle(chess_move, piece.color);
                }
                false
            }
        }
    }

    fn is_path_clear(&self, chess_move: &Move) -> bool {
        let from_row = chess_move.from_row as i32;
        let from_col = chess_move.from_col as i32;
        let to_row = chess_move.to_row as i32;
        let to_col = chess_move.to_col as i32;

        let row_step = (to_row - from_row).signum();
        let col_step = (to_col - from_col).signum();

        let mut current_row = from_row + row_step;
        let mut current_col = from_col + col_step;

        while current_row != to_row || current_col != to_col {
            if self
                .board
                .get(current_row as usize, current_col as usize)
                .is_some()
            {
                return false;
            }
            current_row += row_step;
            current_col += col_step;
        }

        true
    }

    /// Find the position of the king for a given color
    fn find_king(&self, color: Color) -> Option<(usize, usize)> {
        for row in 0..BOARD_SIZE {
            for col in 0..BOARD_SIZE {
                if let Some(piece) = self.board.get(row, col) {
                    if piece.piece_type == PieceType::King && piece.color == color {
                        return Some((row, col));
                    }
                }
            }
        }
        None
    }

    /// Check if a square is under attack by the opponent
    fn is_square_under_attack(&self, row: usize, col: usize, by_color: Color) -> bool {
        // Check all squares for opponent pieces that can attack this position
        for from_row in 0..BOARD_SIZE {
            for from_col in 0..BOARD_SIZE {
                if let Some(piece) = self.board.get(from_row, from_col) {
                    if piece.color == by_color {
                        let test_move = Move {
                            from_row,
                            from_col,
                            to_row: row,
                            to_col: col,
                        };
                        // For attack purposes, we check if the move is valid
                        // even if there's a piece at the destination
                        if self.is_valid_move(&test_move, &piece) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    /// Check if the king of a given color is in check
    fn is_king_in_check(&self, color: Color) -> bool {
        if let Some((king_row, king_col)) = self.find_king(color) {
            self.is_square_under_attack(king_row, king_col, color.opposite())
        } else {
            false
        }
    }

    /// Check if a move would leave the moving player's king in check
    fn would_move_leave_king_in_check(&self, chess_move: &Move, piece_color: Color) -> bool {
        // Make a copy of the board to test the move
        let mut test_game = self.clone();
        test_game.board.move_piece(
            chess_move.from_row,
            chess_move.from_col,
            chess_move.to_row,
            chess_move.to_col,
        );
        test_game.is_king_in_check(piece_color)
    }

    /// Check if a player has any legal moves
    fn has_any_legal_moves(&self, color: Color) -> bool {
        for from_row in 0..BOARD_SIZE {
            for from_col in 0..BOARD_SIZE {
                if let Some(piece) = self.board.get(from_row, from_col) {
                    if piece.color == color {
                        // Try all possible destination squares
                        for to_row in 0..BOARD_SIZE {
                            for to_col in 0..BOARD_SIZE {
                                let test_move = Move {
                                    from_row,
                                    from_col,
                                    to_row,
                                    to_col,
                                };

                                // Check if this is a valid move
                                if self.is_valid_move(&test_move, &piece) {
                                    // Check destination doesn't have friendly piece
                                    if let Some(dest_piece) = self.board.get(to_row, to_col) {
                                        if dest_piece.color == piece.color {
                                            continue;
                                        }
                                    }

                                    // Check if move would leave king in check
                                    if !self.would_move_leave_king_in_check(&test_move, color) {
                                        return true;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        false
    }

    /// Update the game status based on the current board position
    fn update_game_status(&mut self) {
        let in_check = self.is_king_in_check(self.current_turn);
        let has_legal_moves = self.has_any_legal_moves(self.current_turn);

        if in_check && !has_legal_moves {
            self.status = GameStatus::Checkmate;
        } else if !in_check && !has_legal_moves {
            self.status = GameStatus::Stalemate;
        } else if in_check {
            self.status = GameStatus::Check;
        } else if self.draw_offered_by.is_some() {
            self.status = GameStatus::DrawOffered;
        } else {
            self.status = GameStatus::Active;
        }
    }

    /// Resign the game for the given color
    pub fn resign(&mut self, _color: Color) -> Result<(), String> {
        // Check if game is already over
        if self.status == GameStatus::Checkmate
            || self.status == GameStatus::Stalemate
            || self.status == GameStatus::Resigned
            || self.status == GameStatus::Draw
        {
            return Err("Game is already over".to_string());
        }

        self.status = GameStatus::Resigned;
        Ok(())
    }

    /// Offer a draw for the given color
    pub fn offer_draw(&mut self, color: Color) -> Result<(), String> {
        // Check if game is already over
        if self.status == GameStatus::Checkmate
            || self.status == GameStatus::Stalemate
            || self.status == GameStatus::Resigned
            || self.status == GameStatus::Draw
        {
            return Err("Game is already over".to_string());
        }

        // Check if it's this player's turn
        if self.current_turn != color {
            return Err("Not your turn to offer a draw".to_string());
        }

        self.draw_offered_by = Some(color);
        self.status = GameStatus::DrawOffered;
        Ok(())
    }

    /// Accept a draw offer for the given color
    pub fn accept_draw(&mut self, color: Color) -> Result<(), String> {
        // Check if there is a draw offer
        if self.draw_offered_by.is_none() {
            return Err("No draw offer to accept".to_string());
        }

        // Check if the draw was offered by the opponent
        if self.draw_offered_by == Some(color) {
            return Err("You cannot accept your own draw offer".to_string());
        }

        self.status = GameStatus::Draw;
        self.draw_offered_by = None;
        Ok(())
    }

    /// Update castling rights based on piece movement
    fn update_castling_rights(&mut self, chess_move: &Move, piece: &Piece) {
        match piece.piece_type {
            PieceType::King => {
                if piece.color == Color::White {
                    self.white_king_moved = true;
                } else {
                    self.black_king_moved = true;
                }
            }
            PieceType::Rook => {
                // Check which rook moved based on starting position
                if piece.color == Color::White && chess_move.from_row == 0 {
                    if chess_move.from_col == 0 {
                        self.white_rook_queenside_moved = true;
                    } else if chess_move.from_col == 7 {
                        self.white_rook_kingside_moved = true;
                    }
                } else if piece.color == Color::Black && chess_move.from_row == 7 {
                    if chess_move.from_col == 0 {
                        self.black_rook_queenside_moved = true;
                    } else if chess_move.from_col == 7 {
                        self.black_rook_kingside_moved = true;
                    }
                }
            }
            _ => {}
        }
    }

    /// Check if castling is possible for the given move
    fn can_castle(&self, chess_move: &Move, color: Color) -> bool {
        let (king_row, king_moved, rook_kingside_moved, rook_queenside_moved) = match color {
            Color::White => (
                0,
                self.white_king_moved,
                self.white_rook_kingside_moved,
                self.white_rook_queenside_moved,
            ),
            Color::Black => (
                7,
                self.black_king_moved,
                self.black_rook_kingside_moved,
                self.black_rook_queenside_moved,
            ),
        };

        // King must not have moved
        if king_moved {
            return false;
        }

        // Check if king is on correct starting position
        if chess_move.from_row != king_row || chess_move.from_col != 4 {
            return false;
        }

        // Determine if kingside or queenside castling
        let is_kingside = chess_move.to_col == 6;
        let is_queenside = chess_move.to_col == 2;

        if !is_kingside && !is_queenside {
            return false;
        }

        // Check if the corresponding rook has moved
        if is_kingside && rook_kingside_moved {
            return false;
        }
        if is_queenside && rook_queenside_moved {
            return false;
        }

        // Check if squares between king and rook are empty
        if is_kingside {
            // Kingside: f1/f8 (col 5) and g1/g8 (col 6)
            if self.board.get(king_row, 5).is_some() || self.board.get(king_row, 6).is_some() {
                return false;
            }
        } else {
            // Queenside: d1/d8 (col 3), c1/c8 (col 2), and b1/b8 (col 1)
            if self.board.get(king_row, 3).is_some()
                || self.board.get(king_row, 2).is_some()
                || self.board.get(king_row, 1).is_some()
            {
                return false;
            }
        }

        // Check that king is not in check
        if self.is_king_in_check(color) {
            return false;
        }

        // Check that king doesn't pass through check
        let squares_to_check = if is_kingside {
            vec![5, 6] // f1/f8 and g1/g8
        } else {
            vec![3, 2] // d1/d8 and c1/c8
        };

        for col in squares_to_check {
            if self.is_square_under_attack(king_row, col, color.opposite()) {
                return false;
            }
        }

        true
    }

    /// Execute a castling move
    fn execute_castling(&mut self, chess_move: &Move, color: Color) -> Result<(), String> {
        // Validate castling is legal
        if !self.can_castle(chess_move, color) {
            return Err("Castling is not allowed".to_string());
        }

        let king_row = chess_move.from_row;
        let is_kingside = chess_move.to_col == 6;

        // Move the king
        self.board.move_piece(
            chess_move.from_row,
            chess_move.from_col,
            chess_move.to_row,
            chess_move.to_col,
        );

        // Move the rook
        if is_kingside {
            // Kingside: rook moves from h-file (7) to f-file (5)
            self.board.move_piece(king_row, 7, king_row, 5);
        } else {
            // Queenside: rook moves from a-file (0) to d-file (3)
            self.board.move_piece(king_row, 0, king_row, 3);
        }

        // Update castling rights
        if color == Color::White {
            self.white_king_moved = true;
        } else {
            self.black_king_moved = true;
        }

        // Clear en-passant target
        self.en_passant_target = None;

        // Record the move
        self.move_history.push(chess_move.clone());

        // Switch turns
        self.current_turn = self.current_turn.opposite();

        // Update game status
        self.update_game_status();

        Ok(())
    }
}

impl Default for ChessGame {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_game() {
        let game = ChessGame::new();
        assert_eq!(game.current_turn, Color::White);
        assert_eq!(game.status, GameStatus::Active);
        assert_eq!(game.move_history.len(), 0);
    }

    #[test]
    fn test_pawn_forward_move() {
        let mut game = ChessGame::new();
        let chess_move = Move {
            from_row: 1,
            from_col: 4,
            to_row: 2,
            to_col: 4,
        };
        assert!(game.make_move(chess_move).is_ok());
        assert_eq!(game.current_turn, Color::Black);
    }

    #[test]
    fn test_pawn_double_move_from_start() {
        let mut game = ChessGame::new();
        let chess_move = Move {
            from_row: 1,
            from_col: 4,
            to_row: 3,
            to_col: 4,
        };
        assert!(game.make_move(chess_move).is_ok());
    }

    #[test]
    fn test_invalid_move_wrong_turn() {
        let mut game = ChessGame::new();
        let chess_move = Move {
            from_row: 6,
            from_col: 4,
            to_row: 5,
            to_col: 4,
        };
        assert!(game.make_move(chess_move).is_err());
    }

    #[test]
    fn test_knight_move() {
        let mut game = ChessGame::new();
        let chess_move = Move {
            from_row: 0,
            from_col: 1,
            to_row: 2,
            to_col: 2,
        };
        assert!(game.make_move(chess_move).is_ok());
    }

    #[test]
    fn test_invalid_move_out_of_bounds() {
        let mut game = ChessGame::new();
        let chess_move = Move {
            from_row: 1,
            from_col: 4,
            to_row: 10,
            to_col: 4,
        };
        assert!(game.make_move(chess_move).is_err());
    }

    #[test]
    fn test_initial_scores_are_zero() {
        let game = ChessGame::new();
        assert_eq!(game.white_score, 0);
        assert_eq!(game.black_score, 0);
        assert_eq!(game.captured_by_white.len(), 0);
        assert_eq!(game.captured_by_black.len(), 0);
    }

    #[test]
    fn test_capture_updates_score() {
        let mut game = ChessGame::new();

        // Move white pawn forward (e2 to e4)
        game.make_move(Move {
            from_row: 1,
            from_col: 4,
            to_row: 3,
            to_col: 4,
        })
        .unwrap();

        // Move black pawn forward (d7 to d5)
        game.make_move(Move {
            from_row: 6,
            from_col: 3,
            to_row: 4,
            to_col: 3,
        })
        .unwrap();

        // White captures black pawn (e4 to d5)
        game.make_move(Move {
            from_row: 3,
            from_col: 4,
            to_row: 4,
            to_col: 3,
        })
        .unwrap();

        // Verify white captured a pawn
        assert_eq!(game.white_score, 1);
        assert_eq!(game.black_score, 0);
        assert_eq!(game.captured_by_white.len(), 1);
        assert_eq!(game.captured_by_white[0].piece_type, PieceType::Pawn);
    }

    #[test]
    fn test_multiple_captures_accumulate_score() {
        let mut game = ChessGame::new();

        // White pawn e2 to e4
        game.make_move(Move {
            from_row: 1,
            from_col: 4,
            to_row: 3,
            to_col: 4,
        })
        .unwrap();

        // Black pawn d7 to d5
        game.make_move(Move {
            from_row: 6,
            from_col: 3,
            to_row: 4,
            to_col: 3,
        })
        .unwrap();

        // White captures black pawn (e4 to d5) - score +1
        game.make_move(Move {
            from_row: 3,
            from_col: 4,
            to_row: 4,
            to_col: 3,
        })
        .unwrap();

        // Black pawn e7 to e6
        game.make_move(Move {
            from_row: 6,
            from_col: 4,
            to_row: 5,
            to_col: 4,
        })
        .unwrap();

        // White pawn d5 to e6 - captures black pawn, score +1 more
        game.make_move(Move {
            from_row: 4,
            from_col: 3,
            to_row: 5,
            to_col: 4,
        })
        .unwrap();

        // Verify accumulated score
        assert_eq!(game.white_score, 2); // 1 (pawn) + 1 (pawn)
        assert_eq!(game.captured_by_white.len(), 2);
    }

    #[test]
    fn test_check_detection() {
        // Set up a position where white king is in check
        let mut game = ChessGame::new();
        game.board = Board::empty();

        // White king at e1 (0, 4)
        game.board
            .set(0, 4, Some(Piece::new(PieceType::King, Color::White)));
        // Black rook at e8 (7, 4) - attacking the white king
        game.board
            .set(7, 4, Some(Piece::new(PieceType::Rook, Color::Black)));
        // Black king at a8 (7, 0)
        game.board
            .set(7, 0, Some(Piece::new(PieceType::King, Color::Black)));

        assert!(game.is_king_in_check(Color::White));
        assert!(!game.is_king_in_check(Color::Black));
    }

    #[test]
    fn test_move_into_check_rejected() {
        // Set up a position where moving would put own king in check
        let mut game = ChessGame::new();
        game.board = Board::empty();

        // White king at e1 (0, 4)
        game.board
            .set(0, 4, Some(Piece::new(PieceType::King, Color::White)));
        // White bishop at d2 (1, 3) blocking a diagonal check
        game.board
            .set(1, 3, Some(Piece::new(PieceType::Bishop, Color::White)));
        // Black bishop at a5 (4, 0) can attack king diagonally through d2
        game.board
            .set(4, 0, Some(Piece::new(PieceType::Bishop, Color::Black)));
        // Black king at a8 (7, 0)
        game.board
            .set(7, 0, Some(Piece::new(PieceType::King, Color::Black)));

        game.current_turn = Color::White;

        // Try to move the bishop away, which would expose the king to check
        let chess_move = Move {
            from_row: 1,
            from_col: 3,
            to_row: 2,
            to_col: 4,
        };
        let result = game.make_move(chess_move);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Move would leave king in check");
    }

    #[test]
    fn test_checkmate_detection() {
        // Set up a back rank checkmate position
        let mut game = ChessGame::new();
        game.board = Board::empty();

        // Simpler checkmate: white king in corner with no escape
        // White king at a1 (0, 0)
        game.board
            .set(0, 0, Some(Piece::new(PieceType::King, Color::White)));
        // Black queen at b2 (1, 1) will deliver checkmate (protected by king)
        game.board
            .set(7, 1, Some(Piece::new(PieceType::Queen, Color::Black)));
        // Black king at c3 (2, 2) protects the queen
        game.board
            .set(2, 2, Some(Piece::new(PieceType::King, Color::Black)));

        game.current_turn = Color::Black;

        // Move black queen to deliver checkmate at b2
        let chess_move = Move {
            from_row: 7,
            from_col: 1,
            to_row: 1,
            to_col: 1,
        };
        assert!(game.make_move(chess_move).is_ok());
        assert_eq!(game.status, GameStatus::Checkmate);
    }

    #[test]
    fn test_stalemate_detection() {
        // Set up a stalemate position
        let mut game = ChessGame::new();
        game.board = Board::empty();

        // White king at a1 (0, 0)
        game.board
            .set(0, 0, Some(Piece::new(PieceType::King, Color::White)));
        // Black king at c2 (1, 2) - controls b1 and b2
        game.board
            .set(1, 2, Some(Piece::new(PieceType::King, Color::Black)));
        // Black rook at b8 (7, 1) will move to deliver stalemate
        game.board
            .set(7, 1, Some(Piece::new(PieceType::Rook, Color::Black)));

        game.current_turn = Color::Black;

        // Move black rook to b2 to create stalemate
        // After this move, white king at a1 cannot move:
        // - a2 is controlled by black king at c2
        // - b1 is controlled by black king at c2
        // - b2 is occupied by black rook
        let chess_move = Move {
            from_row: 7,
            from_col: 1,
            to_row: 1,
            to_col: 1,
        };
        let result = game.make_move(chess_move);
        if let Err(e) = &result {
            panic!("Move failed with error: {}", e);
        }
        assert!(result.is_ok());
        assert_eq!(game.status, GameStatus::Stalemate);
    }

    #[test]
    fn test_check_status_after_checking_move() {
        // Test that status is updated to Check when a checking move is made
        let mut game = ChessGame::new();
        game.board = Board::empty();

        // White king at e1 (0, 4)
        game.board
            .set(0, 4, Some(Piece::new(PieceType::King, Color::White)));
        // Black rook at a8 (7, 0)
        game.board
            .set(7, 0, Some(Piece::new(PieceType::Rook, Color::Black)));
        // Black king at h8 (7, 7)
        game.board
            .set(7, 7, Some(Piece::new(PieceType::King, Color::Black)));

        game.current_turn = Color::Black;

        // Move black rook to check white king
        let chess_move = Move {
            from_row: 7,
            from_col: 0,
            to_row: 0,
            to_col: 0,
        };
        assert!(game.make_move(chess_move).is_ok());
        assert_eq!(game.status, GameStatus::Check);
    }

    #[test]
    fn test_cannot_move_when_game_is_over() {
        let mut game = ChessGame::new();
        game.status = GameStatus::Checkmate;

        let chess_move = Move {
            from_row: 1,
            from_col: 4,
            to_row: 2,
            to_col: 4,
        };
        let result = game.make_move(chess_move);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Game is already over");
    }

    #[test]
    fn test_king_can_escape_check() {
        // Test that a king in check can move out of check
        let mut game = ChessGame::new();
        game.board = Board::empty();

        // White king at e1 (0, 4) - in check from rook
        game.board
            .set(0, 4, Some(Piece::new(PieceType::King, Color::White)));
        // Black rook at e8 (7, 4) - giving check
        game.board
            .set(7, 4, Some(Piece::new(PieceType::Rook, Color::Black)));
        // Black king at a8 (7, 0)
        game.board
            .set(7, 0, Some(Piece::new(PieceType::King, Color::Black)));

        game.current_turn = Color::White;
        game.status = GameStatus::Check;

        // King moves to d1 to escape check
        let chess_move = Move {
            from_row: 0,
            from_col: 4,
            to_row: 0,
            to_col: 3,
        };
        assert!(game.make_move(chess_move).is_ok());
        assert_eq!(game.status, GameStatus::Active);
    }

    #[test]
    fn test_resign_white() {
        let mut game = ChessGame::new();
        assert_eq!(game.status, GameStatus::Active);

        let result = game.resign(Color::White);
        assert!(result.is_ok());
        assert_eq!(game.status, GameStatus::Resigned);
    }

    #[test]
    fn test_resign_black() {
        let mut game = ChessGame::new();
        assert_eq!(game.status, GameStatus::Active);

        let result = game.resign(Color::Black);
        assert!(result.is_ok());
        assert_eq!(game.status, GameStatus::Resigned);
    }

    #[test]
    fn test_cannot_resign_after_checkmate() {
        let mut game = ChessGame::new();
        game.status = GameStatus::Checkmate;

        let result = game.resign(Color::White);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Game is already over");
    }

    #[test]
    fn test_cannot_move_after_resignation() {
        let mut game = ChessGame::new();
        game.resign(Color::White).unwrap();

        let chess_move = Move {
            from_row: 1,
            from_col: 4,
            to_row: 2,
            to_col: 4,
        };
        let result = game.make_move(chess_move);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Game is already over");
    }

    #[test]
    fn test_offer_draw() {
        let mut game = ChessGame::new();
        assert_eq!(game.status, GameStatus::Active);
        assert_eq!(game.draw_offered_by, None);

        let result = game.offer_draw(Color::White);
        assert!(result.is_ok());
        assert_eq!(game.status, GameStatus::DrawOffered);
        assert_eq!(game.draw_offered_by, Some(Color::White));
    }

    #[test]
    fn test_cannot_offer_draw_on_opponent_turn() {
        let mut game = ChessGame::new();
        assert_eq!(game.current_turn, Color::White);

        let result = game.offer_draw(Color::Black);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Not your turn to offer a draw");
    }

    #[test]
    fn test_accept_draw() {
        let mut game = ChessGame::new();

        // White offers draw
        game.offer_draw(Color::White).unwrap();
        assert_eq!(game.status, GameStatus::DrawOffered);

        // Black accepts draw
        let result = game.accept_draw(Color::Black);
        assert!(result.is_ok());
        assert_eq!(game.status, GameStatus::Draw);
        assert_eq!(game.draw_offered_by, None);
    }

    #[test]
    fn test_cannot_accept_own_draw_offer() {
        let mut game = ChessGame::new();

        // White offers draw
        game.offer_draw(Color::White).unwrap();

        // White tries to accept own draw
        let result = game.accept_draw(Color::White);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "You cannot accept your own draw offer");
    }

    #[test]
    fn test_cannot_accept_draw_without_offer() {
        let mut game = ChessGame::new();

        let result = game.accept_draw(Color::Black);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "No draw offer to accept");
    }

    #[test]
    fn test_draw_offer_cleared_on_move() {
        let mut game = ChessGame::new();

        // White offers draw
        game.offer_draw(Color::White).unwrap();
        assert_eq!(game.draw_offered_by, Some(Color::White));
        assert_eq!(game.status, GameStatus::DrawOffered);

        // White makes a move instead
        let chess_move = Move {
            from_row: 1,
            from_col: 4,
            to_row: 2,
            to_col: 4,
        };
        game.make_move(chess_move).unwrap();

        // Draw offer should be cleared
        assert_eq!(game.draw_offered_by, None);
        assert_eq!(game.status, GameStatus::Active);
    }

    #[test]
    fn test_cannot_move_after_draw_accepted() {
        let mut game = ChessGame::new();

        // Offer and accept draw
        game.offer_draw(Color::White).unwrap();
        game.accept_draw(Color::Black).unwrap();
        assert_eq!(game.status, GameStatus::Draw);

        // Try to make a move
        let chess_move = Move {
            from_row: 1,
            from_col: 4,
            to_row: 2,
            to_col: 4,
        };
        let result = game.make_move(chess_move);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Game is already over");
    }

    #[test]
    fn test_en_passant_white() {
        let mut game = ChessGame::new();

        // White pawn e2 to e4
        game.make_move(Move {
            from_row: 1,
            from_col: 4,
            to_row: 3,
            to_col: 4,
        })
        .unwrap();

        // Black pawn a7 to a6 (random move)
        game.make_move(Move {
            from_row: 6,
            from_col: 0,
            to_row: 5,
            to_col: 0,
        })
        .unwrap();

        // White pawn e4 to e5
        game.make_move(Move {
            from_row: 3,
            from_col: 4,
            to_row: 4,
            to_col: 4,
        })
        .unwrap();

        // Black pawn d7 to d5 (double move, creates en-passant opportunity)
        game.make_move(Move {
            from_row: 6,
            from_col: 3,
            to_row: 4,
            to_col: 3,
        })
        .unwrap();

        // Verify en-passant target is set
        assert_eq!(game.en_passant_target, Some((5, 3)));

        // White captures en-passant: e5 to d6
        let result = game.make_move(Move {
            from_row: 4,
            from_col: 4,
            to_row: 5,
            to_col: 3,
        });
        assert!(result.is_ok());

        // Verify the black pawn at d5 was captured
        assert!(game.board.get(4, 3).is_none());
        assert_eq!(game.white_score, 1);
        assert_eq!(game.captured_by_white.len(), 1);
    }

    #[test]
    fn test_en_passant_black() {
        let mut game = ChessGame::new();

        // White pawn a2 to a3 (random move)
        game.make_move(Move {
            from_row: 1,
            from_col: 0,
            to_row: 2,
            to_col: 0,
        })
        .unwrap();

        // Black pawn e7 to e5
        game.make_move(Move {
            from_row: 6,
            from_col: 4,
            to_row: 4,
            to_col: 4,
        })
        .unwrap();

        // White pawn a3 to a4 (random move)
        game.make_move(Move {
            from_row: 2,
            from_col: 0,
            to_row: 3,
            to_col: 0,
        })
        .unwrap();

        // Black pawn e5 to e4
        game.make_move(Move {
            from_row: 4,
            from_col: 4,
            to_row: 3,
            to_col: 4,
        })
        .unwrap();

        // White pawn d2 to d4 (double move, creates en-passant opportunity)
        game.make_move(Move {
            from_row: 1,
            from_col: 3,
            to_row: 3,
            to_col: 3,
        })
        .unwrap();

        // Verify en-passant target is set
        assert_eq!(game.en_passant_target, Some((2, 3)));

        // Black captures en-passant: e4 to d3
        let result = game.make_move(Move {
            from_row: 3,
            from_col: 4,
            to_row: 2,
            to_col: 3,
        });
        assert!(result.is_ok());

        // Verify the white pawn at d4 was captured
        assert!(game.board.get(3, 3).is_none());
        assert_eq!(game.black_score, 1);
        assert_eq!(game.captured_by_black.len(), 1);
    }

    #[test]
    fn test_en_passant_expires_after_one_turn() {
        let mut game = ChessGame::new();

        // White pawn e2 to e4
        game.make_move(Move {
            from_row: 1,
            from_col: 4,
            to_row: 3,
            to_col: 4,
        })
        .unwrap();

        // Black pawn a7 to a6 (random move)
        game.make_move(Move {
            from_row: 6,
            from_col: 0,
            to_row: 5,
            to_col: 0,
        })
        .unwrap();

        // White pawn e4 to e5
        game.make_move(Move {
            from_row: 3,
            from_col: 4,
            to_row: 4,
            to_col: 4,
        })
        .unwrap();

        // Black pawn d7 to d5 (double move, creates en-passant opportunity)
        game.make_move(Move {
            from_row: 6,
            from_col: 3,
            to_row: 4,
            to_col: 3,
        })
        .unwrap();

        // White makes a different move (not en-passant)
        game.make_move(Move {
            from_row: 1,
            from_col: 0,
            to_row: 2,
            to_col: 0,
        })
        .unwrap();

        // En-passant should no longer be available
        assert_eq!(game.en_passant_target, None);
    }

    #[test]
    fn test_white_kingside_castle() {
        let mut game = ChessGame::new();
        game.board = Board::empty();

        // Setup position for white kingside castling
        game.board
            .set(0, 4, Some(Piece::new(PieceType::King, Color::White)));
        game.board
            .set(0, 7, Some(Piece::new(PieceType::Rook, Color::White)));
        game.board
            .set(7, 4, Some(Piece::new(PieceType::King, Color::Black)));

        game.current_turn = Color::White;

        // Execute kingside castle (e1 to g1)
        let result = game.make_move(Move {
            from_row: 0,
            from_col: 4,
            to_row: 0,
            to_col: 6,
        });
        assert!(result.is_ok());

        // Verify king moved to g1
        assert_eq!(
            game.board.get(0, 6),
            Some(Piece::new(PieceType::King, Color::White))
        );
        // Verify rook moved to f1
        assert_eq!(
            game.board.get(0, 5),
            Some(Piece::new(PieceType::Rook, Color::White))
        );
        // Verify original positions are empty
        assert!(game.board.get(0, 4).is_none());
        assert!(game.board.get(0, 7).is_none());
    }

    #[test]
    fn test_white_queenside_castle() {
        let mut game = ChessGame::new();
        game.board = Board::empty();

        // Setup position for white queenside castling
        game.board
            .set(0, 4, Some(Piece::new(PieceType::King, Color::White)));
        game.board
            .set(0, 0, Some(Piece::new(PieceType::Rook, Color::White)));
        game.board
            .set(7, 4, Some(Piece::new(PieceType::King, Color::Black)));

        game.current_turn = Color::White;

        // Execute queenside castle (e1 to c1)
        let result = game.make_move(Move {
            from_row: 0,
            from_col: 4,
            to_row: 0,
            to_col: 2,
        });
        assert!(result.is_ok());

        // Verify king moved to c1
        assert_eq!(
            game.board.get(0, 2),
            Some(Piece::new(PieceType::King, Color::White))
        );
        // Verify rook moved to d1
        assert_eq!(
            game.board.get(0, 3),
            Some(Piece::new(PieceType::Rook, Color::White))
        );
        // Verify original positions are empty
        assert!(game.board.get(0, 4).is_none());
        assert!(game.board.get(0, 0).is_none());
    }

    #[test]
    fn test_black_kingside_castle() {
        let mut game = ChessGame::new();
        game.board = Board::empty();

        // Setup position for black kingside castling
        game.board
            .set(7, 4, Some(Piece::new(PieceType::King, Color::Black)));
        game.board
            .set(7, 7, Some(Piece::new(PieceType::Rook, Color::Black)));
        game.board
            .set(0, 4, Some(Piece::new(PieceType::King, Color::White)));

        game.current_turn = Color::Black;

        // Execute kingside castle (e8 to g8)
        let result = game.make_move(Move {
            from_row: 7,
            from_col: 4,
            to_row: 7,
            to_col: 6,
        });
        assert!(result.is_ok());

        // Verify king moved to g8
        assert_eq!(
            game.board.get(7, 6),
            Some(Piece::new(PieceType::King, Color::Black))
        );
        // Verify rook moved to f8
        assert_eq!(
            game.board.get(7, 5),
            Some(Piece::new(PieceType::Rook, Color::Black))
        );
    }

    #[test]
    fn test_black_queenside_castle() {
        let mut game = ChessGame::new();
        game.board = Board::empty();

        // Setup position for black queenside castling
        game.board
            .set(7, 4, Some(Piece::new(PieceType::King, Color::Black)));
        game.board
            .set(7, 0, Some(Piece::new(PieceType::Rook, Color::Black)));
        game.board
            .set(0, 4, Some(Piece::new(PieceType::King, Color::White)));

        game.current_turn = Color::Black;

        // Execute queenside castle (e8 to c8)
        let result = game.make_move(Move {
            from_row: 7,
            from_col: 4,
            to_row: 7,
            to_col: 2,
        });
        assert!(result.is_ok());

        // Verify king moved to c8
        assert_eq!(
            game.board.get(7, 2),
            Some(Piece::new(PieceType::King, Color::Black))
        );
        // Verify rook moved to d8
        assert_eq!(
            game.board.get(7, 3),
            Some(Piece::new(PieceType::Rook, Color::Black))
        );
    }

    #[test]
    fn test_cannot_castle_after_king_moved() {
        let mut game = ChessGame::new();
        game.board = Board::empty();

        // Setup position
        game.board
            .set(0, 4, Some(Piece::new(PieceType::King, Color::White)));
        game.board
            .set(0, 7, Some(Piece::new(PieceType::Rook, Color::White)));
        game.board
            .set(7, 4, Some(Piece::new(PieceType::King, Color::Black)));

        game.current_turn = Color::White;

        // Move king
        game.make_move(Move {
            from_row: 0,
            from_col: 4,
            to_row: 0,
            to_col: 5,
        })
        .unwrap();

        // Move it back
        game.current_turn = Color::White;
        game.make_move(Move {
            from_row: 0,
            from_col: 5,
            to_row: 0,
            to_col: 4,
        })
        .unwrap();

        // Try to castle - should fail
        game.current_turn = Color::White;
        let result = game.make_move(Move {
            from_row: 0,
            from_col: 4,
            to_row: 0,
            to_col: 6,
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_cannot_castle_after_rook_moved() {
        let mut game = ChessGame::new();
        game.board = Board::empty();

        // Setup position
        game.board
            .set(0, 4, Some(Piece::new(PieceType::King, Color::White)));
        game.board
            .set(0, 7, Some(Piece::new(PieceType::Rook, Color::White)));
        game.board
            .set(7, 4, Some(Piece::new(PieceType::King, Color::Black)));

        game.current_turn = Color::White;

        // Move rook
        game.make_move(Move {
            from_row: 0,
            from_col: 7,
            to_row: 0,
            to_col: 6,
        })
        .unwrap();

        // Move it back
        game.current_turn = Color::White;
        game.make_move(Move {
            from_row: 0,
            from_col: 6,
            to_row: 0,
            to_col: 7,
        })
        .unwrap();

        // Try to castle - should fail
        game.current_turn = Color::White;
        let result = game.make_move(Move {
            from_row: 0,
            from_col: 4,
            to_row: 0,
            to_col: 6,
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_cannot_castle_through_check() {
        let mut game = ChessGame::new();
        game.board = Board::empty();

        // Setup position where f1 is under attack
        game.board
            .set(0, 4, Some(Piece::new(PieceType::King, Color::White)));
        game.board
            .set(0, 7, Some(Piece::new(PieceType::Rook, Color::White)));
        game.board
            .set(7, 5, Some(Piece::new(PieceType::Rook, Color::Black))); // Attacking f1
        game.board
            .set(7, 4, Some(Piece::new(PieceType::King, Color::Black)));

        game.current_turn = Color::White;

        // Try to castle through check - should fail
        let result = game.make_move(Move {
            from_row: 0,
            from_col: 4,
            to_row: 0,
            to_col: 6,
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_cannot_castle_while_in_check() {
        let mut game = ChessGame::new();
        game.board = Board::empty();

        // Setup position where king is in check
        game.board
            .set(0, 4, Some(Piece::new(PieceType::King, Color::White)));
        game.board
            .set(0, 7, Some(Piece::new(PieceType::Rook, Color::White)));
        game.board
            .set(7, 4, Some(Piece::new(PieceType::Rook, Color::Black))); // Checking the king
        game.board
            .set(7, 0, Some(Piece::new(PieceType::King, Color::Black)));

        game.current_turn = Color::White;

        // Try to castle while in check - should fail
        let result = game.make_move(Move {
            from_row: 0,
            from_col: 4,
            to_row: 0,
            to_col: 6,
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_cannot_castle_with_pieces_in_between() {
        let mut game = ChessGame::new();
        game.board = Board::empty();

        // Setup position with knight blocking
        game.board
            .set(0, 4, Some(Piece::new(PieceType::King, Color::White)));
        game.board
            .set(0, 7, Some(Piece::new(PieceType::Rook, Color::White)));
        game.board
            .set(0, 6, Some(Piece::new(PieceType::Knight, Color::White))); // Blocking
        game.board
            .set(7, 4, Some(Piece::new(PieceType::King, Color::Black)));

        game.current_turn = Color::White;

        // Try to castle - should fail
        let result = game.make_move(Move {
            from_row: 0,
            from_col: 4,
            to_row: 0,
            to_col: 6,
        });
        assert!(result.is_err());
    }
}
