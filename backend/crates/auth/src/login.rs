use std::sync::OnceLock;

use loglens_storage::{users, SqlitePool, DEFAULT_TENANT};
use serde::Serialize;

use crate::{hash_password, verify_password, AuthError, JwtKeys};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub username: String,
    pub role: String,
}

/// Same error for unknown user and wrong password; similar timing for both.
pub async fn login(
    pool: &SqlitePool,
    keys: &JwtKeys,
    username: &str,
    password: &str,
) -> Result<LoginResponse, AuthError> {
    let user = users::find_by_username(pool, DEFAULT_TENANT, username).await?;
    let stored = user.as_ref().map(|u| u.password_hash.clone());
    let password = password.to_string();
    let valid = tokio::task::spawn_blocking(move || match stored {
        Some(hash) => verify_password(&hash, &password),
        None => {
            verify_dummy(&password);
            false
        }
    })
    .await
    .unwrap_or(false);
    let Some(user) = user.filter(|_| valid) else {
        return Err(AuthError::InvalidCredentials);
    };
    Ok(LoginResponse {
        token: keys.issue(&user)?,
        username: user.username,
        role: user.role,
    })
}

fn verify_dummy(password: &str) {
    static DUMMY: OnceLock<Option<String>> = OnceLock::new();
    if let Some(hash) = DUMMY.get_or_init(|| hash_password("loglens-dummy-password").ok()) {
        verify_password(hash, password);
    }
}

#[cfg(test)]
mod tests {
    use loglens_storage::connect_in_memory;

    use super::*;
    use crate::ensure_admin;

    const SECRET: &str = "0123456789abcdef0123456789abcdef";

    #[tokio::test]
    async fn login_success_returns_valid_token() {
        let pool = connect_in_memory().await.unwrap();
        ensure_admin(&pool, "admin", Some("pw")).await.unwrap();
        let keys = JwtKeys::new(SECRET);

        let res = login(&pool, &keys, "admin", "pw").await.unwrap();
        assert_eq!(res.username, "admin");
        assert_eq!(res.role, "admin");
        let claims = keys.verify(&res.token).unwrap();
        assert_eq!(claims.sub, "admin");
        assert_eq!(claims.tid, DEFAULT_TENANT);
    }

    #[tokio::test]
    async fn wrong_password_and_unknown_user_look_the_same() {
        let pool = connect_in_memory().await.unwrap();
        ensure_admin(&pool, "admin", Some("pw")).await.unwrap();
        let keys = JwtKeys::new(SECRET);

        let wrong = login(&pool, &keys, "admin", "nope").await.unwrap_err();
        let unknown = login(&pool, &keys, "ghost", "pw").await.unwrap_err();
        assert!(matches!(wrong, AuthError::InvalidCredentials));
        assert!(matches!(unknown, AuthError::InvalidCredentials));
        assert_eq!(wrong.to_string(), unknown.to_string());
    }
}
