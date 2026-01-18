use super::session::GameSession;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

/// Capacity for broadcast channels used to send game updates to WebSocket clients
const BROADCAST_CHANNEL_CAPACITY: usize = 100;

#[derive(Clone)]
pub struct GameState {
    pub games: Arc<RwLock<HashMap<String, GameSession>>>,
    pub broadcasters: Arc<RwLock<HashMap<String, broadcast::Sender<GameSession>>>>,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            games: Arc::new(RwLock::new(HashMap::new())),
            broadcasters: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn create_game(&self) -> String {
        let session = GameSession::new();
        let game_id = session.id.clone();

        let mut games = self.games.write().await;
        games.insert(game_id.clone(), session.clone());

        // Create a broadcast channel for this game
        let (tx, _) = broadcast::channel(BROADCAST_CHANNEL_CAPACITY);
        let mut broadcasters = self.broadcasters.write().await;
        broadcasters.insert(game_id.clone(), tx);

        game_id
    }

    pub async fn get_game(&self, game_id: &str) -> Option<GameSession> {
        let games = self.games.read().await;
        games.get(game_id).cloned()
    }

    pub async fn update_game(&self, game_id: &str, session: GameSession) -> bool {
        let mut games = self.games.write().await;
        if games.contains_key(game_id) {
            games.insert(game_id.to_string(), session.clone());

            // Broadcast the update to WebSocket clients
            let broadcasters = self.broadcasters.read().await;
            if let Some(tx) = broadcasters.get(game_id) {
                // It's ok if send fails (no receivers)
                let _ = tx.send(session);
            }

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
            let result = session.add_player(player_id);

            // Broadcast the update to WebSocket clients if player joined successfully
            if result.is_ok() {
                let broadcasters = self.broadcasters.read().await;
                if let Some(tx) = broadcasters.get(game_id) {
                    let _ = tx.send(session.clone());
                }
            }

            result
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

    /// Subscribe to updates for a specific game
    pub async fn subscribe_to_game(&self, game_id: &str) -> broadcast::Receiver<GameSession> {
        let mut broadcasters = self.broadcasters.write().await;

        // Get or create broadcaster for this game
        let tx = broadcasters.entry(game_id.to_string()).or_insert_with(|| {
            let (tx, _) = broadcast::channel(BROADCAST_CHANNEL_CAPACITY);
            tx
        });

        tx.subscribe()
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}
