#![allow(dead_code)]

use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use async_trait::async_trait;
use axum::{
    body::{to_bytes, Body},
    http::{header, Request, Response, StatusCode},
    Router,
};
use chrono::{TimeZone, Utc};
use futures::{stream, StreamExt};
use loglens_ai::{AiError, AiProvider, ChatMessage, TextStream};
use loglens_api::{build_router, AppState};
use loglens_auth::JwtKeys;
use loglens_core::{
    LogEntry, LogFilters, LogStore, LogStream, Meta, Severity, SeveritySource, Stats, StoreError,
};
use loglens_severity::Classifier;
use loglens_storage::SqlitePool;
use serde_json::{json, Value};
use tower::ServiceExt;

pub const ADMIN: &str = "admin";
pub const PASSWORD: &str = "correct-horse-battery";
const SECRET: &str = "0123456789abcdef0123456789abcdef";

#[derive(Default)]
pub struct FakeStore {
    pub entries: Vec<LogEntry>,
    pub last_filters: Mutex<Option<LogFilters>>,
    pub last_range: Mutex<Option<Duration>>,
}

#[async_trait]
impl LogStore for FakeStore {
    async fn query(&self, filters: &LogFilters) -> Result<Vec<LogEntry>, StoreError> {
        if let Ok(mut last) = self.last_filters.lock() {
            *last = Some(filters.clone());
        }
        Ok(self.entries.clone())
    }

    async fn meta(&self) -> Result<Meta, StoreError> {
        Ok(Meta {
            hosts: vec!["web1".into()],
            sources: vec!["ssh".into()],
        })
    }

    async fn stats(&self, range: Duration) -> Result<Stats, StoreError> {
        if let Ok(mut last) = self.last_range.lock() {
            *last = Some(range);
        }
        let mut stats = Stats::default();
        stats.by_severity.insert("critical".into(), 3);
        stats.by_source.insert("ssh".into(), 3);
        Ok(stats)
    }

    async fn tail(&self, _filters: &LogFilters) -> Result<LogStream, StoreError> {
        Ok(stream::iter(self.entries.clone().into_iter().map(Ok)).boxed())
    }
}

/// Answers every completion with non-JSON text and streams two chunks.
pub struct FakeAi;

#[async_trait]
impl AiProvider for FakeAi {
    async fn complete(&self, _messages: &[ChatMessage]) -> Result<String, AiError> {
        Ok("no structured answer".into())
    }

    async fn stream(&self, _messages: &[ChatMessage]) -> Result<TextStream, AiError> {
        let chunks = ["Hello", " world"].map(|c| Ok(c.to_string()));
        Ok(stream::iter(chunks).boxed())
    }
}

pub struct TestApp {
    pub router: Router,
    pub store: Arc<FakeStore>,
    pub pool: SqlitePool,
}

pub async fn setup(entries: Vec<LogEntry>) -> TestApp {
    let pool = loglens_storage::connect_in_memory().await.unwrap();
    loglens_auth::ensure_admin(&pool, ADMIN, Some(PASSWORD))
        .await
        .unwrap();
    let store = Arc::new(FakeStore {
        entries,
        ..Default::default()
    });
    let ai: Arc<dyn AiProvider> = Arc::new(FakeAi);
    let classifier = Classifier::new(pool.clone(), ai.clone()).with_timeout(Duration::from_secs(2));
    let state = AppState {
        pool: pool.clone(),
        jwt: JwtKeys::new(SECRET),
        store: store.clone(),
        ai,
        classifier: Arc::new(classifier),
        ai_mask: false,
    };
    TestApp {
        router: build_router(state),
        store,
        pool,
    }
}

pub fn entry(id: &str, message: &str, severity: Severity, source: SeveritySource) -> LogEntry {
    LogEntry {
        id: id.into(),
        timestamp: Utc.with_ymd_and_hms(2026, 11, 21, 10, 0, 0).unwrap(),
        host: "web1".into(),
        source: "ssh".into(),
        container: None,
        unit: None,
        message: message.into(),
        severity,
        severity_source: source,
        reason: None,
    }
}

pub async fn send(app: &TestApp, request: Request<Body>) -> Response<Body> {
    app.router.clone().oneshot(request).await.unwrap()
}

pub fn get(uri: &str, token: Option<&str>) -> Request<Body> {
    let mut builder = Request::get(uri);
    if let Some(token) = token {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }
    builder.body(Body::empty()).unwrap()
}

pub fn post_json(uri: &str, token: Option<&str>, body: &Value) -> Request<Body> {
    let mut builder = Request::post(uri).header(header::CONTENT_TYPE, "application/json");
    if let Some(token) = token {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }
    builder.body(Body::from(body.to_string())).unwrap()
}

pub async fn body_text(response: Response<Body>) -> String {
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    String::from_utf8(bytes.to_vec()).unwrap()
}

pub async fn body_json(response: Response<Body>) -> Value {
    serde_json::from_str(&body_text(response).await).unwrap()
}

pub async fn login(app: &TestApp) -> String {
    let body = json!({ "username": ADMIN, "password": PASSWORD });
    let response = send(app, post_json("/api/auth/login", None, &body)).await;
    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response).await;
    json["token"].as_str().unwrap().to_string()
}
