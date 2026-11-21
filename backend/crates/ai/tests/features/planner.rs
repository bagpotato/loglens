use chrono::{Duration, TimeZone, Utc};
use loglens_ai::{fallback_filters, plan_query, MAX_CHAT_LOGS};
use loglens_core::{Meta, Severity};

use crate::fakes::FakeProvider;

fn meta() -> Meta {
    Meta {
        hosts: vec!["web1".to_string(), "db1".to_string()],
        sources: vec!["ssh".to_string(), "nginx".to_string()],
    }
}

fn fixed_now() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 11, 21, 15, 30, 0).unwrap()
}

#[tokio::test]
async fn valid_json_becomes_filters() {
    let provider = FakeProvider::new(
        r#"{"host":"web1","source":"ssh","severity":["important","critical"],"q":"Failed password",
            "start":"2026-11-21T00:00:00Z","end":"2026-11-21T15:30:00Z","limit":100}"#,
    );
    let f = plan_query(&provider, "ssh brute force today?", &meta(), fixed_now()).await;

    assert_eq!(f.host.as_deref(), Some("web1"));
    assert_eq!(f.source.as_deref(), Some("ssh"));
    assert_eq!(f.severity, vec![Severity::Critical, Severity::Important]);
    assert_eq!(f.q.as_deref(), Some("Failed password"));
    assert_eq!(
        f.start,
        Some(Utc.with_ymd_and_hms(2026, 11, 21, 0, 0, 0).unwrap())
    );
    assert_eq!(f.end, Some(fixed_now()));
    assert_eq!(f.limit, Some(100));
}

#[tokio::test]
async fn prompt_contains_time_hosts_sources_and_severities() {
    let provider = FakeProvider::new("{}");
    plan_query(&provider, "¿hubo errores?", &meta(), fixed_now()).await;

    let sent = provider.last_complete();
    let user = &sent[1].content;
    assert!(sent[0].content.contains("JSON"));
    assert!(user.contains("2026-11-21T15:30:00Z"));
    assert!(user.contains("web1, db1"));
    assert!(user.contains("ssh, nginx"));
    assert!(user.contains("critical, important, warning, normal"));
    assert!(user.contains("¿hubo errores?"));
}

#[tokio::test]
async fn fenced_json_with_prose_is_accepted() {
    let provider = FakeProvider::new(
        "Sure! Here is the filter:\n```json\n{\"source\": \"SSH\", \"severity\": \"important\", \"q\": null}\n```\nHope it helps {not json}",
    );
    let f = plan_query(&provider, "ssh?", &meta(), fixed_now()).await;

    assert_eq!(f.source.as_deref(), Some("ssh"));
    assert_eq!(f.severity, vec![Severity::Important]);
    assert_eq!(f.q, None);
    assert_eq!(f.end, Some(fixed_now()));
    assert_eq!(f.start, Some(fixed_now() - Duration::hours(24)));
}

#[tokio::test]
async fn garbage_falls_back_to_last_24h() {
    let provider = FakeProvider::new("I cannot help with that.");
    let f = plan_query(&provider, "what happened?", &meta(), fixed_now()).await;
    assert_eq!(f, fallback_filters(fixed_now()));
    assert_eq!(f.host, None);
    assert!(f.severity.is_empty());
    assert_eq!(f.start, Some(fixed_now() - Duration::hours(24)));
}

#[tokio::test]
async fn provider_error_falls_back_to_last_24h() {
    let provider = FakeProvider::failing();
    let f = plan_query(&provider, "what happened?", &meta(), fixed_now()).await;
    assert_eq!(f, fallback_filters(fixed_now()));
}

#[tokio::test]
async fn unknown_values_are_dropped_and_limits_enforced() {
    let provider = FakeProvider::new(
        r#"{"host":"mail9","source":"postfix","severity":["fatal","warning"],
            "start":"2026-11-21T12:00:00Z","end":"2026-11-21T10:00:00Z","limit":5000}"#,
    );
    let f = plan_query(&provider, "mail?", &meta(), fixed_now()).await;

    assert_eq!(f.host, None);
    assert_eq!(f.source, None);
    assert_eq!(f.severity, vec![Severity::Warning]);
    assert_eq!(f.start, Some(fixed_now() - Duration::hours(24)));
    assert_eq!(f.end, Some(fixed_now()));
    assert_eq!(f.limit, Some(MAX_CHAT_LOGS));
}

#[tokio::test]
async fn invalid_timestamps_default_to_last_24h() {
    let provider = FakeProvider::new(r#"{"start":"yesterday","end":"now"}"#);
    let f = plan_query(&provider, "q", &meta(), fixed_now()).await;
    assert_eq!(f.start, Some(fixed_now() - Duration::hours(24)));
    assert_eq!(f.end, Some(fixed_now()));
}
