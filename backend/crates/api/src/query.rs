use std::time::Duration;

use chrono::{DateTime, Utc};
use loglens_core::{LogFilters, Severity};
use serde::Deserialize;

use crate::ApiError;

pub const DEFAULT_RANGE: Duration = Duration::from_secs(24 * 3600);
pub const MAX_RANGE: Duration = Duration::from_secs(30 * 24 * 3600);

/// Raw query string of `/api/logs` and `/api/logs/tail`.
#[derive(Debug, Default, Deserialize)]
pub struct LogQuery {
    pub host: Option<String>,
    pub source: Option<String>,
    pub severity: Option<String>,
    pub q: Option<String>,
    pub start: Option<String>,
    pub end: Option<String>,
    pub limit: Option<String>,
}

impl LogQuery {
    pub fn into_filters(self) -> Result<LogFilters, ApiError> {
        let start = non_empty(self.start)
            .map(|v| parse_time("start", &v))
            .transpose()?;
        let end = non_empty(self.end)
            .map(|v| parse_time("end", &v))
            .transpose()?;
        if let (Some(s), Some(e)) = (start, end) {
            if s > e {
                return Err(ApiError::bad_request("start must be before end"));
            }
        }
        let limit = non_empty(self.limit)
            .map(|v| {
                v.parse::<u32>()
                    .map_err(|_| ApiError::bad_request(format!("invalid limit '{v}'")))
            })
            .transpose()?;
        Ok(LogFilters {
            host: non_empty(self.host),
            source: non_empty(self.source),
            severity: parse_severities(self.severity.as_deref().unwrap_or(""))?,
            q: non_empty(self.q),
            start,
            end,
            limit,
            ..Default::default()
        })
    }
}

#[derive(Debug, Default, Deserialize)]
pub struct StatsQuery {
    pub range: Option<String>,
}

/// Parses `<n>m|h|d` (default 24h, max 30d).
pub fn parse_range(raw: Option<&str>) -> Result<Duration, ApiError> {
    let raw = match raw.map(str::trim).filter(|v| !v.is_empty()) {
        Some(v) => v,
        None => return Ok(DEFAULT_RANGE),
    };
    let invalid = || ApiError::bad_request(format!("invalid range '{raw}', expected <n>m|h|d"));
    let split = raw.len() - raw.chars().last().map_or(0, char::len_utf8);
    let (number, unit) = raw.split_at(split);
    let n: u64 = number.parse().map_err(|_| invalid())?;
    let unit_secs = match unit {
        "m" => 60,
        "h" => 3600,
        "d" => 86_400,
        _ => return Err(invalid()),
    };
    if n == 0 {
        return Err(invalid());
    }
    let range = n
        .checked_mul(unit_secs)
        .map(Duration::from_secs)
        .filter(|d| *d <= MAX_RANGE)
        .ok_or_else(|| ApiError::bad_request("range must not exceed 30d"))?;
    Ok(range)
}

fn non_empty(value: Option<String>) -> Option<String> {
    value
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

fn parse_time(name: &str, value: &str) -> Result<DateTime<Utc>, ApiError> {
    DateTime::parse_from_rfc3339(value)
        .map(|t| t.with_timezone(&Utc))
        .map_err(|_| ApiError::bad_request(format!("invalid {name} '{value}', expected RFC3339")))
}

fn parse_severities(raw: &str) -> Result<Vec<Severity>, ApiError> {
    let mut out = Vec::new();
    for part in raw.split(',').map(str::trim).filter(|p| !p.is_empty()) {
        let severity: Severity = part
            .parse()
            .map_err(|_| ApiError::bad_request(format!("invalid severity '{part}'")))?;
        if !out.contains(&severity) {
            out.push(severity);
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn query(pairs: &[(&str, &str)]) -> LogQuery {
        let get = |k: &str| {
            pairs
                .iter()
                .find(|(key, _)| *key == k)
                .map(|(_, v)| v.to_string())
        };
        LogQuery {
            host: get("host"),
            source: get("source"),
            severity: get("severity"),
            q: get("q"),
            start: get("start"),
            end: get("end"),
            limit: get("limit"),
        }
    }

    #[test]
    fn parses_full_query() {
        let f = query(&[
            ("host", "web1"),
            ("source", ""),
            ("severity", "critical, important,critical"),
            ("start", "2026-11-21T10:00:00Z"),
            ("end", "2026-11-21T12:00:00+01:00"),
            ("limit", "50"),
        ])
        .into_filters()
        .unwrap();
        assert_eq!(f.host.as_deref(), Some("web1"));
        assert_eq!(f.source, None);
        assert_eq!(f.severity, vec![Severity::Critical, Severity::Important]);
        assert_eq!(f.limit, Some(50));
        assert_eq!(f.end.unwrap().to_rfc3339(), "2026-11-21T11:00:00+00:00");
    }

    #[test]
    fn rejects_invalid_values() {
        for pairs in [
            [("severity", "critical,bogus")],
            [("start", "yesterday")],
            [("limit", "-1")],
        ] {
            assert!(query(&pairs).into_filters().is_err());
        }
        let reversed = query(&[
            ("start", "2026-11-21T12:00:00Z"),
            ("end", "2026-11-21T10:00:00Z"),
        ]);
        assert!(reversed.into_filters().is_err());
    }

    #[test]
    fn parses_ranges() {
        assert_eq!(parse_range(None).unwrap(), DEFAULT_RANGE);
        assert_eq!(parse_range(Some("15m")).unwrap(), Duration::from_secs(900));
        assert_eq!(
            parse_range(Some("6h")).unwrap(),
            Duration::from_secs(21_600)
        );
        assert_eq!(parse_range(Some("30d")).unwrap(), MAX_RANGE);
        for bad in [
            "31d",
            "0h",
            "h",
            "10",
            "5w",
            "-1h",
            "1.5h",
            "9999999999999999999d",
        ] {
            assert!(parse_range(Some(bad)).is_err(), "{bad}");
        }
    }
}
