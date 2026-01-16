use super::session::GameSession;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct GameState {
    pub games: Arc<RwLock<HashMap<String, GameSession>>>,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            games: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn create_game(&self) -> String {
        let session = GameSession::new();
        let game_id = session.id.clone();

        let mut games = self.games.write().await;
        games.insert(game_id.clone(), session);

        game_id
    }

    pub async fn get_game(&self, game_id: &str) -> Option<GameSession> {
        let games = self.games.read().await;
        games.get(game_id).cloned()
    }

    pub async fn update_game(&self, game_id: &str, session: GameSession) -> bool {
        let mut games = self.games.write().await;
        if games.contains_key(game_id) {
            games.insert(game_id.to_string(), session);
            true
        } else {
            false
        }
    }

    pub async fn join_game(
        &self,
        game_id: &str,
        player_id: String,
    ) -> Result<crate::chess::Color, String> {
        let mut games = self.games.write().await;

        if let Some(session) = games.get_mut(game_id) {
            session.add_player(player_id)
        } else {
            Err("Game not found".to_string())
        }
    }

    pub async fn list_games(&self) -> Vec<(String, bool)> {
        let games = self.games.read().await;
        games
            .iter()
            .map(|(id, session)| (id.clone(), session.is_full()))
            .collect()
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}
