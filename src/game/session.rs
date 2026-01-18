use crate::chess::{ChessGame, Color};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: String,
    pub color: Color,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSession {
    pub id: String,
    pub game: ChessGame,
    pub white_player: Option<Player>,
    pub black_player: Option<Player>,
    pub created_at: u64,
}

impl GameSession {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            game: ChessGame::new(),
            white_player: None,
            black_player: None,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    pub fn add_player(&mut self, player_id: String) -> Result<Color, String> {
        if self.white_player.is_none() {
            self.white_player = Some(Player {
                id: player_id,
                color: Color::White,
            });
            Ok(Color::White)
        } else if self.black_player.is_none() {
            self.black_player = Some(Player {
                id: player_id,
                color: Color::Black,
            });
            Ok(Color::Black)
        } else {
            Err("Game is full".to_string())
        }
    }

    pub fn is_player_in_game(&self, player_id: &str) -> bool {
        self.white_player
            .as_ref()
            .is_some_and(|p| p.id == player_id)
            || self
                .black_player
                .as_ref()
                .is_some_and(|p| p.id == player_id)
    }

    pub fn get_player_color(&self, player_id: &str) -> Option<Color> {
        if self
            .white_player
            .as_ref()
            .is_some_and(|p| p.id == player_id)
        {
            Some(Color::White)
        } else if self
            .black_player
            .as_ref()
            .is_some_and(|p| p.id == player_id)
        {
            Some(Color::Black)
        } else {
            None
        }
    }

    pub fn is_full(&self) -> bool {
        self.white_player.is_some() && self.black_player.is_some()
    }
}

impl Default for GameSession {
    fn default() -> Self {
        Self::new()
    }
}
