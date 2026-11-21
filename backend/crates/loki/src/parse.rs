use std::collections::HashMap;

use chrono::DateTime;
use loglens_core::{LogEntry, Severity, SeveritySource};
use serde::Deserialize;
use sha2::{Digest, Sha256};

const UNKNOWN_HOST: &str = "unknown";
const UNKNOWN_SOURCE: &str = "other";

#[derive(Deserialize)]
struct JsonLine {
    message: String,
    #[serde(default)]
    severity_source: Option<String>,
    #[serde(default)]
    unit: Option<String>,
}

/// Maps one Loki value (stream labels, timestamp in ns, raw line) to a `LogEntry`.
/// Returns `None` when the timestamp is not a valid nanosecond integer.
pub fn parse_entry(labels: &HashMap<String, String>, ts_ns: &str, line: &str) -> Option<LogEntry> {
    let nanos: i64 = ts_ns.trim().parse().ok()?;
    let timestamp = DateTime::from_timestamp_nanos(nanos);

    let (message, severity_source, unit) = match serde_json::from_str::<JsonLine>(line) {
        Ok(parsed) => (
            parsed.message,
            parsed
                .severity_source
                .and_then(|s| s.parse().ok())
                .unwrap_or(SeveritySource::Default),
            parsed.unit.filter(|u| !u.is_empty()),
        ),
        Err(_) => (line.to_string(), SeveritySource::Default, None),
    };

    let severity = labels
        .get("severity")
        .and_then(|s| s.parse::<Severity>().ok())
        .unwrap_or(Severity::Normal);

    Some(LogEntry {
        id: format!("{nanos}-{}", short_hash(line)),
        timestamp,
        host: label_or(labels, "host", UNKNOWN_HOST),
        source: label_or(labels, "source", UNKNOWN_SOURCE),
        container: labels.get("container").filter(|c| !c.is_empty()).cloned(),
        unit,
        message,
        severity,
        severity_source,
        reason: None,
    })
}

fn label_or(labels: &HashMap<String, String>, key: &str, fallback: &str) -> String {
    labels
        .get(key)
        .filter(|v| !v.is_empty())
        .cloned()
        .unwrap_or_else(|| fallback.to_string())
}

fn short_hash(line: &str) -> String {
    let digest = Sha256::digest(line.as_bytes());
    digest[..4].iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn labels(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn parses_json_line() {
        let l = labels(&[
            ("host", "web1"),
            ("source", "docker"),
            ("severity", "important"),
            ("container", "nginx"),
        ]);
        let line = r#"{"message":"Failed password for root","severity_source":"rule","unit":"sshd.service"}"#;
        let e = parse_entry(&l, "1763719200000000000", line).unwrap();
        assert_eq!(e.message, "Failed password for root");
        assert_eq!(e.severity, Severity::Important);
        assert_eq!(e.severity_source, SeveritySource::Rule);
        assert_eq!(e.unit.as_deref(), Some("sshd.service"));
        assert_eq!(e.container.as_deref(), Some("nginx"));
        assert_eq!(e.host, "web1");
        assert_eq!(e.source, "docker");
        assert_eq!(e.timestamp.to_rfc3339(), "2025-11-21T10:00:00+00:00");
        assert!(e.id.starts_with("1763719200000000000-"));
        assert_eq!(e.id.len(), "1763719200000000000-".len() + 8);
        assert_eq!(e.reason, None);
    }

    #[test]
    fn raw_line_falls_back_to_default() {
        let l = labels(&[
            ("host", "db1"),
            ("source", "kernel"),
            ("severity", "warning"),
        ]);
        let e = parse_entry(&l, "1000000001", "plain text line").unwrap();
        assert_eq!(e.message, "plain text line");
        assert_eq!(e.severity_source, SeveritySource::Default);
        assert_eq!(e.severity, Severity::Warning);
        assert_eq!(e.unit, None);
        assert_eq!(e.container, None);
        assert_eq!(e.timestamp.timestamp_subsec_nanos(), 1);
    }

    #[test]
    fn bad_or_missing_severity_is_normal() {
        let bad = labels(&[("severity", "bogus")]);
        let e = parse_entry(&bad, "1", "x").unwrap();
        assert_eq!(e.severity, Severity::Normal);
        assert_eq!(e.host, UNKNOWN_HOST);
        assert_eq!(e.source, UNKNOWN_SOURCE);
        let none = parse_entry(&HashMap::new(), "1", "x").unwrap();
        assert_eq!(none.severity, Severity::Normal);
    }

    #[test]
    fn id_is_stable_and_line_dependent() {
        let l = HashMap::new();
        let a = parse_entry(&l, "5", "same").unwrap();
        let b = parse_entry(&l, "5", "same").unwrap();
        let c = parse_entry(&l, "5", "other").unwrap();
        assert_eq!(a.id, b.id);
        assert_ne!(a.id, c.id);
    }

    #[test]
    fn invalid_timestamp_is_rejected() {
        assert!(parse_entry(&HashMap::new(), "abc", "x").is_none());
    }
}
