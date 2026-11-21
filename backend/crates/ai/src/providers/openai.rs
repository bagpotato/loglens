use async_trait::async_trait;
use reqwest::Client;
use serde::Serialize;
use serde_json::Value;

use super::{http_client, json_body, send, text_stream, Step, COMPLETE_TIMEOUT};
use crate::{AiError, AiProvider, ChatMessage, TextStream};

/// OpenAI-compatible chat completions API (`POST {url}/v1/chat/completions`).
#[derive(Clone)]
pub struct OpenAiProvider {
    client: Client,
    endpoint: String,
    model: String,
    key: Option<String>,
}

impl std::fmt::Debug for OpenAiProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OpenAiProvider")
            .field("endpoint", &self.endpoint)
            .field("model", &self.model)
            .field("key", &self.key.as_ref().map(|_| "***"))
            .finish()
    }
}

#[derive(Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: &'a [ChatMessage],
    stream: bool,
}

impl OpenAiProvider {
    /// `url` may or may not already end with `/v1`.
    pub fn new(url: &str, model: &str, key: Option<String>) -> Self {
        let base = url.trim_end_matches('/');
        let endpoint = if base.ends_with("/v1") {
            format!("{base}/chat/completions")
        } else {
            format!("{base}/v1/chat/completions")
        };
        Self {
            client: http_client(),
            endpoint,
            model: model.to_string(),
            key: key.filter(|k| !k.is_empty()),
        }
    }

    fn request(&self, messages: &[ChatMessage], stream: bool) -> reqwest::RequestBuilder {
        let request = self.client.post(&self.endpoint).json(&ChatRequest {
            model: &self.model,
            messages,
            stream,
        });
        match &self.key {
            Some(key) => request.bearer_auth(key),
            None => request,
        }
    }
}

fn parse_line(line: &str) -> Result<Step, AiError> {
    let Some(payload) = line.strip_prefix("data:") else {
        return Ok(Step::Skip);
    };
    let payload = payload.trim();
    if payload == "[DONE]" {
        return Ok(Step::Done(None));
    }
    let value: Value = serde_json::from_str(payload)
        .map_err(|e| AiError::InvalidResponse(format!("bad stream event: {e}")))?;
    if let Some(error) = value.get("error") {
        let message = error
            .get("message")
            .and_then(Value::as_str)
            .map_or_else(|| error.to_string(), str::to_string);
        return Err(AiError::InvalidResponse(message));
    }
    Ok(value
        .pointer("/choices/0/delta/content")
        .and_then(Value::as_str)
        .map_or(Step::Skip, |t| Step::Text(t.to_string())))
}

#[async_trait]
impl AiProvider for OpenAiProvider {
    async fn complete(&self, messages: &[ChatMessage]) -> Result<String, AiError> {
        let request = self.request(messages, false).timeout(COMPLETE_TIMEOUT);
        let value = json_body(send(request).await?).await?;
        value
            .pointer("/choices/0/message/content")
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| AiError::InvalidResponse("missing choices[0].message.content".into()))
    }

    async fn stream(&self, messages: &[ChatMessage]) -> Result<TextStream, AiError> {
        let response = send(self.request(messages, true)).await?;
        Ok(text_stream(response, parse_line))
    }
}
