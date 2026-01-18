use bcrypt::{hash, verify, DEFAULT_COST};

// DEFAULT_COST is 12, which provides a good balance between security and performance
pub fn hash_password(password: &str) -> Result<String, String> {
    hash(password, DEFAULT_COST).map_err(|e| format!("Password hashing error: {}", e))
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, String> {
    verify(password, hash).map_err(|e| format!("Password verification error: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hashing() {
        let password = "test_password_123";
        let hash = hash_password(password).unwrap();

        assert_ne!(password, hash);
        assert!(verify_password(password, &hash).unwrap());
    }

    #[test]
    fn test_password_verification_failure() {
        let password = "correct_password";
        let wrong_password = "wrong_password";
        let hash = hash_password(password).unwrap();

        assert!(!verify_password(wrong_password, &hash).unwrap());
    }
}
