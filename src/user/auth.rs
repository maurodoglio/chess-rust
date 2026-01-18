use super::account::{Claims, User};
use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use std::time::{SystemTime, UNIX_EPOCH};

// JWT secret should be loaded from environment variable in production
// For development, use: export JWT_SECRET="your-secret-key-here"
fn get_jwt_secret() -> String {
    std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "your-secret-key-change-in-production".to_string())
}

const TOKEN_EXPIRATION_HOURS: u64 = 24;

pub fn hash_password(password: &str) -> Result<String, String> {
    hash(password, DEFAULT_COST).map_err(|e| format!("Failed to hash password: {}", e))
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, String> {
    verify(password, hash).map_err(|e| format!("Failed to verify password: {}", e))
}

pub fn generate_token(user: &User) -> Result<String, String> {
    let expiration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + (TOKEN_EXPIRATION_HOURS * 3600);

    let claims = Claims {
        sub: user.id.clone(),
        username: user.username.clone(),
        exp: expiration as usize,
    };

    let secret = get_jwt_secret();
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .map_err(|e| format!("Failed to generate token: {}", e))
}

pub fn verify_token(token: &str) -> Result<Claims, String> {
    let secret = get_jwt_secret();
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|e| format!("Invalid token: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_password() {
        let password = "test_password";
        let hash = hash_password(password).unwrap();
        assert_ne!(hash, password);
        assert!(hash.starts_with("$2"));
    }

    #[test]
    fn test_verify_password_correct() {
        let password = "test_password";
        let hash = hash_password(password).unwrap();
        let result = verify_password(password, &hash).unwrap();
        assert!(result);
    }

    #[test]
    fn test_verify_password_incorrect() {
        let password = "test_password";
        let hash = hash_password(password).unwrap();
        let result = verify_password("wrong_password", &hash).unwrap();
        assert!(!result);
    }

    #[test]
    fn test_generate_and_verify_token() {
        let user = User::new(
            "testuser".to_string(),
            "test@example.com".to_string(),
            "hash".to_string(),
        );

        let token = generate_token(&user).unwrap();
        let claims = verify_token(&token).unwrap();

        assert_eq!(claims.sub, user.id);
        assert_eq!(claims.username, user.username);
    }

    #[test]
    fn test_verify_invalid_token() {
        let result = verify_token("invalid.token.here");
        assert!(result.is_err());
    }
}
