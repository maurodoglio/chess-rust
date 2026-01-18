use super::user::User;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct UserStore {
    users: Arc<RwLock<HashMap<String, User>>>,
}

impl UserStore {
    pub fn new() -> Self {
        Self {
            users: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn create_user(&self, username: String, password_hash: String) -> Result<(), String> {
        let mut users = self.users.write().await;

        if users.contains_key(&username) {
            return Err("Username already exists".to_string());
        }

        let user = User::new(username.clone(), password_hash);
        users.insert(username, user);
        Ok(())
    }

    pub async fn get_user(&self, username: &str) -> Option<User> {
        let users = self.users.read().await;
        users.get(username).cloned()
    }

    #[allow(dead_code)]
    pub async fn user_exists(&self, username: &str) -> bool {
        let users = self.users.read().await;
        users.contains_key(username)
    }
}

impl Default for UserStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_user() {
        let store = UserStore::new();
        let result = store
            .create_user("testuser".to_string(), "hash123".to_string())
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_duplicate_username() {
        let store = UserStore::new();
        store
            .create_user("testuser".to_string(), "hash123".to_string())
            .await
            .unwrap();
        let result = store
            .create_user("testuser".to_string(), "hash456".to_string())
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_user() {
        let store = UserStore::new();
        store
            .create_user("testuser".to_string(), "hash123".to_string())
            .await
            .unwrap();
        let user = store.get_user("testuser").await;
        assert!(user.is_some());
        assert_eq!(user.unwrap().username, "testuser");
    }

    #[tokio::test]
    async fn test_user_exists() {
        let store = UserStore::new();
        store
            .create_user("testuser".to_string(), "hash123".to_string())
            .await
            .unwrap();
        assert!(store.user_exists("testuser").await);
        assert!(!store.user_exists("nonexistent").await);
    }
}
