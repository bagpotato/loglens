use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use chrono::Utc;
use loglens_ai::{AiError, AiProvider, ChatMessage, TextStream};
use loglens_core::{LogEntry, Severity, SeveritySource};
use loglens_severity::{template_hash, Classifier};
use loglens_storage::{connect_in_memory, severity_cache, SqlitePool, DEFAULT_TENANT};

#[derive(Clone, Copy)]
enum Mode {
    Valid,
    InvalidJson,
}

struct FakeAi {
    mode: Mode,
    delay: Duration,
    calls: Mutex<Vec<Vec<String>>>,
}

impl FakeAi {
    fn new(mode: Mode, delay: Duration) -> Arc<Self> {
        Arc::new(Self {
            mode,
            delay,
            calls: Mutex::new(Vec::new()),
        })
    }

    fn calls(&self) -> Vec<Vec<String>> {
        self.calls.lock().unwrap().clone()
    }
}

#[async_trait]
impl AiProvider for FakeAi {
    async fn complete(&self, messages: &[ChatMessage]) -> Result<String, AiError> {
        let lines: Vec<String> = messages[1].content.lines().map(String::from).collect();
        self.calls.lock().unwrap().push(lines.clone());
        tokio::time::sleep(self.delay).await;
        match self.mode {
            Mode::InvalidJson => Ok("Sorry, I can't classify that.".into()),
            Mode::Valid => {
                let items: Vec<String> = lines
                    .iter()
                    .filter_map(|l| l.split_once(": "))
                    .map(|(id, _)| {
                        format!(r#"{{"id":"{id}","severity":"warning","reason":"looks odd"}}"#)
                    })
                    .collect();
                Ok(format!("```json\n[{}]\n```", items.join(",")))
            }
        }
    }

    async fn stream(&self, _messages: &[ChatMessage]) -> Result<TextStream, AiError> {
        Err(AiError::Http("not used".into()))
    }
}

fn entry(message: &str, source: SeveritySource) -> LogEntry {
    LogEntry {
        id: format!("1-{}", &template_hash(message)[..8]),
        timestamp: Utc::now(),
        host: "web1".into(),
        source: "ssh".into(),
        container: None,
        unit: None,
        message: message.into(),
        severity: Severity::Normal,
        severity_source: source,
        reason: None,
    }
}

async fn setup(mode: Mode, delay: Duration) -> (SqlitePool, Arc<FakeAi>, Classifier) {
    let pool = connect_in_memory().await.unwrap();
    let ai = FakeAi::new(mode, delay);
    let classifier = Classifier::new(pool.clone(), ai.clone());
    (pool, ai, classifier)
}

fn letters(i: usize) -> String {
    let abc: Vec<char> = "ghijklmnopqrstuvwxyz".chars().collect();
    format!("{}{}", abc[i / abc.len()], abc[i % abc.len()])
}

#[tokio::test]
async fn enrich_cached_never_waits_for_the_ai() {
    let (_pool, _ai, classifier) = setup(Mode::Valid, Duration::from_millis(300)).await;
    let msg = "worker heartbeat lag=42ms queue=emails";
    let mut first = vec![entry(msg, SeveritySource::Default)];
    let started = std::time::Instant::now();
    classifier.enrich_cached(&mut first).await;
    assert!(started.elapsed() < Duration::from_millis(150));
    assert_eq!(first[0].severity_source, SeveritySource::Default);

    tokio::time::sleep(Duration::from_millis(500)).await;
    let mut later = vec![entry(msg, SeveritySource::Default)];
    classifier.enrich_cached(&mut later).await;
    assert_eq!(later[0].severity_source, SeveritySource::Ai);
}

#[tokio::test]
async fn masking_hides_usernames_from_the_ai() {
    let pool = connect_in_memory().await.unwrap();
    let ai = FakeAi::new(Mode::Valid, Duration::ZERO);
    let classifier = Classifier::new(pool, ai.clone()).with_masking(true);
    let mut entries = vec![entry(
        "pam_unix(sshd:session): session opened for user alice by (uid=0)",
        SeveritySource::Default,
    )];
    classifier.enrich(&mut entries).await;
    let sent = ai.calls().concat().join("\n");
    assert!(!sent.contains("alice"), "username leaked: {sent}");
    assert_eq!(entries[0].severity_source, SeveritySource::Ai);
}

#[tokio::test]
async fn same_template_is_classified_once_then_cached() {
    let (_pool, ai, classifier) = setup(Mode::Valid, Duration::ZERO).await;
    let mut first = vec![
        entry(
            "Failed password for root from 1.2.3.4 port 22 ssh2",
            SeveritySource::Default,
        ),
        entry(
            "Failed password for root from 5.6.7.8 port 99 ssh2",
            SeveritySource::Default,
        ),
    ];
    classifier.enrich(&mut first).await;
    assert_eq!(ai.calls().len(), 1);
    assert_eq!(ai.calls()[0].len(), 1);
    for e in &first {
        assert_eq!(e.severity, Severity::Warning);
        assert_eq!(e.severity_source, SeveritySource::Ai);
        assert_eq!(e.reason.as_deref(), Some("looks odd"));
    }

    let mut second = vec![entry(
        "Failed password for root from 9.9.9.9 port 2222 ssh2",
        SeveritySource::Default,
    )];
    classifier.enrich(&mut second).await;
    assert_eq!(ai.calls().len(), 1);
    assert_eq!(second[0].severity, Severity::Warning);
    assert_eq!(second[0].severity_source, SeveritySource::Ai);
    assert_eq!(second[0].reason.as_deref(), Some("looks odd"));
}

#[tokio::test]
async fn invalid_json_keeps_ingest_severity_and_caches_nothing() {
    let (pool, ai, classifier) = setup(Mode::InvalidJson, Duration::ZERO).await;
    let msg = "kernel: something strange happened on device sda";
    let mut entries = vec![entry(msg, SeveritySource::Default)];
    classifier.enrich(&mut entries).await;
    assert_eq!(ai.calls().len(), 1);
    assert_eq!(entries[0].severity, Severity::Normal);
    assert_eq!(entries[0].severity_source, SeveritySource::Default);
    assert_eq!(entries[0].reason, None);
    let cached = severity_cache::get_many(&pool, DEFAULT_TENANT, &[template_hash(msg)])
        .await
        .unwrap();
    assert!(cached.is_empty());
}

#[tokio::test]
async fn rule_entries_are_never_sent_to_ai() {
    let (_pool, ai, classifier) = setup(Mode::Valid, Duration::ZERO).await;
    let mut rule = entry("Out of memory: Killed process 1234", SeveritySource::Rule);
    rule.severity = Severity::Critical;
    let mut entries = vec![rule.clone()];
    classifier.enrich(&mut entries).await;
    assert!(ai.calls().is_empty());
    assert_eq!(entries[0], rule);

    entries.push(entry("cron job finished", SeveritySource::Default));
    classifier.enrich(&mut entries).await;
    assert_eq!(ai.calls(), vec![vec!["1: cron job finished".to_string()]]);
    assert_eq!(entries[0], rule);
}

#[tokio::test]
async fn distinct_templates_are_sent_in_batches_of_twenty() {
    let (_pool, ai, classifier) = setup(Mode::Valid, Duration::ZERO).await;
    let mut entries: Vec<LogEntry> = (0..45)
        .map(|i| {
            entry(
                &format!("service {} failed", letters(i)),
                SeveritySource::Default,
            )
        })
        .collect();
    classifier.enrich(&mut entries).await;
    let mut sizes: Vec<usize> = ai.calls().iter().map(Vec::len).collect();
    sizes.sort_unstable();
    assert_eq!(sizes, vec![5, 20, 20]);
    assert!(entries
        .iter()
        .all(|e| e.severity_source == SeveritySource::Ai));
}

#[tokio::test]
async fn concurrent_calls_share_one_in_flight_request() {
    let (_pool, ai, classifier) = setup(Mode::Valid, Duration::from_millis(100)).await;
    let mut a = vec![entry("disk sda almost full", SeveritySource::Default)];
    let mut b = vec![entry("disk sda almost full", SeveritySource::Default)];
    tokio::join!(classifier.enrich(&mut a), classifier.enrich(&mut b));
    assert_eq!(ai.calls().len(), 1);
}

#[tokio::test]
async fn timeout_leaves_entries_and_caches_in_background() {
    let (pool, ai, classifier) = setup(Mode::Valid, Duration::from_millis(200)).await;
    let classifier = classifier.with_timeout(Duration::from_millis(20));
    let msg = "nginx upstream timed out";
    let mut entries = vec![entry(msg, SeveritySource::Default)];
    classifier.enrich(&mut entries).await;
    assert_eq!(entries[0].severity_source, SeveritySource::Default);

    tokio::time::sleep(Duration::from_millis(400)).await;
    let cached = severity_cache::get_many(&pool, DEFAULT_TENANT, &[template_hash(msg)])
        .await
        .unwrap();
    assert_eq!(cached.len(), 1);

    classifier.enrich(&mut entries).await;
    assert_eq!(ai.calls().len(), 1);
    assert_eq!(entries[0].severity_source, SeveritySource::Ai);
}
