use chrono::{DateTime, Duration, SecondsFormat, Utc};
use loglens_core::{LogFilters, Meta, Severity};
use serde_json::Value;

use crate::prompts::PLAN_QUERY_PROMPT;
use crate::{AiProvider, ChatMessage};

/// Upper bound of log entries sent to the model in one chat answer.
pub const MAX_CHAT_LOGS: u32 = 300;

const MAX_QUERY_LEN: usize = 200;

/// Turns a question into log filters. Never fails: any problem falls back to the last 24 h.
pub async fn plan_query(
    provider: &dyn AiProvider,
    question: &str,
    meta: &Meta,
    now: DateTime<Utc>,
) -> LogFilters {
    let messages = [
        ChatMessage::system(PLAN_QUERY_PROMPT),
        ChatMessage::user(planner_input(question, meta, now)),
    ];
    match provider.complete(&messages).await {
        Ok(answer) => parse_plan(&answer, meta, now),
        Err(err) => {
            tracing::warn!(error = %err, "query planning failed, using last 24 h");
            fallback_filters(now)
        }
    }
}

/// Last 24 hours, no other filters.
pub fn fallback_filters(now: DateTime<Utc>) -> LogFilters {
    LogFilters {
        start: Some(now - Duration::hours(24)),
        end: Some(now),
        limit: Some(MAX_CHAT_LOGS),
        ..Default::default()
    }
}

fn planner_input(question: &str, meta: &Meta, now: DateTime<Utc>) -> String {
    let severities: Vec<&str> = Severity::ALL.iter().map(|s| s.as_str()).collect();
    format!(
        "Current time: {}\nKnown hosts: {}\nKnown sources: {}\nAllowed severities: {}\n\nQuestion: {}",
        now.to_rfc3339_opts(SecondsFormat::Secs, true),
        list_or_none(&meta.hosts),
        list_or_none(&meta.sources),
        severities.join(", "),
        question
    )
}

fn list_or_none(values: &[String]) -> String {
    if values.is_empty() {
        "(none)".to_string()
    } else {
        values.join(", ")
    }
}

/// Parses and validates the model answer; falls back to the last 24 h when it is not usable.
pub fn parse_plan(answer: &str, meta: &Meta, now: DateTime<Utc>) -> LogFilters {
    match extract_json_object(answer) {
        Some(value) => validate(&value, meta, now),
        None => {
            tracing::warn!("query planner answer is not JSON, using last 24 h");
            fallback_filters(now)
        }
    }
}

/// Finds the first JSON object in the text, tolerating code fences and surrounding prose.
fn extract_json_object(text: &str) -> Option<Value> {
    let mut search_from = 0;
    while let Some(offset) = text[search_from..].find('{') {
        let start = search_from + offset;
        let mut stream = serde_json::Deserializer::from_str(&text[start..]).into_iter::<Value>();
        if let Some(Ok(value)) = stream.next() {
            if value.is_object() {
                return Some(value);
            }
        }
        search_from = start + 1;
    }
    None
}

fn validate(value: &Value, meta: &Meta, now: DateTime<Utc>) -> LogFilters {
    let (start, end) = time_range(value, now);
    LogFilters {
        host: known_value(value, "host", &meta.hosts),
        source: known_value(value, "source", &meta.sources),
        severity: severities(value),
        q: text_field(value, "q").filter(|q| q.chars().count() <= MAX_QUERY_LEN),
        start: Some(start),
        end: Some(end),
        limit: Some(limit(value)),
        ..Default::default()
    }
}

fn text_field(value: &Value, key: &str) -> Option<String> {
    let text = value.get(key)?.as_str()?.trim();
    let empty = text.is_empty() || text.eq_ignore_ascii_case("null");
    (!empty).then(|| text.to_string())
}

fn known_value(value: &Value, key: &str, known: &[String]) -> Option<String> {
    let wanted = text_field(value, key)?;
    known
        .iter()
        .find(|k| k.eq_ignore_ascii_case(&wanted))
        .cloned()
}

fn severities(value: &Value) -> Vec<Severity> {
    let raw: Vec<&str> = match value.get("severity") {
        Some(Value::Array(items)) => items.iter().filter_map(Value::as_str).collect(),
        Some(Value::String(one)) => one.split(',').collect(),
        _ => Vec::new(),
    };
    let mut parsed: Vec<Severity> = raw.iter().filter_map(|s| s.parse().ok()).collect();
    parsed.sort();
    parsed.dedup();
    if parsed.len() == Severity::ALL.len() {
        Vec::new()
    } else {
        parsed
    }
}

fn timestamp(value: &Value, key: &str) -> Option<DateTime<Utc>> {
    let text = text_field(value, key)?;
    DateTime::parse_from_rfc3339(&text)
        .ok()
        .map(|t| t.with_timezone(&Utc))
}

fn time_range(value: &Value, now: DateTime<Utc>) -> (DateTime<Utc>, DateTime<Utc>) {
    let end = timestamp(value, "end").unwrap_or(now);
    let start = timestamp(value, "start").unwrap_or(end - Duration::hours(24));
    if start < end {
        (start, end)
    } else {
        (now - Duration::hours(24), now)
    }
}

fn limit(value: &Value) -> u32 {
    value
        .get("limit")
        .and_then(Value::as_u64)
        .map_or(MAX_CHAT_LOGS, |l| {
            u32::try_from(l)
                .unwrap_or(MAX_CHAT_LOGS)
                .clamp(1, MAX_CHAT_LOGS)
        })
}
