use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, SubsecRound, Utc};
use futures::stream::{self, StreamExt};
use loglens_ai::{AiError, AiProvider, ChatMessage, TextStream};
use loglens_core::{
    Direction, LogEntry, LogFilters, LogStore, LogStream, Meta, Severity, SeveritySource, Stats,
    StoreError,
};

pub type StreamReply = Box<dyn Fn(&[ChatMessage]) -> Vec<Result<String, AiError>> + Send + Sync>;

pub struct FakeProvider {
    pub complete_reply: Result<String, String>,
    pub stream_reply: StreamReply,
    pub completes: Mutex<Vec<Vec<ChatMessage>>>,
    pub streams: Mutex<Vec<Vec<ChatMessage>>>,
}

impl FakeProvider {
    pub fn new(complete_reply: &str) -> Self {
        Self {
            complete_reply: Ok(complete_reply.to_string()),
            stream_reply: Box::new(|_| Vec::new()),
            completes: Mutex::new(Vec::new()),
            streams: Mutex::new(Vec::new()),
        }
    }

    pub fn failing() -> Self {
        Self {
            complete_reply: Err("connection refused".to_string()),
            ..Self::new("")
        }
    }

    pub fn with_stream(mut self, reply: StreamReply) -> Self {
        self.stream_reply = reply;
        self
    }

    pub fn last_complete(&self) -> Vec<ChatMessage> {
        self.completes.lock().unwrap().last().cloned().unwrap()
    }

    pub fn last_stream(&self) -> Vec<ChatMessage> {
        self.streams.lock().unwrap().last().cloned().unwrap()
    }
}

#[async_trait]
impl AiProvider for FakeProvider {
    async fn complete(&self, messages: &[ChatMessage]) -> Result<String, AiError> {
        self.completes.lock().unwrap().push(messages.to_vec());
        self.complete_reply.clone().map_err(AiError::Http)
    }

    async fn stream(&self, messages: &[ChatMessage]) -> Result<TextStream, AiError> {
        self.streams.lock().unwrap().push(messages.to_vec());
        let chunks = (self.stream_reply)(messages);
        Ok(stream::iter(chunks).boxed())
    }
}

pub struct FakeStore {
    pub entries: Vec<LogEntry>,
    pub queries: Mutex<Vec<LogFilters>>,
}

impl FakeStore {
    pub fn new(entries: Vec<LogEntry>) -> Arc<Self> {
        Arc::new(Self {
            entries,
            queries: Mutex::new(Vec::new()),
        })
    }

    fn matches(filters: &LogFilters, e: &LogEntry) -> bool {
        filters.host.as_ref().is_none_or(|h| *h == e.host)
            && filters.source.as_ref().is_none_or(|s| *s == e.source)
            && (filters.severity.is_empty() || filters.severity.contains(&e.severity))
            && filters
                .q
                .as_ref()
                .is_none_or(|q| e.message.to_lowercase().contains(&q.to_lowercase()))
            && filters.start.is_none_or(|s| e.timestamp >= s)
            && filters.end.is_none_or(|end| e.timestamp <= end)
    }
}

#[async_trait]
impl LogStore for FakeStore {
    async fn query(&self, filters: &LogFilters) -> Result<Vec<LogEntry>, StoreError> {
        self.queries.lock().unwrap().push(filters.clone());
        let mut found: Vec<LogEntry> = self
            .entries
            .iter()
            .filter(|e| Self::matches(filters, e))
            .cloned()
            .collect();
        found.sort_by_key(|e| e.timestamp);
        if filters.direction == Direction::Backward {
            found.reverse();
        }
        found.truncate(filters.effective_limit() as usize);
        Ok(found)
    }

    async fn meta(&self) -> Result<Meta, StoreError> {
        let mut hosts: Vec<String> = self.entries.iter().map(|e| e.host.clone()).collect();
        let mut sources: Vec<String> = self.entries.iter().map(|e| e.source.clone()).collect();
        hosts.sort();
        hosts.dedup();
        sources.sort();
        sources.dedup();
        Ok(Meta { hosts, sources })
    }

    async fn stats(&self, _range: Duration) -> Result<Stats, StoreError> {
        Err(StoreError::Unavailable("not used".to_string()))
    }

    async fn tail(&self, _filters: &LogFilters) -> Result<LogStream, StoreError> {
        Err(StoreError::Unavailable("not used".to_string()))
    }
}

pub fn now() -> DateTime<Utc> {
    Utc::now().trunc_subsecs(0)
}

pub fn entry(id: &str, ts: DateTime<Utc>, host: &str, source: &str, message: &str) -> LogEntry {
    LogEntry {
        id: id.to_string(),
        timestamp: ts,
        host: host.to_string(),
        source: source.to_string(),
        container: None,
        unit: None,
        message: message.to_string(),
        severity: Severity::Normal,
        severity_source: SeveritySource::Default,
        reason: None,
    }
}

/// SSH brute force from one IP on web1, mixed with normal nginx and cron traffic.
pub fn demo_entries(now: DateTime<Utc>) -> Vec<LogEntry> {
    let mut entries = Vec::new();
    let attack_start = now - chrono::Duration::minutes(90);
    for i in 0..8 {
        let mut e = entry(
            &format!("ssh-{i}"),
            attack_start + chrono::Duration::seconds(i * 3),
            "web1",
            "ssh",
            &format!(
                "Failed password for invalid user admin{i} from 203.0.113.7 port 4{i}022 ssh2"
            ),
        );
        e.severity = Severity::Important;
        e.severity_source = SeveritySource::Rule;
        entries.push(e);
    }
    entries.push(entry(
        "ssh-ok",
        now - chrono::Duration::minutes(30),
        "web1",
        "ssh",
        "Accepted publickey for deploy from 198.51.100.4 port 50122 ssh2",
    ));
    for i in 0..5 {
        entries.push(entry(
            &format!("nginx-{i}"),
            now - chrono::Duration::minutes(10 * i),
            "web1",
            "nginx",
            "\"GET /health HTTP/1.1\" 200 2",
        ));
    }
    entries.push(entry(
        "cron-1",
        now - chrono::Duration::minutes(45),
        "db1",
        "cron",
        "(root) CMD (run-parts /etc/cron.hourly)",
    ));
    entries
}
