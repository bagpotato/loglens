use std::str::FromStr;

use sqlx::migrate::Migrator;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use sqlx::SqlitePool;

use crate::StorageError;

pub const DEFAULT_TENANT: i64 = 1;

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

/// Opens (creating if missing) a SQLite database in WAL mode and runs migrations.
pub async fn connect(db_url: &str) -> Result<SqlitePool, StorageError> {
    let options = SqliteConnectOptions::from_str(db_url)?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .foreign_keys(true);
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;
    MIGRATOR.run(&pool).await?;
    Ok(pool)
}

/// Single-connection in-memory database, migrated. Used by tests.
pub async fn connect_in_memory() -> Result<SqlitePool, StorageError> {
    let options = SqliteConnectOptions::from_str("sqlite::memory:")?.foreign_keys(true);
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .min_connections(1)
        .idle_timeout(None)
        .max_lifetime(None)
        .connect_with(options)
        .await?;
    MIGRATOR.run(&pool).await?;
    Ok(pool)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn file_database_is_created_and_migrated() {
        let dir = std::env::temp_dir().join(format!("loglens-storage-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.db");
        let _ = std::fs::remove_file(&path);
        let url = format!("sqlite://{}", path.display());
        let pool = connect(&url).await.unwrap();
        let (mode,): (String,) = sqlx::query_as("PRAGMA journal_mode")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(mode.to_lowercase(), "wal");
        pool.close().await;
        assert!(path.exists());
        let again = connect(&url).await.unwrap();
        again.close().await;
        let _ = std::fs::remove_dir_all(&dir);
    }
}
