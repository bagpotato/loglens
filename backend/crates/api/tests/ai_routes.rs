mod common;

use axum::http::{header, StatusCode};
use common::{body_json, body_text, entry, login, post_json, send, setup};
use loglens_core::{Severity, SeveritySource};
use serde_json::{json, Value};

#[tokio::test]
async fn chat_streams_server_sent_events() {
    let app = setup(vec![entry(
        "1-a",
        "Failed password for root from 10.0.0.5",
        Severity::Important,
        SeveritySource::Rule,
    )])
    .await;
    let token = login(&app).await;
    let body = json!({
        "question": "were there SSH brute force attempts today?",
        "history": [{ "role": "user", "content": "hi" }, { "role": "assistant", "content": "hello" }]
    });
    let response = send(&app, post_json("/api/ai/chat", Some(&token), &body)).await;
    assert_eq!(response.status(), StatusCode::OK);
    let content_type = response.headers()[header::CONTENT_TYPE].to_str().unwrap();
    assert!(
        content_type.starts_with("text/event-stream"),
        "{content_type}"
    );

    let text = body_text(response).await;
    let events: Vec<Value> = text
        .lines()
        .filter_map(|line| line.strip_prefix("data:"))
        .map(|data| serde_json::from_str(data.trim()).unwrap())
        .collect();
    let types: Vec<&str> = events.iter().filter_map(|e| e["type"].as_str()).collect();
    assert_eq!(types.first(), Some(&"filters"), "{text}");
    assert_eq!(types.last(), Some(&"done"), "{text}");
    assert!(types.contains(&"delta"), "{text}");
    assert!(events[0]["filters"].is_object());
    assert!(events[0]["count"].is_u64());
    let answer: String = events
        .iter()
        .filter(|e| e["type"] == "delta")
        .filter_map(|e| e["text"].as_str())
        .collect();
    assert_eq!(answer, "Hello world");
}

#[tokio::test]
async fn chat_rejects_empty_question() {
    let app = setup(Vec::new()).await;
    let token = login(&app).await;
    let body = json!({ "question": "   " });
    let response = send(&app, post_json("/api/ai/chat", Some(&token), &body)).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert!(body_json(response).await["error"].is_string());
}

#[tokio::test]
async fn explain_returns_explanation() {
    let target = entry(
        "1-a",
        "kernel panic",
        Severity::Critical,
        SeveritySource::Rule,
    );
    let app = setup(vec![target.clone()]).await;
    let token = login(&app).await;
    let body = json!({ "entry": target });
    let response = send(&app, post_json("/api/ai/explain", Some(&token), &body)).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert!(body_json(response).await["explanation"].is_string());
}
