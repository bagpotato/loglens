use loglens_storage::{users, SqlitePool, StorageError, DEFAULT_TENANT};

use crate::{hash_password, AuthError};

const ADMIN_ROLE: &str = "admin";

/// Creates the first admin of the default tenant when it has no users.
/// Returns whether a user was created.
pub async fn ensure_admin(
    pool: &SqlitePool,
    username: &str,
    password: Option<&str>,
) -> Result<bool, AuthError> {
    if users::count(pool, DEFAULT_TENANT).await? > 0 {
        return Ok(false);
    }
    let password = password.ok_or(AuthError::MissingAdminPassword)?;
    let hash = hash_password(password)?;
    match users::create(pool, DEFAULT_TENANT, username, &hash, ADMIN_ROLE).await {
        Ok(_) => {
            tracing::info!(username, "created initial admin user");
            Ok(true)
        }
        Err(StorageError::Conflict) => Ok(false),
        Err(e) => Err(e.into()),
    }
}

#[cfg(test)]
mod tests {
    use loglens_storage::connect_in_memory;

    use super::*;
    use crate::verify_password;

    #[tokio::test]
    async fn creates_admin_once() {
        let pool = connect_in_memory().await.unwrap();
        assert!(ensure_admin(&pool, "admin", Some("pw")).await.unwrap());
        assert!(!ensure_admin(&pool, "admin", Some("other")).await.unwrap());
        assert!(!ensure_admin(&pool, "admin", None).await.unwrap());

        assert_eq!(users::count(&pool, DEFAULT_TENANT).await.unwrap(), 1);
        let user = users::find_by_username(&pool, DEFAULT_TENANT, "admin")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(user.role, "admin");
        assert!(verify_password(&user.password_hash, "pw"));
    }

    #[tokio::test]
    async fn requires_password_on_empty_database() {
        let pool = connect_in_memory().await.unwrap();
        let err = ensure_admin(&pool, "admin", None).await.unwrap_err();
        assert!(matches!(err, AuthError::MissingAdminPassword));
    }
}
