use std::time::Duration;

use loglens_core::LogStore;
use loglens_loki::LokiStore;
use serde_json::json;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn meta_lists_sorted_hosts_and_sources() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/loki/api/v1/label/host/values"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"status": "success", "data": ["web2", "db1", "web1"]})),
        )
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/loki/api/v1/label/source/values"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"status": "success", "data": ["ssh", "kernel"]})),
        )
        .mount(&server)
        .await;

    let meta = LokiStore::new(&server.uri()).meta().await.unwrap();
    assert_eq!(meta.hosts, ["db1", "web1", "web2"]);
    assert_eq!(meta.sources, ["kernel", "ssh"]);
}

#[tokio::test]
async fn meta_missing_label_is_empty() {
    let server = MockServer::start().await;
    Mock::given(path("/loki/api/v1/label/host/values"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"status": "success"})))
        .mount(&server)
        .await;
    Mock::given(path("/loki/api/v1/label/source/values"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({"status": "success", "data": []})),
        )
        .mount(&server)
        .await;

    let meta = LokiStore::new(&server.uri()).meta().await.unwrap();
    assert!(meta.hosts.is_empty());
    assert!(meta.sources.is_empty());
}

fn vector(label: &str, rows: &[(&str, &str)]) -> serde_json::Value {
    let result: Vec<_> = rows
        .iter()
        .map(|(k, v)| json!({"metric": {label: k}, "value": [1763719200.123, v]}))
        .collect();
    json!({"status": "success", "data": {"resultType": "vector", "result": result}})
}

#[tokio::test]
async fn stats_runs_two_instant_queries() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/loki/api/v1/query"))
        .and(query_param(
            "query",
            r#"sum by (severity) (count_over_time({severity=~".+"}[90m]))"#,
        ))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(vector("severity", &[("critical", "3"), ("normal", "120")])),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/loki/api/v1/query"))
        .and(query_param(
            "query",
            r#"sum by (source) (count_over_time({severity=~".+"}[90m]))"#,
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(vector("source", &[("ssh", "123")])))
        .expect(1)
        .mount(&server)
        .await;

    let stats = LokiStore::new(&server.uri())
        .stats(Duration::from_secs(90 * 60))
        .await
        .unwrap();
    assert_eq!(stats.by_severity.get("critical"), Some(&3));
    assert_eq!(stats.by_severity.get("normal"), Some(&120));
    assert_eq!(stats.by_source.get("ssh"), Some(&123));
    assert_eq!(stats.by_source.len(), 1);
}
