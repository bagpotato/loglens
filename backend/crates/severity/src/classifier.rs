use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use futures::future::join_all;
use loglens_ai::{AiProvider, Masker};
use loglens_core::{LogEntry, Severity, SeveritySource};
use loglens_storage::{severity_cache, CachedSeverity, SqlitePool, DEFAULT_TENANT};

use crate::batch::{classify_batch, BatchItem, MAX_BATCH};
use crate::template::{hash_template, normalize};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(8);

type Verdicts = HashMap<String, (Severity, String)>;

/// Assigns AI severities to log entries that no ingest rule matched.
pub struct Classifier {
    inner: Arc<Inner>,
    timeout: Duration,
}

struct Inner {
    pool: SqlitePool,
    ai: Arc<dyn AiProvider>,
    in_flight: Mutex<HashSet<String>>,
    mask: bool,
}

impl Inner {
    fn in_flight(&self) -> MutexGuard<'_, HashSet<String>> {
        self.in_flight.lock().unwrap_or_else(|e| e.into_inner())
    }
}

/// Releases claimed template hashes even if the AI task panics or is aborted.
struct Claim {
    inner: Arc<Inner>,
    hashes: Vec<String>,
}

impl Drop for Claim {
    fn drop(&mut self) {
        let mut set = self.inner.in_flight();
        for h in &self.hashes {
            set.remove(h);
        }
    }
}

impl Classifier {
    pub fn new(pool: SqlitePool, ai: Arc<dyn AiProvider>) -> Self {
        Self {
            inner: Arc::new(Inner {
                pool,
                ai,
                in_flight: Mutex::new(HashSet::new()),
                mask: false,
            }),
            timeout: DEFAULT_TIMEOUT,
        }
    }

    /// Masks IPs and usernames in templates before they are sent to the AI.
    /// Call right after `new`, before the classifier is shared.
    pub fn with_masking(mut self, mask: bool) -> Self {
        if let Some(inner) = Arc::get_mut(&mut self.inner) {
            inner.mask = mask;
        }
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Enriches entries in place. Never fails: on any error entries keep
    /// their ingest severity.
    pub async fn enrich(&self, entries: &mut [LogEntry]) {
        self.enrich_waiting(entries, self.timeout).await;
    }

    /// Applies cached verdicts only; misses are classified in the background for next time.
    /// For latency-sensitive paths such as the live tail.
    pub async fn enrich_cached(&self, entries: &mut [LogEntry]) {
        self.enrich_waiting(entries, Duration::ZERO).await;
    }

    async fn enrich_waiting(&self, entries: &mut [LogEntry], wait: Duration) {
        let mut templates: HashMap<String, String> = HashMap::new();
        let hashes: Vec<Option<String>> = entries
            .iter()
            .map(|e| {
                if e.severity_source == SeveritySource::Rule {
                    return None;
                }
                let template = normalize(&e.message);
                let hash = hash_template(&template);
                templates.entry(hash.clone()).or_insert(template);
                Some(hash)
            })
            .collect();
        if templates.is_empty() {
            return;
        }

        let keys: Vec<String> = templates.keys().cloned().collect();
        let mut verdicts: Verdicts =
            match severity_cache::get_many(&self.inner.pool, DEFAULT_TENANT, &keys).await {
                Ok(hits) => hits
                    .into_iter()
                    .map(|(h, c)| (h, (c.severity, c.reason)))
                    .collect(),
                Err(err) => {
                    tracing::warn!(error = %err, "severity cache lookup failed");
                    HashMap::new()
                }
            };

        let misses: Vec<(String, String)> = templates
            .into_iter()
            .filter(|(h, _)| !verdicts.contains_key(h))
            .collect();
        if let Some(claim) = self.claim(misses) {
            verdicts.extend(self.classify_with_timeout(claim, wait).await);
        }

        apply(entries, &hashes, &verdicts);
    }

    fn claim(&self, misses: Vec<(String, String)>) -> Option<(Claim, Vec<(String, String)>)> {
        let mut set = self.inner.in_flight();
        let claimed: Vec<(String, String)> = misses
            .into_iter()
            .filter(|(h, _)| set.insert(h.clone()))
            .collect();
        drop(set);
        if claimed.is_empty() {
            return None;
        }
        let claim = Claim {
            inner: Arc::clone(&self.inner),
            hashes: claimed.iter().map(|(h, _)| h.clone()).collect(),
        };
        Some((claim, claimed))
    }

    async fn classify_with_timeout(
        &self,
        claimed: (Claim, Vec<(String, String)>),
        wait: Duration,
    ) -> Verdicts {
        let inner = Arc::clone(&self.inner);
        let (claim, templates) = claimed;
        let task = tokio::spawn(async move {
            let verdicts = classify_all(&inner, templates).await;
            drop(claim);
            verdicts
        });
        if wait.is_zero() {
            return HashMap::new();
        }
        match tokio::time::timeout(wait, task).await {
            Ok(Ok(verdicts)) => verdicts,
            Ok(Err(err)) => {
                tracing::warn!(error = %err, "severity classification task failed");
                HashMap::new()
            }
            Err(_) => {
                tracing::warn!("severity classification timed out; continuing in background");
                HashMap::new()
            }
        }
    }
}

async fn classify_all(inner: &Inner, templates: Vec<(String, String)>) -> Verdicts {
    let batches = templates.chunks(MAX_BATCH).map(|chunk| async move {
        let mut masker = Masker::new(inner.mask);
        let items: Vec<BatchItem> = chunk
            .iter()
            .enumerate()
            .map(|(i, (_, template))| BatchItem {
                id: (i + 1).to_string(),
                template: masker.mask(template),
            })
            .collect();
        match classify_batch(inner.ai.as_ref(), &items).await {
            Ok(by_id) => chunk
                .iter()
                .zip(&items)
                .filter_map(|((hash, _), item)| {
                    by_id.get(&item.id).map(|v| (hash.clone(), v.clone()))
                })
                .collect(),
            Err(err) => {
                tracing::warn!(error = %err, "severity batch failed");
                Vec::new()
            }
        }
    });
    let verdicts: Verdicts = join_all(batches).await.into_iter().flatten().collect();
    for (hash, (severity, reason)) in &verdicts {
        let cached = CachedSeverity {
            template_hash: hash.clone(),
            severity: *severity,
            reason: reason.clone(),
        };
        if let Err(err) = severity_cache::put(&inner.pool, DEFAULT_TENANT, &cached).await {
            tracing::warn!(error = %err, "severity cache write failed");
        }
    }
    verdicts
}

fn apply(entries: &mut [LogEntry], hashes: &[Option<String>], verdicts: &Verdicts) {
    for (entry, hash) in entries.iter_mut().zip(hashes) {
        let Some((severity, reason)) = hash.as_ref().and_then(|h| verdicts.get(h)) else {
            continue;
        };
        entry.severity = *severity;
        entry.severity_source = SeveritySource::Ai;
        entry.reason = (!reason.is_empty()).then(|| reason.clone());
    }
}
