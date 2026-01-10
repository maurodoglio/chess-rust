use super::board::Board;
use super::piece::{Color, Piece, PieceType};
use serde::{Deserialize, Serialize};

const BOARD_SIZE: usize = 8;

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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChessGame {
    pub board: Board,
    pub current_turn: Color,
    pub status: GameStatus,
    pub move_history: Vec<Move>,
}

impl ChessGame {
    pub fn new() -> Self {
        Self {
            board: Board::new(),
            current_turn: Color::White,
            status: GameStatus::Active,
            move_history: Vec::new(),
        }
    }

    pub fn make_move(&mut self, chess_move: Move) -> Result<(), String> {
        // Basic validation
        if chess_move.from_row >= BOARD_SIZE || chess_move.from_col >= BOARD_SIZE 
            || chess_move.to_row >= BOARD_SIZE || chess_move.to_col >= BOARD_SIZE {
            return Err("Invalid move: coordinates out of bounds".to_string());
        }

        // Check if there's a piece at the source position
        let piece = self.board.get(chess_move.from_row, chess_move.from_col)
            .ok_or_else(|| "No piece at source position".to_string())?;

        // Check if it's the correct player's turn
        if piece.color != self.current_turn {
            return Err("Not your turn".to_string());
        }

        // Validate the move is legal for the piece type
        if !self.is_valid_move(&chess_move, &piece) {
            return Err("Invalid move for this piece".to_string());
        }

        // Check if destination has a friendly piece
        if let Some(dest_piece) = self.board.get(chess_move.to_row, chess_move.to_col) {
            if dest_piece.color == piece.color {
                return Err("Cannot capture your own piece".to_string());
            }
        }

        // Make the move
        self.board.move_piece(
            chess_move.from_row,
            chess_move.from_col,
            chess_move.to_row,
            chess_move.to_col,
        );

        // Record the move
        self.move_history.push(chess_move);

        // Switch turns
        self.current_turn = self.current_turn.opposite();

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

                let direction = if piece.color == Color::White { WHITE_DIRECTION } else { BLACK_DIRECTION };
                let start_row = if piece.color == Color::White { WHITE_START_ROW } else { BLACK_START_ROW };

                // Forward move
                if from_col == to_col {
                    if to_row == from_row + direction {
                        // Single step forward
                        return self.board.get(chess_move.to_row, chess_move.to_col).is_none();
                    } else if from_row == start_row && to_row == from_row + (2 * direction) {
                        // Double step from start
                        let middle_row = (from_row + direction) as usize;
                        return self.board.get(chess_move.to_row, chess_move.to_col).is_none()
                            && self.board.get(middle_row, chess_move.from_col).is_none();
                    }
                }
                // Diagonal capture
                else if col_diff == 1 && to_row == from_row + direction {
                    if let Some(target) = self.board.get(chess_move.to_row, chess_move.to_col) {
                        return target.color != piece.color;
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
                (row_diff == 0 || col_diff == 0) && (row_diff + col_diff > 0) && self.is_path_clear(chess_move)
            }
            PieceType::Queen => {
                ((row_diff == col_diff) || (row_diff == 0 || col_diff == 0))
                    && (row_diff + col_diff > 0)
                    && self.is_path_clear(chess_move)
            }
            PieceType::King => {
                row_diff <= 1 && col_diff <= 1 && (row_diff + col_diff > 0)
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
            if self.board.get(current_row as usize, current_col as usize).is_some() {
                return false;
            }
            current_row += row_step;
            current_col += col_step;
        }

        true
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
}
