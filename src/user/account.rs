use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub games_played: u32,
    pub games_won: u32,
}

impl User {
    pub fn new(username: String, email: String, password_hash: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            username,
            email,
            password_hash,
            created_at: Utc::now(),
            games_played: 0,
            games_won: 0,
        }
    }

    /// Create a public version of the user without sensitive data
    pub fn to_public(&self) -> PublicUser {
        PublicUser {
            id: self.id.clone(),
            username: self.username.clone(),
            created_at: self.created_at,
            games_played: self.games_played,
            games_won: self.games_won,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicUser {
    pub id: String,
    pub username: String,
    pub created_at: DateTime<Utc>,
    pub games_played: u32,
    pub games_won: u32,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: PublicUser,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user id
    pub username: String,
    pub exp: usize, // expiration time
}
