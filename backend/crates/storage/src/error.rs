use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),
    #[error("record already exists")]
    Conflict,
    #[error("invalid stored value: {0}")]
    InvalidData(String),
}

impl StorageError {
    pub(crate) fn from_write(err: sqlx::Error) -> Self {
        let unique = err
            .as_database_error()
            .is_some_and(|db| db.is_unique_violation());
        if unique {
            StorageError::Conflict
        } else {
            StorageError::Database(err)
        }
    }
}
