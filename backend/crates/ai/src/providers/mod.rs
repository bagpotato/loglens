mod ollama;
mod openai;

use std::time::Duration;

use futures::stream::{self, StreamExt};
use reqwest::{Client, RequestBuilder, Response};

use crate::stream_lines::{lines, LineStream};
use crate::{AiError, TextStream};

pub use ollama::OllamaProvider;
pub use openai::OpenAiProvider;

const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
/// Idle limit between two reads; streams have no total timeout.
const READ_TIMEOUT: Duration = Duration::from_secs(300);
pub(crate) const COMPLETE_TIMEOUT: Duration = Duration::from_secs(120);
const MAX_ERROR_BODY: usize = 2000;

pub(crate) fn http_client() -> Client {
    Client::builder()
        .connect_timeout(CONNECT_TIMEOUT)
        .read_timeout(READ_TIMEOUT)
        .build()
        .unwrap_or_else(|_| Client::new())
}

/// Attempts per request when the provider is temporarily unavailable (429/502/503/504).
const MAX_ATTEMPTS: u32 = 3;
const BACKOFF: [Duration; 2] = [Duration::from_millis(500), Duration::from_millis(1500)];
const MAX_RETRY_AFTER: Duration = Duration::from_secs(10);

/// Sends the request, retrying temporary overloads, and turns transport failures and
/// non-2xx answers into `AiError`.
pub(crate) async fn send(mut request: RequestBuilder) -> Result<Response, AiError> {
    let mut attempt = 0;
    loop {
        attempt += 1;
        // A copy for the next attempt; JSON bodies are always clonable.
        let next = (attempt < MAX_ATTEMPTS)
            .then(|| request.try_clone())
            .flatten();
        let response = request
            .send()
            .await
            .map_err(|e| AiError::Http(e.to_string()))?;
        let status = response.status();
        if status.is_success() {
            return Ok(response);
        }
        let transient = matches!(status.as_u16(), 429 | 502 | 503 | 504);
        match next {
            Some(copy) if transient => {
                let wait = retry_after(&response).unwrap_or(BACKOFF[(attempt - 1) as usize]);
                tracing::warn!(
                    status = status.as_u16(),
                    attempt,
                    "AI provider busy, retrying"
                );
                tokio::time::sleep(wait).await;
                request = copy;
            }
            _ => return Err(status_error(response).await),
        }
    }
}

fn retry_after(response: &Response) -> Option<Duration> {
    let secs: u64 = response
        .headers()
        .get(reqwest::header::RETRY_AFTER)?
        .to_str()
        .ok()?
        .trim()
        .parse()
        .ok()?;
    Some(Duration::from_secs(secs).min(MAX_RETRY_AFTER))
}

async fn status_error(response: Response) -> AiError {
    let status = response.status();
    let mut body = response.text().await.unwrap_or_default();
    if body.len() > MAX_ERROR_BODY {
        let mut cut = MAX_ERROR_BODY;
        while !body.is_char_boundary(cut) {
            cut -= 1;
        }
        body.truncate(cut);
    }
    AiError::Status {
        status: status.as_u16(),
        body,
    }
}

pub(crate) async fn json_body(response: Response) -> Result<serde_json::Value, AiError> {
    response
        .json()
        .await
        .map_err(|e| AiError::InvalidResponse(e.to_string()))
}

/// What one line of a streaming response means.
pub(crate) enum Step {
    Skip,
    Text(String),
    Done(Option<String>),
}

/// Maps the response body line by line with `parse` until it reports `Done` or the body ends.
pub(crate) fn text_stream<F>(response: Response, parse: F) -> TextStream
where
    F: FnMut(&str) -> Result<Step, AiError> + Send + 'static,
{
    let source: LineStream = lines(response.bytes_stream());
    stream::unfold(
        (source, parse, false),
        |(mut source, mut parse, done)| async move {
            if done {
                return None;
            }
            loop {
                let line = match source.next().await? {
                    Ok(line) => line,
                    Err(e) => return Some((Err(e), (source, parse, true))),
                };
                if line.trim().is_empty() {
                    continue;
                }
                match parse(&line) {
                    Ok(Step::Skip) => continue,
                    Ok(Step::Text(text)) if text.is_empty() => continue,
                    Ok(Step::Text(text)) => return Some((Ok(text), (source, parse, false))),
                    Ok(Step::Done(Some(text))) if !text.is_empty() => {
                        return Some((Ok(text), (source, parse, true)))
                    }
                    Ok(Step::Done(_)) => return None,
                    Err(e) => return Some((Err(e), (source, parse, true))),
                }
            }
        },
    )
    .boxed()
}
