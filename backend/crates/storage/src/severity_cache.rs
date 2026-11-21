use std::collections::HashMap;
use std::str::FromStr;

use loglens_core::Severity;
use sqlx::{QueryBuilder, Row, Sqlite, SqlitePool};

use crate::StorageError;

const CHUNK: usize = 500;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CachedSeverity {
    pub template_hash: String,
    pub severity: Severity,
    pub reason: String,
}

/// Returns cached entries keyed by template hash; unknown hashes are absent.
pub async fn get_many(
    pool: &SqlitePool,
    tenant_id: i64,
    hashes: &[String],
) -> Result<HashMap<String, CachedSeverity>, StorageError> {
    let mut found = HashMap::new();
    for chunk in hashes.chunks(CHUNK) {
        let mut qb: QueryBuilder<Sqlite> = QueryBuilder::new(
            "SELECT template_hash, severity, reason FROM severity_cache WHERE tenant_id = ",
        );
        qb.push_bind(tenant_id).push(" AND template_hash IN (");
        let mut list = qb.separated(", ");
        for hash in chunk {
            list.push_bind(hash);
        }
        list.push_unseparated(")");
        for row in qb.build().fetch_all(pool).await? {
            let template_hash: String = row.try_get("template_hash")?;
            let raw: String = row.try_get("severity")?;
            let severity =
                Severity::from_str(&raw).map_err(|e| StorageError::InvalidData(e.to_string()))?;
            let reason: String = row.try_get("reason")?;
            found.insert(
                template_hash.clone(),
                CachedSeverity {
                    template_hash,
                    severity,
                    reason,
                },
            );
        }
    }
    Ok(found)
}

pub async fn put(
    pool: &SqlitePool,
    tenant_id: i64,
    entry: &CachedSeverity,
) -> Result<(), StorageError> {
    sqlx::query(
        "INSERT INTO severity_cache (tenant_id, template_hash, severity, reason) \
         VALUES (?, ?, ?, ?) \
         ON CONFLICT (tenant_id, template_hash) DO UPDATE SET \
         severity = excluded.severity, reason = excluded.reason, \
         created_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')",
    )
    .bind(tenant_id)
    .bind(&entry.template_hash)
    .bind(entry.severity.as_str())
    .bind(&entry.reason)
    .execute(pool)
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{connect_in_memory, DEFAULT_TENANT};

    fn entry(hash: &str, severity: Severity) -> CachedSeverity {
        CachedSeverity {
            template_hash: hash.into(),
            severity,
            reason: format!("reason {hash}"),
        }
    }

    #[tokio::test]
    async fn empty_input_returns_empty_map() {
        let pool = connect_in_memory().await.unwrap();
        assert!(get_many(&pool, DEFAULT_TENANT, &[])
            .await
            .unwrap()
            .is_empty());
    }

    #[tokio::test]
    async fn put_then_get_many() {
        let pool = connect_in_memory().await.unwrap();
        put(&pool, DEFAULT_TENANT, &entry("a", Severity::Critical))
            .await
            .unwrap();
        put(&pool, DEFAULT_TENANT, &entry("b", Severity::Normal))
            .await
            .unwrap();
        let keys = vec!["a".to_string(), "b".to_string(), "zzz".to_string()];
        let map = get_many(&pool, DEFAULT_TENANT, &keys).await.unwrap();
        assert_eq!(map.len(), 2);
        assert_eq!(map["a"], entry("a", Severity::Critical));
        assert_eq!(map["b"].severity, Severity::Normal);
    }

    #[tokio::test]
    async fn put_upserts() {
        let pool = connect_in_memory().await.unwrap();
        put(&pool, DEFAULT_TENANT, &entry("a", Severity::Warning))
            .await
            .unwrap();
        put(&pool, DEFAULT_TENANT, &entry("a", Severity::Important))
            .await
            .unwrap();
        let map = get_many(&pool, DEFAULT_TENANT, &["a".to_string()])
            .await
            .unwrap();
        assert_eq!(map["a"].severity, Severity::Important);
    }

    #[tokio::test]
    async fn tenants_are_isolated_and_large_batches_work() {
        let pool = connect_in_memory().await.unwrap();
        put(&pool, 2, &entry("a", Severity::Warning)).await.unwrap();
        let mut keys: Vec<String> = (0..1200).map(|i| format!("k{i}")).collect();
        keys.push("a".into());
        assert!(get_many(&pool, DEFAULT_TENANT, &keys)
            .await
            .unwrap()
            .is_empty());
        assert_eq!(get_many(&pool, 2, &keys).await.unwrap().len(), 1);
    }
}
