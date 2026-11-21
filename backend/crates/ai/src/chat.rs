use std::sync::Arc;

use chrono::{DateTime, SecondsFormat, Utc};
use futures::future::ready;
use futures::stream::{self, BoxStream, StreamExt};
use loglens_core::{LogEntry, LogFilters, LogStore};
use serde::Serialize;

use crate::masking::{unmask_stream, Masker};
use crate::prompts::CHAT_PROMPT;
use crate::query_planner::{plan_query, MAX_CHAT_LOGS};
use crate::{AiError, AiProvider, ChatMessage, TextStream};

/// Events of a chat answer, sent to the frontend as SSE `data:` lines.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ChatEvent {
    Filters { filters: LogFilters, count: usize },
    Delta { text: String },
    Done,
    Error { message: String },
}

/// Plans filters, fetches up to 300 logs and streams an answer citing their timestamps.
/// Planning and log fetching errors are returned; errors while answering become an `Error` event.
pub async fn chat(
    provider: Arc<dyn AiProvider>,
    store: Arc<dyn LogStore>,
    question: String,
    history: Vec<ChatMessage>,
    mask: bool,
) -> Result<BoxStream<'static, ChatEvent>, AiError> {
    let now = Utc::now();
    let meta = store.meta().await.map_err(store_error)?;
    let mut masker = Masker::new(mask);
    let masked_question = masker.mask(&question);
    let mut filters = plan_query(provider.as_ref(), &masked_question, &meta, now).await;
    filters.q = filters.q.map(|q| masker.unmask(&q));
    filters.limit = Some(filters.limit.unwrap_or(MAX_CHAT_LOGS).min(MAX_CHAT_LOGS));
    let mut entries = store.query(&filters).await.map_err(store_error)?;
    entries.sort_by_key(|e| e.timestamp);

    let messages = build_messages(&mut masker, &question, &history, &filters, &entries, now);
    let head = stream::once(ready(ChatEvent::Filters {
        filters,
        count: entries.len(),
    }));
    let answer = async move {
        match provider.stream(&messages).await {
            Ok(text) => answer_events(unmask_stream(masker, text)),
            Err(err) => stream::once(ready(error_event(&err))).boxed(),
        }
    };
    Ok(head.chain(stream::once(answer).flatten()).boxed())
}

fn store_error(err: loglens_core::StoreError) -> AiError {
    AiError::Store(err.to_string())
}

fn error_event(err: &AiError) -> ChatEvent {
    ChatEvent::Error {
        message: err.to_string(),
    }
}

/// Deltas followed by `Done`; the first error ends the stream right after its `Error` event.
fn answer_events(text: TextStream) -> BoxStream<'static, ChatEvent> {
    text.filter_map(|chunk| {
        ready(match chunk {
            Ok(text) if text.is_empty() => None,
            Ok(text) => Some(ChatEvent::Delta { text }),
            Err(err) => Some(error_event(&err)),
        })
    })
    .chain(stream::once(ready(ChatEvent::Done)))
    .scan(false, |failed, event| {
        if *failed {
            return ready(None);
        }
        *failed = matches!(event, ChatEvent::Error { .. });
        ready(Some(event))
    })
    .boxed()
}

fn build_messages(
    masker: &mut Masker,
    question: &str,
    history: &[ChatMessage],
    filters: &LogFilters,
    entries: &[LogEntry],
    now: DateTime<Utc>,
) -> Vec<ChatMessage> {
    let system = format!(
        "{CHAT_PROMPT}\nCurrent time: {}",
        now.to_rfc3339_opts(SecondsFormat::Secs, true)
    );
    let mut messages = vec![ChatMessage::system(system)];
    messages.extend(history.iter().map(|m| ChatMessage {
        role: m.role,
        content: masker.mask(&m.content),
    }));
    messages.push(ChatMessage::user(logs_input(
        masker, question, filters, entries,
    )));
    messages
}

fn logs_input(
    masker: &mut Masker,
    question: &str,
    filters: &LogFilters,
    entries: &[LogEntry],
) -> String {
    let filters_json = serde_json::to_string(filters).unwrap_or_default();
    let mut text = format!(
        "Filters used: {}\nLog lines ({}, oldest first):\n",
        masker.mask(&filters_json),
        entries.len()
    );
    if entries.is_empty() {
        text.push_str("(no log lines matched)\n");
    }
    for entry in entries {
        text.push_str(&masker.mask(&entry.to_prompt_line()));
        text.push('\n');
    }
    text.push_str("\nQuestion: ");
    text.push_str(&masker.mask(question));
    text
}
