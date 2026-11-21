use loglens_core::{Direction, LogEntry, LogFilters, LogStore, Severity, StoreError};

use crate::Classifier;

/// Queries logs and filters by the final severity, after AI classification.
///
/// The store filters by the severity assigned at ingest, so with a severity filter set this also
/// fetches the complementary severities, classifies them and keeps the ones the AI moved into the
/// requested set. Entries the AI moved out of the set are dropped.
pub async fn query_classified(
    store: &dyn LogStore,
    classifier: &Classifier,
    filters: &LogFilters,
) -> Result<Vec<LogEntry>, StoreError> {
    let wanted = &filters.severity;
    if wanted.is_empty() || Severity::ALL.iter().all(|s| wanted.contains(s)) {
        let mut entries = store.query(filters).await?;
        classifier.enrich(&mut entries).await;
        return Ok(entries);
    }

    let complement = LogFilters {
        severity: Severity::ALL
            .into_iter()
            .filter(|s| !wanted.contains(s))
            .collect(),
        ..filters.clone()
    };
    let (primary, others) = futures::try_join!(store.query(filters), store.query(&complement))?;

    let mut seen = std::collections::HashSet::new();
    let mut entries: Vec<LogEntry> = primary
        .into_iter()
        .chain(others)
        .filter(|e| seen.insert(e.id.clone()))
        .collect();
    classifier.enrich(&mut entries).await;
    entries.retain(|e| wanted.contains(&e.severity));
    match filters.direction {
        Direction::Backward => entries.sort_by_key(|e| std::cmp::Reverse(e.timestamp)),
        Direction::Forward => entries.sort_by_key(|e| e.timestamp),
    }
    entries.truncate(filters.effective_limit() as usize);
    Ok(entries)
}
