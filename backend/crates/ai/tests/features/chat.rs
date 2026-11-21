use std::sync::Arc;

use chrono::SecondsFormat;
use futures::StreamExt;
use loglens_ai::{chat, AiError, AiProvider, ChatEvent, ChatMessage, Role};
use loglens_core::LogStore;

use crate::fakes::{demo_entries, now, FakeProvider, FakeStore, StreamReply};

const PLAN: &str = r#"```json
{"source": "ssh", "q": "Failed password", "severity": []}
```"#;

/// Answers by citing the first and last log lines the model received.
fn citing_reply() -> StreamReply {
    Box::new(|messages: &[ChatMessage]| {
        let prompt = &messages[messages.len() - 1].content;
        let stamps: Vec<&str> = prompt
            .lines()
            .filter(|l| l.starts_with('['))
            .filter_map(|l| l.split(']').next())
            .collect();
        let (first, last) = (stamps[0], stamps[stamps.len() - 1]);
        // A masked placeholder is deliberately split across chunks.
        let (ip_head, ip_tail) = if prompt.contains("<IP_1>") {
            ("<IP", "_1>")
        } else {
            ("203.0.", "113.7")
        };
        vec![
            Ok("Yes. ".to_string()),
            Ok(format!("{} failed logins from {ip_head}", stamps.len())),
            Ok(String::new()),
            Ok(format!("{ip_tail} between {first}] and ")),
            Ok(format!("{last}].")),
        ]
    })
}

async fn collect(provider: FakeProvider, mask: bool) -> (Vec<ChatEvent>, Arc<FakeProvider>) {
    let provider = Arc::new(provider);
    let store: Arc<dyn LogStore> = FakeStore::new(demo_entries(now()));
    let dyn_provider: Arc<dyn AiProvider> = provider.clone();
    let question = "were there SSH brute force attempts today?".to_string();
    let history = vec![
        ChatMessage::user("hi"),
        ChatMessage::assistant("Hello, ask me about your logs."),
    ];
    let events = chat(dyn_provider, store, question, history, mask)
        .await
        .unwrap()
        .collect()
        .await;
    (events, provider)
}

fn answer_text(events: &[ChatEvent]) -> String {
    events
        .iter()
        .filter_map(|e| match e {
            ChatEvent::Delta { text } => Some(text.as_str()),
            _ => None,
        })
        .collect()
}

#[tokio::test]
async fn ssh_brute_force_question_streams_filters_deltas_and_done() {
    let provider = FakeProvider::new(PLAN).with_stream(citing_reply());
    let (events, provider) = collect(provider, false).await;

    match &events[0] {
        ChatEvent::Filters { filters, count } => {
            assert_eq!(filters.source.as_deref(), Some("ssh"));
            assert_eq!(filters.q.as_deref(), Some("Failed password"));
            assert!(filters.limit.unwrap() <= 300);
            assert_eq!(*count, 8);
        }
        other => panic!("expected filters first, got {other:?}"),
    }
    assert_eq!(events.last(), Some(&ChatEvent::Done));
    assert!(events.iter().all(|e| *e
        != ChatEvent::Delta {
            text: String::new()
        }));

    let first_attack = demo_entries(now())
        .into_iter()
        .filter(|e| e.message.contains("Failed password"))
        .map(|e| e.timestamp)
        .min()
        .unwrap();
    let cited = format!(
        "[{}]",
        first_attack.to_rfc3339_opts(SecondsFormat::Secs, true)
    );
    let text = answer_text(&events);
    assert!(
        text.contains(&cited),
        "answer {text:?} does not cite {cited}"
    );
    assert!(text.contains("8 failed logins from 203.0.113.7"));

    let sent = provider.last_stream();
    assert_eq!(sent[0].role, Role::System);
    assert_eq!(sent[1], ChatMessage::user("hi"));
    assert_eq!(sent[2].role, Role::Assistant);
    let last = &sent[3].content;
    assert!(last.contains("were there SSH brute force attempts today?"));
    assert!(last.contains("Failed password for invalid user admin0 from 203.0.113.7"));
    assert!(!last.contains("Accepted publickey"));
}

#[tokio::test]
async fn masked_chat_hides_ip_in_question_from_planner() {
    let provider = Arc::new(FakeProvider::new(PLAN).with_stream(citing_reply()));
    let store: Arc<dyn LogStore> = FakeStore::new(demo_entries(now()));
    let dyn_provider: Arc<dyn AiProvider> = provider.clone();
    let question = "any failed logins from 198.51.100.9 today?".to_string();
    let _events: Vec<ChatEvent> = chat(dyn_provider, store, question, Vec::new(), true)
        .await
        .unwrap()
        .collect()
        .await;
    let planner_input: String = provider
        .last_complete()
        .iter()
        .map(|m| m.content.as_str())
        .collect();
    assert!(!planner_input.contains("198.51.100.9"), "{planner_input}");
}

#[tokio::test]
async fn masked_chat_hides_ips_from_model_and_unmasks_deltas() {
    let provider = FakeProvider::new(PLAN).with_stream(citing_reply());
    let (events, provider) = collect(provider, true).await;

    let last = &provider.last_stream()[3].content;
    assert!(!last.contains("203.0.113.7"));
    assert!(last.contains("<IP_1>"));
    let text = answer_text(&events);
    assert!(text.contains("from 203.0.113.7"), "answer {text:?}");
    assert_eq!(events.last(), Some(&ChatEvent::Done));
}

#[tokio::test]
async fn zero_logs_still_answers() {
    let plan = r#"{"source": "nginx", "q": "segfault"}"#;
    let reply: StreamReply = Box::new(|_| vec![Ok("The logs are not enough.".to_string())]);
    let (events, provider) = collect(FakeProvider::new(plan).with_stream(reply), false).await;

    assert!(matches!(events[0], ChatEvent::Filters { count: 0, .. }));
    assert_eq!(answer_text(&events), "The logs are not enough.");
    assert_eq!(events.last(), Some(&ChatEvent::Done));
    assert!(provider.last_stream()[3].content.contains("no log lines"));
}

#[tokio::test]
async fn error_mid_stream_emits_error_and_ends() {
    let reply: StreamReply = Box::new(|_| {
        vec![
            Ok("Partial ".to_string()),
            Err(AiError::Http("connection reset".to_string())),
            Ok("never sent".to_string()),
        ]
    });
    let (events, _) = collect(FakeProvider::new(PLAN).with_stream(reply), false).await;

    assert_eq!(events.len(), 3);
    assert!(matches!(events[0], ChatEvent::Filters { .. }));
    assert_eq!(
        events[1],
        ChatEvent::Delta {
            text: "Partial ".to_string()
        }
    );
    match &events[2] {
        ChatEvent::Error { message } => assert!(message.contains("connection reset")),
        other => panic!("expected error, got {other:?}"),
    }
}

#[tokio::test]
async fn planner_failure_still_answers_with_last_24h() {
    let reply: StreamReply = Box::new(|_| vec![Ok("Summary.".to_string())]);
    let (events, _) = collect(FakeProvider::failing().with_stream(reply), false).await;

    match &events[0] {
        ChatEvent::Filters { filters, count } => {
            assert_eq!(filters.source, None);
            assert_eq!(*count, demo_entries(now()).len());
        }
        other => panic!("expected filters, got {other:?}"),
    }
    assert_eq!(events.last(), Some(&ChatEvent::Done));
}

#[test]
fn events_serialize_as_tagged_json() {
    let json = serde_json::to_string(&ChatEvent::Delta { text: "hi".into() }).unwrap();
    assert_eq!(json, r#"{"type":"delta","text":"hi"}"#);
    assert_eq!(
        serde_json::to_string(&ChatEvent::Done).unwrap(),
        r#"{"type":"done"}"#
    );
    let err = serde_json::to_string(&ChatEvent::Error {
        message: "x".into(),
    })
    .unwrap();
    assert_eq!(err, r#"{"type":"error","message":"x"}"#);
}
