use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::Severity;

pub const DEFAULT_LIMIT: u32 = 500;
pub const MAX_LIMIT: u32 = 5000;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    /// Newest first.
    #[default]
    Backward,
    Forward,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct LogFilters {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// Empty means all severities.
    #[serde(default)]
    pub severity: Vec<Severity>,
    /// Case-insensitive text search in the message.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub q: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(default)]
    pub direction: Direction,
}

impl LogFilters {
    pub fn effective_limit(&self) -> u32 {
        self.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn limit_is_clamped() {
        let f = LogFilters {
            limit: Some(999_999),
            ..Default::default()
        };
        assert_eq!(f.effective_limit(), MAX_LIMIT);
        assert_eq!(LogFilters::default().effective_limit(), DEFAULT_LIMIT);
    }

    #[test]
    fn deserializes_partial_json() {
        let f: LogFilters =
            serde_json::from_str(r#"{"severity":["critical"],"source":"ssh"}"#).unwrap();
        assert_eq!(f.severity, vec![Severity::Critical]);
        assert_eq!(f.source.as_deref(), Some("ssh"));
        assert_eq!(f.direction, Direction::Backward);
    }
}
