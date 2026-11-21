use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;
use password_hash::rand_core::OsRng;

use crate::AuthError;

/// Hashes with Argon2id default parameters; returns a PHC string.
pub fn hash_password(password: &str) -> Result<String, AuthError> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| AuthError::Hash(e.to_string()))
}

/// False for a wrong password or an unparseable hash.
pub fn verify_password(hash: &str, password: &str) -> bool {
    PasswordHash::new(hash)
        .and_then(|parsed| Argon2::default().verify_password(password.as_bytes(), &parsed))
        .is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_and_verify() {
        let hash = hash_password("s3cret").unwrap();
        assert!(hash.starts_with("$argon2id$"));
        assert!(verify_password(&hash, "s3cret"));
        assert!(!verify_password(&hash, "wrong"));
    }

    #[test]
    fn same_password_gets_different_salts() {
        assert_ne!(hash_password("a").unwrap(), hash_password("a").unwrap());
    }

    #[test]
    fn garbage_hash_never_verifies() {
        assert!(!verify_password("not-a-hash", "anything"));
    }
}
