use crate::auth::{self, UserStore};
use crate::chess::Move;
use crate::game::{GameSession, GameState};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    middleware,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateGameResponse {
    pub game_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JoinGameRequest {
    pub player_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JoinGameResponse {
    pub color: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MakeMoveRequest {
    pub player_id: String,
    pub chess_move: Move,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerActionRequest {
    pub player_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GameListResponse {
    pub games: Vec<GameInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GameInfo {
    pub id: String,
    pub is_full: bool,
}

pub fn create_router(game_state: GameState, user_store: UserStore) -> Router {
    // Public routes (no authentication required)
    let public_routes = Router::new()
        .route("/health", get(health_check))
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login));

    // Protected routes (authentication required)
    let protected_routes = Router::new()
        .route("/games", post(create_game))
        .route("/games/list", get(list_games))
        .route("/games/:game_id", get(get_game))
        .route("/games/:game_id/spectate", get(spectate_game))
        .route("/games/:game_id/join", post(join_game))
        .route("/games/:game_id/move", post(make_move))
        .route("/games/:game_id/resign", post(resign_game))
        .route("/games/:game_id/offer-draw", post(offer_draw))
        .route("/games/:game_id/accept-draw", post(accept_draw))
        .layer(middleware::from_fn(auth::auth_middleware))
        .with_state(game_state.clone());

    // Combine routes
    public_routes.merge(protected_routes).with_state(user_store)
}

async fn health_check() -> &'static str {
    "OK"
}

async fn create_game(
    State(game_state): State<GameState>,
) -> Result<Json<CreateGameResponse>, StatusCode> {
    let game_id = game_state.create_game().await;
    Ok(Json(CreateGameResponse { game_id }))
}

async fn list_games(
    State(game_state): State<GameState>,
) -> Result<Json<GameListResponse>, StatusCode> {
    let games = game_state.list_games().await;
    let game_list = games
        .into_iter()
        .map(|(id, is_full)| GameInfo { id, is_full })
        .collect();
    Ok(Json(GameListResponse { games: game_list }))
}

async fn get_game(
    State(game_state): State<GameState>,
    Path(game_id): Path<String>,
) -> Result<Json<GameSession>, (StatusCode, Json<ErrorResponse>)> {
    match game_state.get_game(&game_id).await {
        Some(session) => Ok(Json(session)),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Game not found".to_string(),
            }),
        )),
    }
}

async fn spectate_game(
    State(game_state): State<GameState>,
    Path(game_id): Path<String>,
) -> Result<Json<GameSession>, (StatusCode, Json<ErrorResponse>)> {
    // Spectate endpoint provides the same functionality as get_game,
    // but with a more semantic name for viewing games without joining
    get_game(State(game_state), Path(game_id)).await
}

async fn join_game(
    State(game_state): State<GameState>,
    Path(game_id): Path<String>,
    Json(request): Json<JoinGameRequest>,
) -> Result<Json<JoinGameResponse>, (StatusCode, Json<ErrorResponse>)> {
    match game_state.join_game(&game_id, request.player_id).await {
        Ok(color) => {
            let color_str = match color {
                crate::chess::Color::White => "white",
                crate::chess::Color::Black => "black",
            };
            Ok(Json(JoinGameResponse {
                color: color_str.to_string(),
            }))
        }
        Err(err) => Err((StatusCode::BAD_REQUEST, Json(ErrorResponse { error: err }))),
    }
}

async fn make_move(
    State(game_state): State<GameState>,
    Path(game_id): Path<String>,
    Json(request): Json<MakeMoveRequest>,
) -> Result<Json<GameSession>, (StatusCode, Json<ErrorResponse>)> {
    let mut session = match game_state.get_game(&game_id).await {
        Some(s) => s,
        None => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "Game not found".to_string(),
                }),
            ))
        }
    };

    // Verify player is in the game
    if !session.is_player_in_game(&request.player_id) {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "Player not in this game".to_string(),
            }),
        ));
    }

    // Verify it's the player's turn
    let player_color = session
        .get_player_color(&request.player_id)
        .expect("Player color should exist after verifying player is in game");
    if player_color != session.game.current_turn {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Not your turn".to_string(),
            }),
        ));
    }

    // Make the move
    match session.game.make_move(request.chess_move) {
        Ok(_) => {
            game_state.update_game(&game_id, session.clone()).await;
            Ok(Json(session))
        }
        Err(err) => Err((StatusCode::BAD_REQUEST, Json(ErrorResponse { error: err }))),
    }
}

/// Helper function to get game session and verify player
async fn get_session_and_verify_player(
    game_state: &GameState,
    game_id: &str,
    player_id: &str,
) -> Result<(GameSession, crate::chess::Color), (StatusCode, Json<ErrorResponse>)> {
    let session = match game_state.get_game(game_id).await {
        Some(s) => s,
        None => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "Game not found".to_string(),
                }),
            ))
        }
    };

    // Verify player is in the game
    if !session.is_player_in_game(player_id) {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "Player not in this game".to_string(),
            }),
        ));
    }

    let player_color = session
        .get_player_color(player_id)
        .expect("Player color should exist after verifying player is in game");

    Ok((session, player_color))
}

async fn resign_game(
    State(game_state): State<GameState>,
    Path(game_id): Path<String>,
    Json(request): Json<PlayerActionRequest>,
) -> Result<Json<GameSession>, (StatusCode, Json<ErrorResponse>)> {
    let (mut session, player_color) =
        get_session_and_verify_player(&game_state, &game_id, &request.player_id).await?;

    // Resign the game
    match session.game.resign(player_color) {
        Ok(_) => {
            game_state.update_game(&game_id, session.clone()).await;
            Ok(Json(session))
        }
        Err(err) => Err((StatusCode::BAD_REQUEST, Json(ErrorResponse { error: err }))),
    }
}

async fn offer_draw(
    State(game_state): State<GameState>,
    Path(game_id): Path<String>,
    Json(request): Json<PlayerActionRequest>,
) -> Result<Json<GameSession>, (StatusCode, Json<ErrorResponse>)> {
    let (mut session, player_color) =
        get_session_and_verify_player(&game_state, &game_id, &request.player_id).await?;

    // Offer a draw
    match session.game.offer_draw(player_color) {
        Ok(_) => {
            game_state.update_game(&game_id, session.clone()).await;
            Ok(Json(session))
        }
        Err(err) => Err((StatusCode::BAD_REQUEST, Json(ErrorResponse { error: err }))),
    }
}

async fn accept_draw(
    State(game_state): State<GameState>,
    Path(game_id): Path<String>,
    Json(request): Json<PlayerActionRequest>,
) -> Result<Json<GameSession>, (StatusCode, Json<ErrorResponse>)> {
    let (mut session, player_color) =
        get_session_and_verify_player(&game_state, &game_id, &request.player_id).await?;

    // Accept the draw
    match session.game.accept_draw(player_color) {
        Ok(_) => {
            game_state.update_game(&game_id, session.clone()).await;
            Ok(Json(session))
        }
        Err(err) => Err((StatusCode::BAD_REQUEST, Json(ErrorResponse { error: err }))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chess::Color;

    #[tokio::test]
    async fn test_spectate_game_not_found() {
        let game_state = GameState::new();
        let result =
            spectate_game(State(game_state), Path("nonexistent-game-id".to_string())).await;

        assert!(result.is_err());
        let (status, _) = result.unwrap_err();
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_spectate_game_success() {
        let game_state = GameState::new();
        let game_id = game_state.create_game().await;

        let result = spectate_game(State(game_state.clone()), Path(game_id.clone())).await;

        assert!(result.is_ok());
        let game_session = result.unwrap().0;
        assert_eq!(game_session.id, game_id);
        assert_eq!(game_session.game.current_turn, Color::White);
    }

    #[tokio::test]
    async fn test_spectate_game_with_players() {
        let game_state = GameState::new();
        let game_id = game_state.create_game().await;

        // Add players
        game_state
            .join_game(&game_id, "player1".to_string())
            .await
            .unwrap();
        game_state
            .join_game(&game_id, "player2".to_string())
            .await
            .unwrap();

        // Spectate the game
        let result = spectate_game(State(game_state), Path(game_id.clone())).await;

        assert!(result.is_ok());
        let game_session = result.unwrap().0;
        assert_eq!(game_session.id, game_id);
        assert!(game_session.white_player.is_some());
        assert!(game_session.black_player.is_some());
        assert_eq!(game_session.white_player.unwrap().id, "player1");
        assert_eq!(game_session.black_player.unwrap().id, "player2");
    }
}
