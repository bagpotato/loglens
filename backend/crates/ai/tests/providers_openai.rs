use futures::StreamExt;
use loglens_ai::{build_provider, AiError, AiProvider, ChatMessage, OpenAiProvider};
use loglens_config::{AiProviderKind, AiSettings};
use serde_json::json;
use wiremock::matchers::{body_partial_json, header, method, path};
use wiremock::{Mock, MockServer, Request, ResponseTemplate};

fn messages() -> Vec<ChatMessage> {
    vec![ChatMessage::user("hi")]
}

fn completion(text: &str) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(json!({
        "choices": [{"index": 0, "message": {"role": "assistant", "content": text}}]
    }))
}

#[tokio::test]
async fn complete_sends_bearer_and_reads_choice() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .and(header("authorization", "Bearer sk-test"))
        .and(body_partial_json(json!({
            "model": "gpt-x",
            "stream": false,
            "messages": [{"role": "user", "content": "hi"}]
        })))
        .respond_with(completion("hello"))
        .expect(1)
        .mount(&server)
        .await;

    let provider = OpenAiProvider::new(&server.uri(), "gpt-x", Some("sk-test".into()));
    assert_eq!(provider.complete(&messages()).await.unwrap(), "hello");
}

#[tokio::test]
async fn no_key_sends_no_authorization_and_v1_is_not_doubled() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .and(|req: &Request| !req.headers.contains_key("authorization"))
        .respond_with(completion("local"))
        .expect(1)
        .mount(&server)
        .await;

    let url = format!("{}/v1/", server.uri());
    let provider = OpenAiProvider::new(&url, "gpt-x", None);
    assert_eq!(provider.complete(&messages()).await.unwrap(), "local");
}

#[tokio::test]
async fn base_url_with_a_path_is_used_as_is_for_gemini() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1beta/openai/chat/completions"))
        .respond_with(completion("gemini"))
        .expect(1)
        .mount(&server)
        .await;

    let url = format!("{}/v1beta/openai", server.uri());
    let provider = OpenAiProvider::new(&url, "gemini-x", Some("k".into()));
    assert_eq!(provider.complete(&messages()).await.unwrap(), "gemini");
}

#[tokio::test]
async fn overloaded_model_is_retried_then_succeeds() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(503).set_body_string("high demand"))
        .up_to_n_times(1)
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(completion("after retry"))
        .expect(1)
        .mount(&server)
        .await;

    let provider = OpenAiProvider::new(&server.uri(), "m", None);
    assert_eq!(provider.complete(&messages()).await.unwrap(), "after retry");
}

#[tokio::test]
async fn persistent_overload_fails_after_three_attempts() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(503).set_body_string("high demand"))
        .expect(3)
        .mount(&server)
        .await;

    let provider = OpenAiProvider::new(&server.uri(), "m", None);
    let err = provider.complete(&messages()).await.unwrap_err();
    assert!(matches!(err, AiError::Status { status: 503, .. }));
}

#[tokio::test]
async fn stream_parses_sse_until_done() {
    let server = MockServer::start().await;
    let body = concat!(
        ": keep-alive\n\n",
        "data: {\"choices\":[{\"delta\":{\"role\":\"assistant\"}}]}\n\n",
        "data: {\"choices\":[{\"delta\":{\"content\":\"Hel\"}}]}\n\n",
        "data:{\"choices\":[{\"delta\":{\"content\":\"lo\"}}]}\r\n\r\n",
        "data: {\"choices\":[{\"delta\":{},\"finish_reason\":\"stop\"}]}\n\n",
        "data: [DONE]\n\n",
        "data: {\"choices\":[{\"delta\":{\"content\":\"ignored\"}}]}\n\n",
    );
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .and(header("authorization", "Bearer sk-test"))
        .and(body_partial_json(json!({"stream": true})))
        .respond_with(ResponseTemplate::new(200).set_body_raw(body, "text/event-stream"))
        .mount(&server)
        .await;

    let provider = OpenAiProvider::new(&server.uri(), "gpt-x", Some("sk-test".into()));
    let chunks: Vec<String> = provider
        .stream(&messages())
        .await
        .unwrap()
        .map(|c| c.unwrap())
        .collect()
        .await;
    assert_eq!(chunks, vec!["Hel", "lo"]);
}

#[tokio::test]
async fn non_success_status_is_status_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(401).set_body_string("{\"error\":\"bad key\"}"))
        .mount(&server)
        .await;

    let provider = OpenAiProvider::new(&server.uri(), "gpt-x", Some("wrong".into()));
    let err = provider.complete(&messages()).await.unwrap_err();
    assert!(matches!(err, AiError::Status { status: 401, ref body } if body.contains("bad key")));
    let err = provider.stream(&messages()).await.err().unwrap();
    assert!(matches!(err, AiError::Status { status: 401, .. }));
}

#[tokio::test]
async fn malformed_completion_is_invalid_response() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"choices": []})))
        .mount(&server)
        .await;

    let provider = OpenAiProvider::new(&server.uri(), "gpt-x", None);
    let err = provider.complete(&messages()).await.unwrap_err();
    assert!(matches!(err, AiError::InvalidResponse(_)));
}

#[tokio::test]
async fn factory_builds_openai_with_key() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .and(header("authorization", "Bearer k"))
        .respond_with(completion("ok"))
        .expect(1)
        .mount(&server)
        .await;

    let provider = build_provider(&AiSettings {
        provider: AiProviderKind::OpenAi,
        url: server.uri(),
        model: "gpt-x".into(),
        key: Some("k".into()),
        mask: false,
    });
    assert_eq!(provider.complete(&messages()).await.unwrap(), "ok");
}
