use futures::StreamExt;
use loglens_ai::{build_provider, AiError, AiProvider, ChatMessage, OllamaProvider};
use loglens_config::{AiProviderKind, AiSettings};
use serde_json::json;
use wiremock::matchers::{body_partial_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn messages() -> Vec<ChatMessage> {
    vec![ChatMessage::system("be brief"), ChatMessage::user("hi")]
}

#[tokio::test]
async fn complete_reads_message_content() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/chat"))
        .and(body_partial_json(json!({
            "model": "llama3",
            "stream": false,
            "messages": [{"role": "system", "content": "be brief"}, {"role": "user", "content": "hi"}]
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "model": "llama3",
            "message": {"role": "assistant", "content": "hello there"},
            "done": true
        })))
        .expect(1)
        .mount(&server)
        .await;

    let provider = OllamaProvider::new(&server.uri(), "llama3");
    assert_eq!(provider.complete(&messages()).await.unwrap(), "hello there");
}

#[tokio::test]
async fn stream_yields_chunks_until_done() {
    let server = MockServer::start().await;
    let body = [
        r#"{"message":{"role":"assistant","content":"Hel"},"done":false}"#,
        "",
        r#"{"message":{"role":"assistant","content":"lo "},"done":false}"#,
        r#"{"message":{"role":"assistant","content":"world"},"done":false}"#,
        r#"{"message":{"role":"assistant","content":""},"done":true}"#,
        r#"{"message":{"role":"assistant","content":"ignored"},"done":false}"#,
    ]
    .join("\n");
    Mock::given(method("POST"))
        .and(path("/api/chat"))
        .and(body_partial_json(json!({"stream": true})))
        .respond_with(ResponseTemplate::new(200).set_body_raw(body, "application/x-ndjson"))
        .mount(&server)
        .await;

    let provider = OllamaProvider::new(&server.uri(), "llama3");
    let chunks: Vec<String> = provider
        .stream(&messages())
        .await
        .unwrap()
        .map(|c| c.unwrap())
        .collect()
        .await;
    assert_eq!(chunks, vec!["Hel", "lo ", "world"]);
}

#[tokio::test]
async fn stream_reports_inline_error() {
    let server = MockServer::start().await;
    let body = "{\"message\":{\"content\":\"a\"},\"done\":false}\n{\"error\":\"model crashed\"}\n";
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(body, "application/x-ndjson"))
        .mount(&server)
        .await;

    let provider = OllamaProvider::new(&server.uri(), "llama3");
    let items: Vec<_> = provider.stream(&messages()).await.unwrap().collect().await;
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].as_ref().unwrap(), "a");
    assert!(matches!(&items[1], Err(AiError::InvalidResponse(m)) if m == "model crashed"));
}

#[tokio::test]
async fn non_success_status_is_status_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(404).set_body_string("model not found"))
        .mount(&server)
        .await;

    let provider = OllamaProvider::new(&server.uri(), "missing");
    let err = provider.complete(&messages()).await.unwrap_err();
    assert!(matches!(err, AiError::Status { status: 404, ref body } if body == "model not found"));
    let err = provider.stream(&messages()).await.err().unwrap();
    assert!(matches!(err, AiError::Status { status: 404, .. }));
}

#[tokio::test]
async fn unreachable_server_is_http_error() {
    let provider = OllamaProvider::new("http://127.0.0.1:1", "llama3");
    let err = provider.complete(&messages()).await.unwrap_err();
    assert!(matches!(err, AiError::Http(_)));
}

#[tokio::test]
async fn factory_builds_ollama() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/chat"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({"message": {"content": "ok"}})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let provider = build_provider(&AiSettings {
        provider: AiProviderKind::Ollama,
        url: server.uri(),
        model: "llama3".into(),
        key: None,
        mask: true,
    });
    assert_eq!(provider.complete(&messages()).await.unwrap(), "ok");
}
