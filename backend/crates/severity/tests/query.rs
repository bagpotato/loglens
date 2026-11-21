use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use chrono::{TimeDelta, Utc};
use loglens_ai::{AiError, AiProvider, ChatMessage, TextStream};
use loglens_core::{
    LogEntry, LogFilters, LogStore, LogStream, Meta, Severity, SeveritySource, Stats, StoreError,
};
use loglens_severity::{query_classified, Classifier};
use loglens_storage::connect_in_memory;

/// Rates templates mentioning "disk" as critical and everything else as normal.
struct DiskAi;

#[async_trait]
impl AiProvider for DiskAi {
    async fn complete(&self, messages: &[ChatMessage]) -> Result<String, AiError> {
        let items: Vec<String> = messages[1]
            .content
            .lines()
            .filter_map(|l| l.split_once(": "))
            .map(|(id, t)| {
                let sev = if t.contains("disk") {
                    "critical"
                } else {
                    "normal"
                };
                format!(r#"{{"id":"{id}","severity":"{sev}","reason":"test"}}"#)
            })
            .collect();
        Ok(format!("[{}]", items.join(",")))
    }

    async fn stream(&self, _messages: &[ChatMessage]) -> Result<TextStream, AiError> {
        Err(AiError::Http("unused".into()))
    }
}

struct LabelStore(Vec<LogEntry>);

#[async_trait]
impl LogStore for LabelStore {
    async fn query(&self, f: &LogFilters) -> Result<Vec<LogEntry>, StoreError> {
        Ok(self
            .0
            .iter()
            .filter(|e| f.severity.is_empty() || f.severity.contains(&e.severity))
            .cloned()
            .collect())
    }
    async fn meta(&self) -> Result<Meta, StoreError> {
        Ok(Meta::default())
    }
    async fn stats(&self, _range: Duration) -> Result<Stats, StoreError> {
        Ok(Stats::default())
    }
    async fn tail(&self, _f: &LogFilters) -> Result<LogStream, StoreError> {
        Err(StoreError::Unavailable("unused".into()))
    }
}

fn entry(id: &str, secs_ago: i64, message: &str, sev: Severity, src: SeveritySource) -> LogEntry {
    LogEntry {
        id: id.into(),
        timestamp: Utc::now() - TimeDelta::seconds(secs_ago),
        host: "db1".into(),
        source: "kernel".into(),
        container: None,
        unit: None,
        message: message.into(),
        severity: sev,
        severity_source: src,
        reason: None,
    }
}

async fn classifier() -> Classifier {
    Classifier::new(connect_in_memory().await.unwrap(), Arc::new(DiskAi))
}

#[tokio::test]
async fn filters_by_final_severity_after_ai() {
    let store = LabelStore(vec![
        entry(
            "a",
            1,
            "kernel panic",
            Severity::Critical,
            SeveritySource::Rule,
        ),
        entry(
            "b",
            2,
            "disk sda smart failure",
            Severity::Normal,
            SeveritySource::Default,
        ),
        entry(
            "c",
            3,
            "service started",
            Severity::Critical,
            SeveritySource::Default,
        ),
        entry(
            "d",
            4,
            "user logged in",
            Severity::Normal,
            SeveritySource::Default,
        ),
    ]);
    let filters = LogFilters {
        severity: vec![Severity::Critical],
        ..Default::default()
    };
    let got = query_classified(&store, &classifier().await, &filters)
        .await
        .unwrap();
    let ids: Vec<&str> = got.iter().map(|e| e.id.as_str()).collect();
    assert_eq!(ids, vec!["a", "b"]);
    assert_eq!(got[1].severity_source, SeveritySource::Ai);
}

#[tokio::test]
async fn no_severity_filter_returns_everything_enriched() {
    let store = LabelStore(vec![
        entry(
            "a",
            1,
            "kernel panic",
            Severity::Critical,
            SeveritySource::Rule,
        ),
        entry(
            "b",
            2,
            "disk sda smart failure",
            Severity::Normal,
            SeveritySource::Default,
        ),
    ]);
    let got = query_classified(&store, &classifier().await, &LogFilters::default())
        .await
        .unwrap();
    assert_eq!(got.len(), 2);
    assert_eq!(got[1].severity, Severity::Critical);
}

#[tokio::test]
async fn respects_limit_newest_first() {
    let store = LabelStore(vec![
        entry(
            "old",
            9,
            "disk sda smart failure",
            Severity::Normal,
            SeveritySource::Default,
        ),
        entry(
            "new",
            1,
            "kernel panic",
            Severity::Critical,
            SeveritySource::Rule,
        ),
    ]);
    let filters = LogFilters {
        severity: vec![Severity::Critical],
        limit: Some(1),
        ..Default::default()
    };
    let got = query_classified(&store, &classifier().await, &filters)
        .await
        .unwrap();
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].id, "new");
}
