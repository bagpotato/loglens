use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::AlertError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Alert {
    pub title: String,
    pub message: String,
    pub host: String,
    pub timestamp: DateTime<Utc>,
}

/// Delivers an alert to an external channel (ntfy in the MVP).
#[async_trait]
pub trait Notifier: Send + Sync {
    async fn notify(&self, alert: &Alert) -> Result<(), AlertError>;
}
