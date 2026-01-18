use super::{
    jwt::create_jwt,
    password::{hash_password, verify_password},
    store::UserStore,
    user::{AuthResponse, LoginRequest, RegisterRequest},
};
use axum::{extract::State, http::StatusCode, response::Json};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

pub async fn register(
    State(user_store): State<UserStore>,
    Json(request): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Validate username length
    if request.username.is_empty() || request.username.len() < 3 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Username must be at least 3 characters long".to_string(),
            }),
        ));
    }

    // Validate password length
    if request.password.len() < 6 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Password must be at least 6 characters long".to_string(),
            }),
        ));
    }

    // Hash the password
    let password_hash = hash_password(&request.password).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;

    // Create the user
    user_store
        .create_user(request.username.clone(), password_hash)
        .await
        .map_err(|e| (StatusCode::CONFLICT, Json(ErrorResponse { error: e })))?;

    // Generate JWT token
    let token = create_jwt(&request.username).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;

    Ok(Json(AuthResponse {
        token,
        username: request.username,
    }))
}

pub async fn login(
    State(user_store): State<UserStore>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Get the user
    let user = user_store
        .get_user(&request.username)
        .await
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "Invalid username or password".to_string(),
                }),
            )
        })?;

    // Verify password
    let is_valid = verify_password(&request.password, &user.password_hash).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;

    if !is_valid {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Invalid username or password".to_string(),
            }),
        ));
    }

    // Generate JWT token
    let token = create_jwt(&request.username).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;

    Ok(Json(AuthResponse {
        token,
        username: request.username,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::extract::State;

    #[tokio::test]
    async fn test_register_success() {
        let user_store = UserStore::new();
        let request = RegisterRequest {
            username: "testuser".to_string(),
            password: "password123".to_string(),
        };

        let result = register(State(user_store), Json(request)).await;
        assert!(result.is_ok());
        let response = result.unwrap().0;
        assert_eq!(response.username, "testuser");
        assert!(!response.token.is_empty());
    }

    #[tokio::test]
    async fn test_register_short_username() {
        let user_store = UserStore::new();
        let request = RegisterRequest {
            username: "ab".to_string(),
            password: "password123".to_string(),
        };

        let result = register(State(user_store), Json(request)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_register_short_password() {
        let user_store = UserStore::new();
        let request = RegisterRequest {
            username: "testuser".to_string(),
            password: "pass".to_string(),
        };

        let result = register(State(user_store), Json(request)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_register_duplicate_username() {
        let user_store = UserStore::new();
        let request1 = RegisterRequest {
            username: "testuser".to_string(),
            password: "password123".to_string(),
        };
        let request2 = RegisterRequest {
            username: "testuser".to_string(),
            password: "password456".to_string(),
        };

        let _ = register(State(user_store.clone()), Json(request1))
            .await
            .unwrap();
        let result = register(State(user_store), Json(request2)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_login_success() {
        let user_store = UserStore::new();
        let register_req = RegisterRequest {
            username: "testuser".to_string(),
            password: "password123".to_string(),
        };
        let _ = register(State(user_store.clone()), Json(register_req))
            .await
            .unwrap();

        let login_req = LoginRequest {
            username: "testuser".to_string(),
            password: "password123".to_string(),
        };
        let result = login(State(user_store), Json(login_req)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_login_wrong_password() {
        let user_store = UserStore::new();
        let register_req = RegisterRequest {
            username: "testuser".to_string(),
            password: "password123".to_string(),
        };
        let _ = register(State(user_store.clone()), Json(register_req))
            .await
            .unwrap();

        let login_req = LoginRequest {
            username: "testuser".to_string(),
            password: "wrongpassword".to_string(),
        };
        let result = login(State(user_store), Json(login_req)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_login_nonexistent_user() {
        let user_store = UserStore::new();
        let login_req = LoginRequest {
            username: "nonexistent".to_string(),
            password: "password123".to_string(),
        };
        let result = login(State(user_store), Json(login_req)).await;
        assert!(result.is_err());
    }
}
