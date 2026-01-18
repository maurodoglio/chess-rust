use super::account::{PublicUser, User};
use super::auth::{generate_token, hash_password, verify_password};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct UserState {
    pub users: Arc<RwLock<HashMap<String, User>>>,
    pub username_to_id: Arc<RwLock<HashMap<String, String>>>,
    pub email_to_id: Arc<RwLock<HashMap<String, String>>>,
}

impl UserState {
    pub fn new() -> Self {
        Self {
            users: Arc::new(RwLock::new(HashMap::new())),
            username_to_id: Arc::new(RwLock::new(HashMap::new())),
            email_to_id: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register_user(
        &self,
        username: String,
        email: String,
        password: String,
    ) -> Result<(String, User), String> {
        // Validate input
        if username.is_empty() || username.len() < 3 {
            return Err("Username must be at least 3 characters long".to_string());
        }

        // Basic email validation: must contain @ with text before and after
        if email.is_empty() || !email.contains('@') {
            return Err("Invalid email address".to_string());
        }

        // More thorough email validation
        let parts: Vec<&str> = email.split('@').collect();
        if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
            return Err("Invalid email address".to_string());
        }

        // Check domain part has at least one dot and proper format
        if !parts[1].contains('.') || parts[1].starts_with('.') || parts[1].ends_with('.') {
            return Err("Invalid email address".to_string());
        }

        if password.is_empty() || password.len() < 6 {
            return Err("Password must be at least 6 characters long".to_string());
        }

        // Check if username or email already exists
        let username_map = self.username_to_id.read().await;
        if username_map.contains_key(&username) {
            return Err("Username already exists".to_string());
        }

        let email_map = self.email_to_id.read().await;
        if email_map.contains_key(&email) {
            return Err("Email already registered".to_string());
        }
        drop(username_map);
        drop(email_map);

        // Hash password
        let password_hash = hash_password(&password)?;

        // Create user
        let user = User::new(username.clone(), email.clone(), password_hash);
        let user_id = user.id.clone();

        // Store user
        let mut users = self.users.write().await;
        let mut username_map = self.username_to_id.write().await;
        let mut email_map = self.email_to_id.write().await;

        users.insert(user_id.clone(), user.clone());
        username_map.insert(username, user_id.clone());
        email_map.insert(email, user_id.clone());

        // Generate token
        let token = generate_token(&user)?;

        Ok((token, user))
    }

    pub async fn login_user(
        &self,
        username: String,
        password: String,
    ) -> Result<(String, User), String> {
        // Find user by username
        let username_map = self.username_to_id.read().await;
        let user_id = username_map
            .get(&username)
            .ok_or("Invalid username or password")?
            .clone();
        drop(username_map);

        let users = self.users.read().await;
        let user = users.get(&user_id).ok_or("User not found")?.clone();
        drop(users);

        // Verify password
        let is_valid = verify_password(&password, &user.password_hash)?;
        if !is_valid {
            return Err("Invalid username or password".to_string());
        }

        // Generate token
        let token = generate_token(&user)?;

        Ok((token, user))
    }

    pub async fn get_user_by_id(&self, user_id: &str) -> Option<User> {
        let users = self.users.read().await;
        users.get(user_id).cloned()
    }

    pub async fn get_public_user(&self, user_id: &str) -> Option<PublicUser> {
        self.get_user_by_id(user_id).await.map(|u| u.to_public())
    }

    pub async fn update_user_stats(&self, user_id: &str, won: bool) -> Result<(), String> {
        let mut users = self.users.write().await;
        let user = users.get_mut(user_id).ok_or("User not found")?;

        user.games_played += 1;
        if won {
            user.games_won += 1;
        }

        Ok(())
    }
}

impl Default for UserState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_register_user_success() {
        let state = UserState::new();
        let result = state
            .register_user(
                "testuser".to_string(),
                "test@example.com".to_string(),
                "password123".to_string(),
            )
            .await;

        assert!(result.is_ok());
        let (token, user) = result.unwrap();
        assert!(!token.is_empty());
        assert_eq!(user.username, "testuser");
        assert_eq!(user.email, "test@example.com");
    }

    #[tokio::test]
    async fn test_register_duplicate_username() {
        let state = UserState::new();
        state
            .register_user(
                "testuser".to_string(),
                "test1@example.com".to_string(),
                "password123".to_string(),
            )
            .await
            .unwrap();

        let result = state
            .register_user(
                "testuser".to_string(),
                "test2@example.com".to_string(),
                "password456".to_string(),
            )
            .await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Username already exists");
    }

    #[tokio::test]
    async fn test_register_duplicate_email() {
        let state = UserState::new();
        state
            .register_user(
                "testuser1".to_string(),
                "test@example.com".to_string(),
                "password123".to_string(),
            )
            .await
            .unwrap();

        let result = state
            .register_user(
                "testuser2".to_string(),
                "test@example.com".to_string(),
                "password456".to_string(),
            )
            .await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Email already registered");
    }

    #[tokio::test]
    async fn test_register_invalid_username() {
        let state = UserState::new();
        let result = state
            .register_user(
                "ab".to_string(),
                "test@example.com".to_string(),
                "password123".to_string(),
            )
            .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("at least 3 characters"));
    }

    #[tokio::test]
    async fn test_register_invalid_password() {
        let state = UserState::new();
        let result = state
            .register_user(
                "testuser".to_string(),
                "test@example.com".to_string(),
                "12345".to_string(),
            )
            .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("at least 6 characters"));
    }

    #[tokio::test]
    async fn test_register_invalid_email_no_domain() {
        let state = UserState::new();
        let result = state
            .register_user(
                "testuser".to_string(),
                "test@".to_string(),
                "password123".to_string(),
            )
            .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid email"));
    }

    #[tokio::test]
    async fn test_register_invalid_email_no_username() {
        let state = UserState::new();
        let result = state
            .register_user(
                "testuser".to_string(),
                "@example.com".to_string(),
                "password123".to_string(),
            )
            .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid email"));
    }

    #[tokio::test]
    async fn test_register_invalid_email_no_dot() {
        let state = UserState::new();
        let result = state
            .register_user(
                "testuser".to_string(),
                "test@examplecom".to_string(),
                "password123".to_string(),
            )
            .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid email"));
    }

    #[tokio::test]
    async fn test_login_user_success() {
        let state = UserState::new();
        state
            .register_user(
                "testuser".to_string(),
                "test@example.com".to_string(),
                "password123".to_string(),
            )
            .await
            .unwrap();

        let result = state
            .login_user("testuser".to_string(), "password123".to_string())
            .await;

        assert!(result.is_ok());
        let (token, user) = result.unwrap();
        assert!(!token.is_empty());
        assert_eq!(user.username, "testuser");
    }

    #[tokio::test]
    async fn test_login_user_wrong_password() {
        let state = UserState::new();
        state
            .register_user(
                "testuser".to_string(),
                "test@example.com".to_string(),
                "password123".to_string(),
            )
            .await
            .unwrap();

        let result = state
            .login_user("testuser".to_string(), "wrongpassword".to_string())
            .await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid username or password");
    }

    #[tokio::test]
    async fn test_login_user_not_found() {
        let state = UserState::new();
        let result = state
            .login_user("nonexistent".to_string(), "password123".to_string())
            .await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid username or password");
    }

    #[tokio::test]
    async fn test_get_user_by_id() {
        let state = UserState::new();
        let (_, user) = state
            .register_user(
                "testuser".to_string(),
                "test@example.com".to_string(),
                "password123".to_string(),
            )
            .await
            .unwrap();

        let result = state.get_user_by_id(&user.id).await;
        assert!(result.is_some());
        assert_eq!(result.unwrap().username, "testuser");
    }

    #[tokio::test]
    async fn test_update_user_stats() {
        let state = UserState::new();
        let (_, user) = state
            .register_user(
                "testuser".to_string(),
                "test@example.com".to_string(),
                "password123".to_string(),
            )
            .await
            .unwrap();

        assert_eq!(user.games_played, 0);
        assert_eq!(user.games_won, 0);

        state.update_user_stats(&user.id, true).await.unwrap();

        let updated = state.get_user_by_id(&user.id).await.unwrap();
        assert_eq!(updated.games_played, 1);
        assert_eq!(updated.games_won, 1);

        state.update_user_stats(&user.id, false).await.unwrap();

        let updated = state.get_user_by_id(&user.id).await.unwrap();
        assert_eq!(updated.games_played, 2);
        assert_eq!(updated.games_won, 1);
    }
}
