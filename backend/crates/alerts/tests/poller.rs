use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use chrono::{TimeDelta, Utc};
use loglens_alerts::{Alert, AlertError, Dedup, Notifier, Poller, DEFAULT_WINDOW};
use loglens_core::{
    Direction, LogEntry, LogFilters, LogStore, LogStream, Meta, Severity, SeveritySource, Stats,
    StoreError,
};

#[derive(Default)]
struct FakeStore {
    entries: Mutex<Vec<LogEntry>>,
    fail: bool,
}

impl FakeStore {
    fn push(&self, id: &str, message: &str) {
        self.entries.lock().unwrap().push(LogEntry {
            id: id.into(),
            timestamp: Utc::now(),
            host: "web1".into(),
            source: "kernel".into(),
            container: None,
            unit: None,
            message: message.into(),
            severity: Severity::Critical,
            severity_source: SeveritySource::Rule,
            reason: None,
        });
    }
}

#[async_trait]
impl LogStore for FakeStore {
    async fn query(&self, f: &LogFilters) -> Result<Vec<LogEntry>, StoreError> {
        if self.fail {
            return Err(StoreError::Unavailable("down".into()));
        }
        assert_eq!(f.severity, vec![Severity::Critical]);
        assert_eq!(f.direction, Direction::Forward);
        let (start, end) = (f.start.unwrap(), f.end.unwrap());
        Ok(self
            .entries
            .lock()
            .unwrap()
            .iter()
            .filter(|e| e.timestamp >= start && e.timestamp <= end)
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
        Err(StoreError::Unavailable("no tail".into()))
    }
}

#[derive(Default)]
struct Recorder(Mutex<Vec<Alert>>);

#[async_trait]
impl Notifier for Recorder {
    async fn notify(&self, alert: &Alert) -> Result<(), AlertError> {
        self.0.lock().unwrap().push(alert.clone());
        Ok(())
    }
}

async fn poller(store: Arc<FakeStore>, window: Duration) -> (Poller, Arc<Recorder>) {
    let recorder = Arc::new(Recorder::default());
    let pool = loglens_storage::connect_in_memory().await.unwrap();
    let p = Poller::new(store, recorder.clone(), Dedup::new(pool, window));
    (p, recorder)
}

async fn tick() {
    tokio::time::sleep(Duration::from_millis(5)).await;
}

#[tokio::test]
async fn same_message_in_two_polls_notifies_once() {
    let store = Arc::new(FakeStore::default());
    let (mut p, rec) = poller(store.clone(), DEFAULT_WINDOW).await;
    tick().await;
    store.push("1", "kernel panic test");
    assert_eq!(p.poll_once().await, 1);
    tick().await;
    store.push("2", "kernel panic test");
    assert_eq!(p.poll_once().await, 0);
    let alerts = rec.0.lock().unwrap();
    assert_eq!(alerts.len(), 1);
    assert_eq!(alerts[0].title, "LogLens critical on web1");
    assert_eq!(alerts[0].message, "kernel panic test");
}

#[tokio::test]
async fn different_templates_notify_separately() {
    let store = Arc::new(FakeStore::default());
    let (mut p, rec) = poller(store.clone(), DEFAULT_WINDOW).await;
    tick().await;
    store.push("1", "kernel panic test");
    store.push("2", "disk sda failure");
    assert_eq!(p.poll_once().await, 2);
    assert_eq!(rec.0.lock().unwrap().len(), 2);
}

#[tokio::test]
async fn entries_are_never_processed_twice() {
    let store = Arc::new(FakeStore::default());
    let (mut p, rec) = poller(store.clone(), Duration::ZERO).await;
    tick().await;
    store.push("1", "kernel panic test");
    assert_eq!(p.poll_once().await, 1);
    assert_eq!(p.poll_once().await, 0);
    tick().await;
    store.push("2", "kernel panic test");
    assert_eq!(p.poll_once().await, 1);
    assert_eq!(rec.0.lock().unwrap().len(), 2);
}

#[tokio::test]
async fn history_before_start_is_ignored() {
    let store = Arc::new(FakeStore::default());
    store.push("old", "kernel panic test");
    store.entries.lock().unwrap()[0].timestamp -= TimeDelta::minutes(5);
    let (mut p, rec) = poller(store, DEFAULT_WINDOW).await;
    assert_eq!(p.poll_once().await, 0);
    assert!(rec.0.lock().unwrap().is_empty());
}

struct FlakyNotifier {
    failures_left: Mutex<u32>,
    delivered: Mutex<u32>,
}

#[async_trait]
impl Notifier for FlakyNotifier {
    async fn notify(&self, _alert: &Alert) -> Result<(), AlertError> {
        let mut left = self.failures_left.lock().unwrap();
        if *left > 0 {
            *left -= 1;
            return Err(AlertError::Http("ntfy down".into()));
        }
        *self.delivered.lock().unwrap() += 1;
        Ok(())
    }
}

#[tokio::test]
async fn failed_notification_does_not_mute_the_template() {
    let store = Arc::new(FakeStore::default());
    let notifier = Arc::new(FlakyNotifier {
        failures_left: Mutex::new(1),
        delivered: Mutex::new(0),
    });
    let pool = loglens_storage::connect_in_memory().await.unwrap();
    let mut p = Poller::new(
        store.clone(),
        notifier.clone(),
        Dedup::new(pool, DEFAULT_WINDOW),
    );
    tick().await;
    store.push("1", "kernel panic test");
    assert_eq!(p.poll_once().await, 0);
    tick().await;
    store.push("2", "kernel panic test");
    assert_eq!(p.poll_once().await, 1);
    assert_eq!(*notifier.delivered.lock().unwrap(), 1);
}

#[tokio::test]
async fn late_arriving_entry_is_still_alerted() {
    let store = Arc::new(FakeStore::default());
    let (mut p, rec) = poller(store.clone(), DEFAULT_WINDOW).await;
    tokio::time::sleep(Duration::from_millis(50)).await;
    store.push("a", "kernel panic test");
    assert_eq!(p.poll_once().await, 1);
    store.push("b", "disk sda failure");
    store.entries.lock().unwrap()[1].timestamp -= TimeDelta::milliseconds(20);
    assert_eq!(p.poll_once().await, 1);
    assert_eq!(rec.0.lock().unwrap().len(), 2);
}

#[tokio::test]
async fn store_error_sends_nothing() {
    let store = Arc::new(FakeStore {
        fail: true,
        ..Default::default()
    });
    let (mut p, rec) = poller(store, DEFAULT_WINDOW).await;
    assert_eq!(p.poll_once().await, 0);
    assert!(rec.0.lock().unwrap().is_empty());
}
