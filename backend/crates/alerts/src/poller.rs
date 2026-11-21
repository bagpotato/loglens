use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, TimeDelta, Utc};
use loglens_core::{Direction, LogEntry, LogFilters, LogStore, Severity};
use loglens_severity::template_hash;
use tokio::task::JoinHandle;
use tokio::time::MissedTickBehavior;

use crate::{Alert, Dedup, Notifier};

pub const POLL_LIMIT: u32 = 200;
/// Every poll re-scans this window so lines that reach Loki late (agent batching, clock skew)
/// are still alerted. Entries already handled are skipped by id.
const OVERLAP: TimeDelta = TimeDelta::minutes(5);
const MIN_PERIOD: Duration = Duration::from_secs(1);

/// Periodically fetches recent critical logs and sends one alert per template per dedup window.
pub struct Poller {
    store: Arc<dyn LogStore>,
    notifier: Arc<dyn Notifier>,
    dedup: Dedup,
    started_at: DateTime<Utc>,
    handled: HashMap<String, DateTime<Utc>>,
}

impl Poller {
    pub fn new(store: Arc<dyn LogStore>, notifier: Arc<dyn Notifier>, dedup: Dedup) -> Self {
        Self {
            store,
            notifier,
            dedup,
            started_at: Utc::now(),
            handled: HashMap::new(),
        }
    }

    /// Processes critical entries not handled yet; returns the number of alerts sent.
    pub async fn poll_once(&mut self) -> usize {
        let now = Utc::now();
        let filters = LogFilters {
            severity: vec![Severity::Critical],
            start: Some(self.started_at.max(now - OVERLAP)),
            end: Some(now),
            limit: Some(POLL_LIMIT),
            direction: Direction::Forward,
            ..Default::default()
        };
        let entries = match self.store.query(&filters).await {
            Ok(entries) => entries,
            Err(e) => {
                tracing::warn!(error = %e, "alert poll query failed");
                return 0;
            }
        };

        let mut sent = 0;
        for entry in entries {
            if self.handled.contains_key(&entry.id) {
                continue;
            }
            let hash = template_hash(&entry.message);
            if !self.dedup.is_due(&hash, Utc::now()).await {
                self.handled.insert(entry.id.clone(), entry.timestamp);
                continue;
            }
            match self.notifier.notify(&to_alert(&entry)).await {
                Ok(()) => {
                    self.dedup.record_sent(&hash, Utc::now()).await;
                    self.handled.insert(entry.id.clone(), entry.timestamp);
                    sent += 1;
                }
                Err(e) => tracing::warn!(error = %e, id = %entry.id, "alert notification failed"),
            }
        }
        let horizon = now - OVERLAP - TimeDelta::minutes(1);
        self.handled.retain(|_, ts| *ts >= horizon);
        sent
    }

    pub fn spawn(mut self, every: Duration) -> JoinHandle<()> {
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(every.max(MIN_PERIOD));
            ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);
            loop {
                ticker.tick().await;
                let sent = self.poll_once().await;
                if sent > 0 {
                    tracing::info!(sent, "critical alerts sent");
                }
            }
        })
    }
}

fn to_alert(entry: &LogEntry) -> Alert {
    Alert {
        title: format!("LogLens critical on {}", entry.host),
        message: entry.message.clone(),
        host: entry.host.clone(),
        timestamp: entry.timestamp,
    }
}
