use std::time::Duration;

use chrono::{DateTime, TimeDelta, Utc};
use loglens_storage::{alert_state, SqlitePool, DEFAULT_TENANT};

pub const DEFAULT_WINDOW: Duration = Duration::from_secs(10 * 60);

/// Suppresses repeated alerts for the same message template within a time window.
#[derive(Clone)]
pub struct Dedup {
    pool: SqlitePool,
    window: TimeDelta,
}

impl Dedup {
    pub fn new(pool: SqlitePool, window: Duration) -> Self {
        Self {
            pool,
            window: TimeDelta::from_std(window).unwrap_or(TimeDelta::MAX),
        }
    }

    /// True when no alert was sent for this template within the window; then records `now`.
    pub async fn should_send(&self, template_hash: &str, now: DateTime<Utc>) -> bool {
        if !self.is_due(template_hash, now).await {
            return false;
        }
        self.record_sent(template_hash, now).await;
        true
    }

    /// True when no alert was sent for this template within the window. Does not record.
    /// Storage failures are logged and fail open so critical alerts are not lost.
    pub async fn is_due(&self, template_hash: &str, now: DateTime<Utc>) -> bool {
        match alert_state::last_sent(&self.pool, DEFAULT_TENANT, template_hash).await {
            Ok(Some(last)) => now.signed_duration_since(last) >= self.window,
            Ok(None) => true,
            Err(e) => {
                tracing::warn!(error = %e, "alert dedup lookup failed");
                true
            }
        }
    }

    /// Records a delivered alert for this template.
    pub async fn record_sent(&self, template_hash: &str, now: DateTime<Utc>) {
        if let Err(e) = alert_state::mark_sent(&self.pool, DEFAULT_TENANT, template_hash, now).await
        {
            tracing::warn!(error = %e, "alert dedup mark failed");
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use loglens_storage::connect_in_memory;

    use super::*;

    async fn dedup() -> Dedup {
        Dedup::new(connect_in_memory().await.unwrap(), DEFAULT_WINDOW)
    }

    #[tokio::test]
    async fn second_within_window_is_suppressed() {
        let d = dedup().await;
        let t0 = Utc.with_ymd_and_hms(2026, 11, 21, 10, 0, 0).unwrap();
        assert!(d.should_send("abc", t0).await);
        assert!(!d.should_send("abc", t0 + TimeDelta::minutes(9)).await);
    }

    #[tokio::test]
    async fn sends_again_after_window() {
        let d = dedup().await;
        let t0 = Utc.with_ymd_and_hms(2026, 11, 21, 10, 0, 0).unwrap();
        assert!(d.should_send("abc", t0).await);
        assert!(d.should_send("abc", t0 + TimeDelta::minutes(10)).await);
        assert!(!d.should_send("abc", t0 + TimeDelta::minutes(11)).await);
    }

    #[tokio::test]
    async fn templates_are_independent() {
        let d = dedup().await;
        let t0 = Utc.with_ymd_and_hms(2026, 11, 21, 10, 0, 0).unwrap();
        assert!(d.should_send("abc", t0).await);
        assert!(d.should_send("def", t0).await);
    }
}
