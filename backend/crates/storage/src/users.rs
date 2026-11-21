use sqlx::{FromRow, SqlitePool};

use crate::StorageError;

#[derive(Debug, Clone, PartialEq, Eq, FromRow)]
pub struct User {
    pub id: i64,
    pub tenant_id: i64,
    pub username: String,
    pub password_hash: String,
    pub role: String,
}

pub async fn find_by_username(
    pool: &SqlitePool,
    tenant_id: i64,
    username: &str,
) -> Result<Option<User>, StorageError> {
    let user = sqlx::query_as::<_, User>(
        "SELECT id, tenant_id, username, password_hash, role FROM users \
         WHERE tenant_id = ? AND username = ?",
    )
    .bind(tenant_id)
    .bind(username)
    .fetch_optional(pool)
    .await?;
    Ok(user)
}

/// Fails with `StorageError::Conflict` if the username already exists in the tenant.
pub async fn create(
    pool: &SqlitePool,
    tenant_id: i64,
    username: &str,
    password_hash: &str,
    role: &str,
) -> Result<User, StorageError> {
    sqlx::query_as::<_, User>(
        "INSERT INTO users (tenant_id, username, password_hash, role) VALUES (?, ?, ?, ?) \
         RETURNING id, tenant_id, username, password_hash, role",
    )
    .bind(tenant_id)
    .bind(username)
    .bind(password_hash)
    .bind(role)
    .fetch_one(pool)
    .await
    .map_err(StorageError::from_write)
}

pub async fn count(pool: &SqlitePool, tenant_id: i64) -> Result<i64, StorageError> {
    let (n,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users WHERE tenant_id = ?")
        .bind(tenant_id)
        .fetch_one(pool)
        .await?;
    Ok(n)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{connect_in_memory, DEFAULT_TENANT};

    #[tokio::test]
    async fn create_find_and_count() {
        let pool = connect_in_memory().await.unwrap();
        assert_eq!(count(&pool, DEFAULT_TENANT).await.unwrap(), 0);

        let user = create(&pool, DEFAULT_TENANT, "admin", "hash", "admin")
            .await
            .unwrap();
        assert_eq!(user.username, "admin");
        assert_eq!(user.tenant_id, DEFAULT_TENANT);

        let found = find_by_username(&pool, DEFAULT_TENANT, "admin")
            .await
            .unwrap();
        assert_eq!(found, Some(user));
        assert_eq!(count(&pool, DEFAULT_TENANT).await.unwrap(), 1);
    }

    #[tokio::test]
    async fn missing_user_is_none() {
        let pool = connect_in_memory().await.unwrap();
        let found = find_by_username(&pool, DEFAULT_TENANT, "nobody")
            .await
            .unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn duplicate_username_is_conflict() {
        let pool = connect_in_memory().await.unwrap();
        create(&pool, DEFAULT_TENANT, "bob", "h", "admin")
            .await
            .unwrap();
        let err = create(&pool, DEFAULT_TENANT, "bob", "h2", "admin")
            .await
            .unwrap_err();
        assert!(matches!(err, StorageError::Conflict));
    }

    #[tokio::test]
    async fn tenants_are_isolated() {
        let pool = connect_in_memory().await.unwrap();
        create(&pool, 1, "bob", "h", "admin").await.unwrap();
        create(&pool, 2, "bob", "h", "admin").await.unwrap();
        assert_eq!(count(&pool, 1).await.unwrap(), 1);
        assert!(find_by_username(&pool, 3, "bob").await.unwrap().is_none());
    }
}
