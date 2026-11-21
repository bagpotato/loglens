mod common;

use std::time::Duration;

use axum::http::StatusCode;
use common::{body_json, entry, get, login, send, setup};
use loglens_core::{Severity, SeveritySource};
use loglens_storage::{severity_cache, CachedSeverity, DEFAULT_TENANT};

#[tokio::test]
async fn logs_are_enriched_and_filters_parsed() {
    let cached_msg = "disk usage at 91% on /dev/sda1";
    let app = setup(vec![
        entry(
            "1-a",
            "Failed password for root",
            Severity::Important,
            SeveritySource::Rule,
        ),
        entry("2-b", cached_msg, Severity::Normal, SeveritySource::Default),
        entry(
            "3-c",
            "unknown event 42",
            Severity::Warning,
            SeveritySource::Default,
        ),
    ])
    .await;
    let cached = CachedSeverity {
        template_hash: loglens_severity::template_hash(cached_msg),
        severity: Severity::Critical,
        reason: "disk almost full".into(),
    };
    severity_cache::put(&app.pool, DEFAULT_TENANT, &cached)
        .await
        .unwrap();
    let token = login(&app).await;

    let uri = "/api/logs?host=web1&severity=critical,important&q=fail\
               &start=2026-11-21T00:00:00Z&limit=100";
    let response = send(&app, get(uri, Some(&token))).await;
    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response).await;
    let entries = json["entries"].as_array().unwrap();
    let severities: Vec<(&str, &str)> = entries
        .iter()
        .map(|e| {
            (
                e["severity"].as_str().unwrap(),
                e["severity_source"].as_str().unwrap(),
            )
        })
        .collect();
    assert_eq!(severities.len(), 2, "warning entry must be filtered out");
    assert!(severities.contains(&("important", "rule")));
    assert!(severities.contains(&("critical", "ai")));
    assert!(entries.iter().any(|e| e["reason"] == "disk almost full"));

    let filters = app.store.last_filters.lock().unwrap().clone().unwrap();
    assert_eq!(filters.host.as_deref(), Some("web1"));
    assert!(
        filters.severity == vec![Severity::Critical, Severity::Important]
            || filters.severity == vec![Severity::Warning, Severity::Normal]
    );
    assert_eq!(filters.q.as_deref(), Some("fail"));
    assert_eq!(filters.limit, Some(100));
    assert!(filters.start.is_some());
    assert!(filters.end.is_none());
}

#[tokio::test]
async fn invalid_log_params_are_bad_requests() {
    let app = setup(Vec::new()).await;
    let token = login(&app).await;
    for uri in [
        "/api/logs?start=yesterday",
        "/api/logs?severity=critical,loud",
        "/api/logs?limit=many",
        "/api/logs?start=2026-11-21T12:00:00Z&end=2026-11-21T10:00:00Z",
    ] {
        let response = send(&app, get(uri, Some(&token))).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST, "{uri}");
        assert!(body_json(response).await["error"].is_string());
    }
    assert!(app.store.last_filters.lock().unwrap().is_none());
}

#[tokio::test]
async fn stats_parses_range() {
    let app = setup(Vec::new()).await;
    let token = login(&app).await;

    let response = send(&app, get("/api/stats", Some(&token))).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        *app.store.last_range.lock().unwrap(),
        Some(Duration::from_secs(24 * 3600))
    );

    let response = send(&app, get("/api/stats?range=15m", Some(&token))).await;
    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response).await;
    assert_eq!(json["by_severity"]["critical"], 3);
    assert_eq!(json["by_source"]["ssh"], 3);
    assert_eq!(
        *app.store.last_range.lock().unwrap(),
        Some(Duration::from_secs(900))
    );

    for bad in ["31d", "abc", "0h", "12"] {
        let uri = format!("/api/stats?range={bad}");
        let response = send(&app, get(&uri, Some(&token))).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST, "{bad}");
    }
}

#[tokio::test]
async fn tail_requires_websocket_upgrade() {
    let app = setup(Vec::new()).await;
    let token = login(&app).await;
    let response = send(&app, get("/api/logs/tail", Some(&token))).await;
    assert!(response.status().is_client_error());
}
