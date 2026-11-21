use async_trait::async_trait;
use reqwest::Client;
use serde::Serialize;
use serde_json::Value;

use super::{http_client, json_body, send, text_stream, Step, COMPLETE_TIMEOUT};
use crate::{AiError, AiProvider, ChatMessage, TextStream};

/// Ollama chat API (`POST {url}/api/chat`).
#[derive(Debug, Clone)]
pub struct OllamaProvider {
    client: Client,
    endpoint: String,
    model: String,
}

#[derive(Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: &'a [ChatMessage],
    stream: bool,
}

impl OllamaProvider {
    pub fn new(url: &str, model: &str) -> Self {
        Self {
            client: http_client(),
            endpoint: format!("{}/api/chat", url.trim_end_matches('/')),
            model: model.to_string(),
        }
    }

    fn request(&self, messages: &[ChatMessage], stream: bool) -> reqwest::RequestBuilder {
        self.client.post(&self.endpoint).json(&ChatRequest {
            model: &self.model,
            messages,
            stream,
        })
    }
}

fn content(value: &Value) -> Option<&str> {
    value.pointer("/message/content").and_then(Value::as_str)
}

fn parse_line(line: &str) -> Result<Step, AiError> {
    let value: Value = serde_json::from_str(line)
        .map_err(|e| AiError::InvalidResponse(format!("bad stream line: {e}")))?;
    if let Some(error) = value.get("error").and_then(Value::as_str) {
        return Err(AiError::InvalidResponse(error.to_string()));
    }
    let text = content(&value).map(str::to_string);
    if value.get("done").and_then(Value::as_bool) == Some(true) {
        return Ok(Step::Done(text));
    }
    Ok(text.map_or(Step::Skip, Step::Text))
}

#[async_trait]
impl AiProvider for OllamaProvider {
    async fn complete(&self, messages: &[ChatMessage]) -> Result<String, AiError> {
        let request = self.request(messages, false).timeout(COMPLETE_TIMEOUT);
        let value = json_body(send(request).await?).await?;
        content(&value)
            .map(str::to_string)
            .ok_or_else(|| AiError::InvalidResponse("missing message.content".into()))
    }

    async fn stream(&self, messages: &[ChatMessage]) -> Result<TextStream, AiError> {
        let response = send(self.request(messages, true)).await?;
        Ok(text_stream(response, parse_line))
    }
}
