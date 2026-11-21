use std::collections::HashMap;

use loglens_ai::{AiError, AiProvider, ChatMessage, CLASSIFY_PROMPT};
use loglens_core::Severity;
use serde::Deserialize;
use thiserror::Error;

pub const MAX_BATCH: usize = 20;

#[derive(Debug, Error)]
pub enum BatchError {
    #[error(transparent)]
    Ai(#[from] AiError),
    #[error("invalid classification JSON: {0}")]
    InvalidJson(String),
}

/// One template to classify, identified by a short string id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BatchItem {
    pub id: String,
    pub template: String,
}

#[derive(Debug, Deserialize)]
struct RawResult {
    id: serde_json::Value,
    severity: String,
    #[serde(default)]
    reason: Option<String>,
}

pub fn build_messages(items: &[BatchItem]) -> Vec<ChatMessage> {
    let user: String = items
        .iter()
        .map(|item| format!("{}: {}\n", item.id, item.template))
        .collect();
    vec![
        ChatMessage::system(CLASSIFY_PROMPT),
        ChatMessage::user(user),
    ]
}

/// Sends one batch to the provider and returns `id -> (severity, reason)`.
/// Items with unknown ids or invalid severities are dropped.
pub async fn classify_batch(
    ai: &dyn AiProvider,
    items: &[BatchItem],
) -> Result<HashMap<String, (Severity, String)>, BatchError> {
    let reply = ai.complete(&build_messages(items)).await?;
    parse_reply(&reply, items)
}

pub fn parse_reply(
    reply: &str,
    items: &[BatchItem],
) -> Result<HashMap<String, (Severity, String)>, BatchError> {
    let json = extract_array(reply).ok_or_else(|| BatchError::InvalidJson(snippet(reply)))?;
    let raw: Vec<RawResult> =
        serde_json::from_str(json).map_err(|e| BatchError::InvalidJson(e.to_string()))?;
    let mut out = HashMap::new();
    for r in raw {
        let id = match r.id {
            serde_json::Value::String(s) => s.trim().to_string(),
            serde_json::Value::Number(n) => n.to_string(),
            _ => continue,
        };
        if !items.iter().any(|i| i.id == id) {
            continue;
        }
        let Ok(severity) = r.severity.parse::<Severity>() else {
            continue;
        };
        let reason = r.reason.unwrap_or_default().trim().to_string();
        out.insert(id, (severity, reason));
    }
    Ok(out)
}

fn extract_array(reply: &str) -> Option<&str> {
    let start = reply.find('[')?;
    let end = reply.rfind(']')?;
    (end > start).then(|| &reply[start..=end])
}

fn snippet(reply: &str) -> String {
    reply.chars().take(120).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn items(n: usize) -> Vec<BatchItem> {
        (1..=n)
            .map(|i| BatchItem {
                id: i.to_string(),
                template: format!("template {i}"),
            })
            .collect()
    }

    #[test]
    fn parses_fenced_json_with_leading_text() {
        let reply = "Sure! Here you go:\n```json\n[{\"id\":\"1\",\"severity\":\"Critical\",\"reason\":\"disk failure\"},\n{\"id\":2,\"severity\":\"normal\",\"reason\":\"routine\"}]\n```";
        let out = parse_reply(reply, &items(2)).unwrap();
        assert_eq!(out["1"], (Severity::Critical, "disk failure".to_string()));
        assert_eq!(out["2"].0, Severity::Normal);
    }

    #[test]
    fn drops_invalid_severity_and_unknown_ids() {
        let reply =
            r#"[{"id":"1","severity":"bogus","reason":"x"},{"id":"9","severity":"warning"}]"#;
        let out = parse_reply(reply, &items(2)).unwrap();
        assert!(out.is_empty());
    }

    #[test]
    fn invalid_json_is_an_error() {
        assert!(parse_reply("I cannot help with that", &items(1)).is_err());
        assert!(parse_reply("[not json]", &items(1)).is_err());
    }

    #[test]
    fn request_contains_prompt_and_ids() {
        let msgs = build_messages(&items(2));
        assert_eq!(msgs[0].content, CLASSIFY_PROMPT);
        assert_eq!(msgs[1].content, "1: template 1\n2: template 2\n");
    }
}
