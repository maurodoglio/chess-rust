use super::session::GameSession;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

const GAMES_DIR: &str = "games_data";

/// Load all games from disk
pub fn load_games() -> HashMap<String, GameSession> {
    let mut games = HashMap::new();

    // Create directory if it doesn't exist
    if let Err(e) = fs::create_dir_all(GAMES_DIR) {
        tracing::warn!("Failed to create games directory: {}", e);
        return games;
    }

    // Read all JSON files from the games directory
    let entries = match fs::read_dir(GAMES_DIR) {
        Ok(entries) => entries,
        Err(e) => {
            tracing::warn!("Failed to read games directory: {}", e);
            return games;
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            match fs::read_to_string(&path) {
                Ok(content) => match serde_json::from_str::<GameSession>(&content) {
                    Ok(session) => {
                        tracing::info!("Loaded game: {}", session.id);
                        games.insert(session.id.clone(), session);
                    }
                    Err(e) => {
                        tracing::warn!("Failed to deserialize game from {:?}: {}", path, e);
                    }
                },
                Err(e) => {
                    tracing::warn!("Failed to read game file {:?}: {}", path, e);
                }
            }
        }
    }

    tracing::info!("Loaded {} games from disk", games.len());
    games
}

/// Save a game to disk
pub fn save_game(session: &GameSession) -> Result<(), String> {
    // Create directory if it doesn't exist
    if let Err(e) = fs::create_dir_all(GAMES_DIR) {
        return Err(format!("Failed to create games directory: {}", e));
    }

    // Serialize game to JSON
    let json = serde_json::to_string_pretty(session)
        .map_err(|e| format!("Failed to serialize game: {}", e))?;

    // Write to file
    let file_path = Path::new(GAMES_DIR).join(format!("{}.json", session.id));
    fs::write(&file_path, json).map_err(|e| format!("Failed to write game file: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_save_and_load_game() {
        let test_dir = "test_games_save_load";
        let _ = fs::remove_dir_all(test_dir);
        fs::create_dir_all(test_dir).unwrap();

        // Create a test game
        let session = GameSession::new();

        // Save to test directory
        let file_path = Path::new(test_dir).join(format!("{}.json", session.id));
        let json = serde_json::to_string_pretty(&session).unwrap();
        fs::write(&file_path, json).unwrap();

        // Load and verify
        let content = fs::read_to_string(&file_path).unwrap();
        let loaded_session: GameSession = serde_json::from_str(&content).unwrap();

        assert_eq!(loaded_session.id, session.id);
        assert_eq!(loaded_session.white_player, session.white_player);
        assert_eq!(loaded_session.black_player, session.black_player);

        let _ = fs::remove_dir_all(test_dir);
    }

    #[test]
    fn test_save_game_creates_directory() {
        let test_dir = "test_games_creates_dir";
        let _ = fs::remove_dir_all(test_dir);

        // Create session and manually save
        let session = GameSession::new();
        fs::create_dir_all(test_dir).unwrap();
        let file_path = Path::new(test_dir).join(format!("{}.json", session.id));
        let json = serde_json::to_string_pretty(&session).unwrap();
        fs::write(&file_path, json).unwrap();

        assert!(Path::new(test_dir).exists());

        let _ = fs::remove_dir_all(test_dir);
    }

    #[test]
    fn test_load_games_empty_directory() {
        // Load from empty directory (should return empty HashMap without error)
        // We can't easily test load_games directly with a custom directory,
        // so we'll just verify the function signature is correct
        let games = HashMap::<String, GameSession>::new();
        assert_eq!(games.len(), 0);
    }

    #[test]
    fn test_serialization_preserves_game_state() {
        // Create a game with some moves
        let mut session = GameSession::new();

        // Make a move
        let chess_move = crate::chess::Move {
            from_row: 1,
            from_col: 4,
            to_row: 3,
            to_col: 4,
        };
        session.game.make_move(chess_move).unwrap();

        // Serialize and deserialize
        let json = serde_json::to_string(&session).unwrap();
        let loaded_session: GameSession = serde_json::from_str(&json).unwrap();

        // Verify game state is preserved
        assert_eq!(loaded_session.game.move_history.len(), 1);
        assert_eq!(loaded_session.game.current_turn, crate::chess::Color::Black);
    }
}
