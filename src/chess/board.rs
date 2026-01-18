use super::piece::{Color, Piece, PieceType};
use serde::{Deserialize, Serialize};

const BOARD_SIZE: usize = 8;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Board {
    squares: [[Option<Piece>; BOARD_SIZE]; BOARD_SIZE],
}

impl Board {
    pub fn new() -> Self {
        let mut board = Self {
            squares: [[None; BOARD_SIZE]; BOARD_SIZE],
        };
        board.setup_initial_position();
        board
    }

    #[allow(dead_code)]
    pub fn empty() -> Self {
        Self {
            squares: [[None; BOARD_SIZE]; BOARD_SIZE],
        }
    }

    fn setup_initial_position(&mut self) {
        // Setup pawns
        for col in 0..BOARD_SIZE {
            self.squares[1][col] = Some(Piece::new(PieceType::Pawn, Color::White));
            self.squares[6][col] = Some(Piece::new(PieceType::Pawn, Color::Black));
        }

        // Setup white pieces
        self.squares[0][0] = Some(Piece::new(PieceType::Rook, Color::White));
        self.squares[0][1] = Some(Piece::new(PieceType::Knight, Color::White));
        self.squares[0][2] = Some(Piece::new(PieceType::Bishop, Color::White));
        self.squares[0][3] = Some(Piece::new(PieceType::Queen, Color::White));
        self.squares[0][4] = Some(Piece::new(PieceType::King, Color::White));
        self.squares[0][5] = Some(Piece::new(PieceType::Bishop, Color::White));
        self.squares[0][6] = Some(Piece::new(PieceType::Knight, Color::White));
        self.squares[0][7] = Some(Piece::new(PieceType::Rook, Color::White));

        // Setup black pieces
        self.squares[7][0] = Some(Piece::new(PieceType::Rook, Color::Black));
        self.squares[7][1] = Some(Piece::new(PieceType::Knight, Color::Black));
        self.squares[7][2] = Some(Piece::new(PieceType::Bishop, Color::Black));
        self.squares[7][3] = Some(Piece::new(PieceType::Queen, Color::Black));
        self.squares[7][4] = Some(Piece::new(PieceType::King, Color::Black));
        self.squares[7][5] = Some(Piece::new(PieceType::Bishop, Color::Black));
        self.squares[7][6] = Some(Piece::new(PieceType::Knight, Color::Black));
        self.squares[7][7] = Some(Piece::new(PieceType::Rook, Color::Black));
    }

    pub fn get(&self, row: usize, col: usize) -> Option<Piece> {
        if row < BOARD_SIZE && col < BOARD_SIZE {
            self.squares[row][col]
        } else {
            None
        }
    }

    #[allow(dead_code)]
    pub fn set(&mut self, row: usize, col: usize, piece: Option<Piece>) {
        if row < BOARD_SIZE && col < BOARD_SIZE {
            self.squares[row][col] = piece;
        }
    }

    pub fn move_piece(
        &mut self,
        from_row: usize,
        from_col: usize,
        to_row: usize,
        to_col: usize,
    ) -> bool {
        if from_row >= BOARD_SIZE
            || from_col >= BOARD_SIZE
            || to_row >= BOARD_SIZE
            || to_col >= BOARD_SIZE
        {
            return false;
        }

        if let Some(piece) = self.squares[from_row][from_col] {
            self.squares[to_row][to_col] = Some(piece);
            self.squares[from_row][from_col] = None;
            true
        } else {
            false
        }
    }

    #[allow(dead_code)]
    pub fn is_valid_position(row: i32, col: i32) -> bool {
        row >= 0 && row < BOARD_SIZE as i32 && col >= 0 && col < BOARD_SIZE as i32
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}
