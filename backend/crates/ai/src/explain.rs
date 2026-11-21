use chrono::Duration;
use loglens_core::{Direction, LogEntry, LogFilters, LogStore};

use crate::masking::Masker;
use crate::prompts::EXPLAIN_PROMPT;
use crate::{AiError, AiProvider, ChatMessage};

/// Lines of context taken before and after the explained entry.
pub const CONTEXT_LINES: usize = 10;

const CONTEXT_WINDOW_HOURS: i64 = 1;

/// Explains one entry using surrounding lines from the same host and source. Returns Markdown.
pub async fn explain(
    provider: &dyn AiProvider,
    store: &dyn LogStore,
    entry: &LogEntry,
    mask: bool,
) -> Result<String, AiError> {
    let (before, after) = context(store, entry).await?;
    let mut masker = Masker::new(mask);
    let prompt = explain_input(&mut masker, entry, &before, &after);
    let messages = [
        ChatMessage::system(EXPLAIN_PROMPT),
        ChatMessage::user(prompt),
    ];
    let answer = provider.complete(&messages).await?;
    Ok(masker.unmask(&answer))
}

async fn context(
    store: &dyn LogStore,
    entry: &LogEntry,
) -> Result<(Vec<LogEntry>, Vec<LogEntry>), AiError> {
    let window = Duration::hours(CONTEXT_WINDOW_HOURS);
    let base = LogFilters {
        host: Some(entry.host.clone()),
        source: Some(entry.source.clone()),
        limit: Some(context_query_limit()),
        ..Default::default()
    };
    let before_filters = LogFilters {
        start: Some(entry.timestamp - window),
        end: Some(entry.timestamp),
        direction: Direction::Backward,
        ..base.clone()
    };
    let after_filters = LogFilters {
        start: Some(entry.timestamp),
        end: Some(entry.timestamp + window),
        direction: Direction::Forward,
        ..base
    };
    let before_raw = store.query(&before_filters).await.map_err(store_error)?;
    let after_raw = store.query(&after_filters).await.map_err(store_error)?;

    let mut before: Vec<LogEntry> = before_raw
        .into_iter()
        .filter(|e| e.id != entry.id && e.timestamp <= entry.timestamp)
        .take(CONTEXT_LINES)
        .collect();
    before.reverse();
    let after = after_raw
        .into_iter()
        .filter(|e| e.id != entry.id && e.timestamp >= entry.timestamp)
        .filter(|e| !before.iter().any(|b| b.id == e.id))
        .take(CONTEXT_LINES)
        .collect();
    Ok((before, after))
}

/// One extra line so the explained entry itself can be dropped from the results.
fn context_query_limit() -> u32 {
    u32::try_from(CONTEXT_LINES + 1).unwrap_or(u32::MAX)
}

fn store_error(err: loglens_core::StoreError) -> AiError {
    AiError::Store(err.to_string())
}

fn explain_input(
    masker: &mut Masker,
    entry: &LogEntry,
    before: &[LogEntry],
    after: &[LogEntry],
) -> String {
    let mut text = String::from("Log entry to explain:\n");
    text.push_str(&masker.mask(&entry.to_prompt_line()));
    text.push_str("\n\nLines before it (oldest first):\n");
    push_lines(&mut text, masker, before);
    text.push_str("\nLines after it (oldest first):\n");
    push_lines(&mut text, masker, after);
    text
}

fn push_lines(text: &mut String, masker: &mut Masker, entries: &[LogEntry]) {
    if entries.is_empty() {
        text.push_str("(none)\n");
    }
    for entry in entries {
        text.push_str(&masker.mask(&entry.to_prompt_line()));
        text.push('\n');
    }
}
