use chrono::{TimeZone, Utc};
use loglens_core::{Direction, LogFilters, LogStore, Severity, StoreError};
use loglens_loki::LokiStore;
use serde_json::json;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn two_streams() -> serde_json::Value {
    json!({
        "status": "success",
        "data": {
            "resultType": "streams",
            "result": [
                {
                    "stream": {"host": "web1", "source": "ssh", "severity": "important"},
                    "values": [
                        ["1763719203000000000", "{\"message\":\"third\",\"severity_source\":\"rule\"}"],
                        ["1763719201000000000", "{\"message\":\"first\",\"severity_source\":\"rule\"}"]
                    ]
                },
                {
                    "stream": {"host": "db1", "source": "docker", "severity": "normal", "container": "pg"},
                    "values": [
                        ["1763719202000000000", "raw second"]
                    ]
                }
            ]
        }
    })
}

async fn server_with_streams() -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/loki/api/v1/query_range"))
        .respond_with(ResponseTemplate::new(200).set_body_json(two_streams()))
        .mount(&server)
        .await;
    server
}

#[tokio::test]
async fn backward_is_newest_first_and_sends_params() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/loki/api/v1/query_range"))
        .and(query_param(
            "query",
            r#"{host="web1", severity=~"critical|important"}"#,
        ))
        .and(query_param("start", "1763715600000000000"))
        .and(query_param("end", "1763719200000000000"))
        .and(query_param("limit", "500"))
        .and(query_param("direction", "backward"))
        .respond_with(ResponseTemplate::new(200).set_body_json(two_streams()))
        .expect(1)
        .mount(&server)
        .await;

    let store = LokiStore::new(&server.uri());
    let filters = LogFilters {
        host: Some("web1".into()),
        severity: vec![Severity::Critical, Severity::Important],
        start: Some(Utc.timestamp_opt(1_763_715_600, 0).unwrap()),
        end: Some(Utc.timestamp_opt(1_763_719_200, 0).unwrap()),
        ..Default::default()
    };
    let entries = store.query(&filters).await.unwrap();
    let messages: Vec<&str> = entries.iter().map(|e| e.message.as_str()).collect();
    assert_eq!(messages, ["third", "raw second", "first"]);
    assert_eq!(entries[1].host, "db1");
    assert_eq!(entries[1].container.as_deref(), Some("pg"));
    assert_eq!(entries[0].severity, Severity::Important);
}

#[tokio::test]
async fn forward_is_oldest_first_and_truncated_to_limit() {
    let server = server_with_streams().await;
    let store = LokiStore::new(&server.uri());
    let filters = LogFilters {
        limit: Some(2),
        direction: Direction::Forward,
        ..Default::default()
    };
    let entries = store.query(&filters).await.unwrap();
    let messages: Vec<&str> = entries.iter().map(|e| e.message.as_str()).collect();
    assert_eq!(messages, ["first", "raw second"]);
}

#[tokio::test]
async fn default_range_is_last_24_hours() {
    let server = server_with_streams().await;
    let store = LokiStore::new(&server.uri());
    store.query(&LogFilters::default()).await.unwrap();

    let requests = server.received_requests().await.unwrap();
    let params: std::collections::HashMap<String, String> =
        requests[0].url.query_pairs().into_owned().collect();
    let start: i64 = params["start"].parse().unwrap();
    let end: i64 = params["end"].parse().unwrap();
    assert_eq!(end - start, 24 * 3600 * 1_000_000_000);
    assert_eq!(params["query"], r#"{severity=~".+"}"#);
}

#[tokio::test]
async fn bad_request_maps_to_invalid_query() {
    let server = MockServer::start().await;
    Mock::given(path("/loki/api/v1/query_range"))
        .respond_with(ResponseTemplate::new(400).set_body_string("parse error"))
        .mount(&server)
        .await;
    let store = LokiStore::new(&server.uri());
    let err = store.query(&LogFilters::default()).await.unwrap_err();
    assert!(matches!(err, StoreError::InvalidQuery(_)), "{err:?}");
}

#[tokio::test]
async fn unreachable_server_is_unavailable() {
    let store = LokiStore::new("http://127.0.0.1:1");
    let err = store.query(&LogFilters::default()).await.unwrap_err();
    assert!(matches!(err, StoreError::Unavailable(_)), "{err:?}");
}
