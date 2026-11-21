use chrono::{DateTime, SecondsFormat, Utc};
use serde::{Deserialize, Serialize};

use crate::{Severity, SeveritySource};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LogEntry {
    /// Stable id: Loki timestamp in nanoseconds, a dash, and 8 hex chars of the line hash.
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub host: String,
    pub source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub container: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    pub message: String,
    pub severity: Severity,
    pub severity_source: SeveritySource,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl LogEntry {
    /// One-line representation used in AI prompts and alerts.
    pub fn to_prompt_line(&self) -> String {
        format!(
            "[{}] host={} source={} severity={}: {}",
            self.timestamp.to_rfc3339_opts(SecondsFormat::Secs, true),
            self.host,
            self.source,
            self.severity,
            self.message
        )
    }
}
