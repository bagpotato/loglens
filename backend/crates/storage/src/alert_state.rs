use chrono::{DateTime, Utc};
use sqlx::SqlitePool;

use crate::StorageError;

pub async fn last_sent(
    pool: &SqlitePool,
    tenant_id: i64,
    template_hash: &str,
) -> Result<Option<DateTime<Utc>>, StorageError> {
    let row: Option<(DateTime<Utc>,)> = sqlx::query_as(
        "SELECT last_sent_at FROM alert_state WHERE tenant_id = ? AND template_hash = ?",
    )
    .bind(tenant_id)
    .bind(template_hash)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|(at,)| at))
}

pub async fn mark_sent(
    pool: &SqlitePool,
    tenant_id: i64,
    template_hash: &str,
    at: DateTime<Utc>,
) -> Result<(), StorageError> {
    sqlx::query(
        "INSERT INTO alert_state (tenant_id, template_hash, last_sent_at) VALUES (?, ?, ?) \
         ON CONFLICT (tenant_id, template_hash) DO UPDATE SET last_sent_at = excluded.last_sent_at",
    )
    .bind(tenant_id)
    .bind(template_hash)
    .bind(at)
    .execute(pool)
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, TimeZone};

    use super::*;
    use crate::{connect_in_memory, DEFAULT_TENANT};

    #[tokio::test]
    async fn unknown_hash_has_no_last_sent() {
        let pool = connect_in_memory().await.unwrap();
        assert!(last_sent(&pool, DEFAULT_TENANT, "x")
            .await
            .unwrap()
            .is_none());
    }

    #[tokio::test]
    async fn mark_sent_roundtrips_and_upserts() {
        let pool = connect_in_memory().await.unwrap();
        let first = Utc.with_ymd_and_hms(2026, 11, 21, 10, 0, 0).unwrap();
        mark_sent(&pool, DEFAULT_TENANT, "h", first).await.unwrap();
        assert_eq!(
            last_sent(&pool, DEFAULT_TENANT, "h").await.unwrap(),
            Some(first)
        );

        let later = first + Duration::milliseconds(90_123);
        mark_sent(&pool, DEFAULT_TENANT, "h", later).await.unwrap();
        assert_eq!(
            last_sent(&pool, DEFAULT_TENANT, "h").await.unwrap(),
            Some(later)
        );
        assert!(last_sent(&pool, 2, "h").await.unwrap().is_none());
    }
}
