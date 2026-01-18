use axum::{
    extract::{
        ws::{Message, WebSocket},
        Path, State, WebSocketUpgrade,
    },
    response::Response,
};
use futures::{sink::SinkExt, stream::StreamExt};

use crate::game::GameState;

/// Handle WebSocket upgrade request for a specific game
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    Path(game_id): Path<String>,
    State(game_state): State<GameState>,
) -> Response {
    ws.on_upgrade(move |socket| websocket_connection(socket, game_id, game_state))
}

/// Handle an active WebSocket connection
async fn websocket_connection(socket: WebSocket, game_id: String, game_state: GameState) {
    let (mut sender, mut receiver) = socket.split();

    // Subscribe to game updates
    let mut rx = game_state.subscribe_to_game(&game_id).await;

    // Send initial game state
    if let Some(session) = game_state.get_game(&game_id).await {
        if let Ok(json) = serde_json::to_string(&session) {
            if sender.send(Message::Text(json)).await.is_err() {
                tracing::debug!("Failed to send initial state to WebSocket client");
                return;
            }
        }
    } else {
        // Game not found, close connection
        let _ = sender.send(Message::Close(None)).await;
        return;
    }

    // Spawn a task to handle receiving messages from the client
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            // For now, we just ignore client messages
            // In the future, we could handle ping/pong or other client requests
            if matches!(msg, Message::Close(_)) {
                break;
            }
        }
    });

    // Task to send game updates to the client
    let mut send_task = tokio::spawn(async move {
        while let Ok(session) = rx.recv().await {
            if let Ok(json) = serde_json::to_string(&session) {
                if sender.send(Message::Text(json)).await.is_err() {
                    tracing::debug!("Failed to send game update to WebSocket client");
                    break;
                }
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    }

    tracing::debug!("WebSocket connection closed for game {}", game_id);
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_websocket_module_exists() {
        // Basic test to ensure the module compiles
        assert!(true);
    }
}
