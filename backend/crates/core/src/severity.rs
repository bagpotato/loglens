use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::CoreError;

/// Ordered from most to least urgent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Critical,
    Important,
    Warning,
    Normal,
}

impl Severity {
    pub const ALL: [Severity; 4] = [
        Severity::Critical,
        Severity::Important,
        Severity::Warning,
        Severity::Normal,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Severity::Critical => "critical",
            Severity::Important => "important",
            Severity::Warning => "warning",
            Severity::Normal => "normal",
        }
    }
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Severity {
    type Err = CoreError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "critical" => Ok(Severity::Critical),
            "important" => Ok(Severity::Important),
            "warning" => Ok(Severity::Warning),
            "normal" => Ok(Severity::Normal),
            other => Err(CoreError::InvalidSeverity(other.to_string())),
        }
    }
}

/// Who decided the severity of an entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SeveritySource {
    Rule,
    Ai,
    Default,
}

impl SeveritySource {
    pub fn as_str(self) -> &'static str {
        match self {
            SeveritySource::Rule => "rule",
            SeveritySource::Ai => "ai",
            SeveritySource::Default => "default",
        }
    }
}

impl FromStr for SeveritySource {
    type Err = CoreError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "rule" => Ok(SeveritySource::Rule),
            "ai" => Ok(SeveritySource::Ai),
            "default" => Ok(SeveritySource::Default),
            other => Err(CoreError::InvalidSeveritySource(other.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_case_insensitive() {
        assert_eq!("CRITICAL".parse::<Severity>(), Ok(Severity::Critical));
        assert!("bogus".parse::<Severity>().is_err());
    }

    #[test]
    fn orders_most_urgent_first() {
        assert!(Severity::Critical < Severity::Normal);
    }

    #[test]
    fn serializes_lowercase() {
        let json = serde_json::to_string(&SeveritySource::Ai).unwrap();
        assert_eq!(json, "\"ai\"");
    }
}
