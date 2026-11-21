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

/// Sends the request and turns transport failures and non-2xx answers into `AiError`.
pub(crate) async fn send(request: RequestBuilder) -> Result<Response, AiError> {
    let response = request
        .send()
        .await
        .map_err(|e| AiError::Http(e.to_string()))?;
    let status = response.status();
    if status.is_success() {
        return Ok(response);
    }
    let mut body = response.text().await.unwrap_or_default();
    if body.len() > MAX_ERROR_BODY {
        let mut cut = MAX_ERROR_BODY;
        while !body.is_char_boundary(cut) {
            cut -= 1;
        }
        body.truncate(cut);
    }
    Err(AiError::Status {
        status: status.as_u16(),
        body,
    })
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
