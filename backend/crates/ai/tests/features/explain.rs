use chrono::Duration;
use loglens_ai::{explain, CONTEXT_LINES, EXPLAIN_PROMPT};
use loglens_core::{Direction, LogEntry};

use crate::fakes::{entry, now, FakeProvider, FakeStore};

/// 31 sshd lines one minute apart on web1 (target is the middle one) plus noise elsewhere.
fn timeline() -> (Vec<LogEntry>, LogEntry) {
    let base = now() - Duration::hours(2);
    let mut entries: Vec<LogEntry> = (0..31)
        .map(|i| {
            entry(
                &format!("e{i}"),
                base + Duration::minutes(i),
                "web1",
                "ssh",
                &format!("line {i} from 10.0.0.{i}"),
            )
        })
        .collect();
    let target = entry(
        "e15",
        base + Duration::minutes(15),
        "web1",
        "ssh",
        "Failed password for root from 203.0.113.7 port 22 ssh2",
    );
    entries[15] = target.clone();
    entries.push(entry("x1", target.timestamp, "web2", "ssh", "other host"));
    entries.push(entry(
        "x2",
        target.timestamp,
        "web1",
        "nginx",
        "other source",
    ));
    (entries, target)
}

#[tokio::test]
async fn sends_entry_and_ten_lines_each_side() {
    let (entries, target) = timeline();
    let store = FakeStore::new(entries);
    let provider = FakeProvider::new("## What happened\nok");

    let answer = explain(&provider, store.as_ref(), &target, false)
        .await
        .unwrap();
    assert!(answer.contains("What happened"));

    let sent = provider.last_complete();
    assert_eq!(sent[0].content, EXPLAIN_PROMPT);
    let prompt = &sent[1].content;
    assert!(prompt.contains(&target.to_prompt_line()));
    for i in (15 - CONTEXT_LINES)..15 {
        assert!(prompt.contains(&format!("line {i} ")), "missing before {i}");
    }
    for i in 16..=(15 + CONTEXT_LINES) {
        assert!(prompt.contains(&format!("line {i} ")), "missing after {i}");
    }
    assert!(!prompt.contains("line 4 "));
    assert!(!prompt.contains("line 26 "));
    assert!(!prompt.contains("other host"));
    assert!(!prompt.contains("other source"));
    let oldest = prompt.find("line 5 ").unwrap();
    let newest_before = prompt.find("line 14 ").unwrap();
    let target_pos = prompt.find(&target.message).unwrap();
    let first_after = prompt.find("line 16 ").unwrap();
    assert!(target_pos < oldest && oldest < newest_before && newest_before < first_after);

    let queries = store.queries.lock().unwrap().clone();
    assert_eq!(queries.len(), 2);
    assert_eq!(queries[0].direction, Direction::Backward);
    assert_eq!(queries[1].direction, Direction::Forward);
    for q in &queries {
        assert_eq!(q.host.as_deref(), Some("web1"));
        assert_eq!(q.source.as_deref(), Some("ssh"));
        assert!(q.end.unwrap() - q.start.unwrap() <= Duration::hours(1));
    }
}

#[tokio::test]
async fn masks_ips_when_enabled_and_unmasks_answer() {
    let (entries, target) = timeline();
    let store = FakeStore::new(entries);
    let provider = FakeProvider::new("Block <IP_1> with:\n```bash\nsudo ufw deny from <IP_1>\n```");

    let answer = explain(&provider, store.as_ref(), &target, true)
        .await
        .unwrap();

    let prompt = &provider.last_complete()[1].content;
    assert!(!prompt.contains("203.0.113.7"));
    assert!(!prompt.contains("10.0.0.14"));
    assert!(prompt.contains("<IP_1>"));
    assert!(answer.contains("sudo ufw deny from 203.0.113.7"));
    assert!(!answer.contains("<IP_1>"));
}

#[tokio::test]
async fn keeps_ips_when_masking_disabled() {
    let (entries, target) = timeline();
    let store = FakeStore::new(entries);
    let provider = FakeProvider::new("fine");

    explain(&provider, store.as_ref(), &target, false)
        .await
        .unwrap();
    assert!(provider.last_complete()[1].content.contains("203.0.113.7"));
}

#[tokio::test]
async fn provider_error_is_returned() {
    let (entries, target) = timeline();
    let store = FakeStore::new(entries);
    let result = explain(&FakeProvider::failing(), store.as_ref(), &target, false).await;
    assert!(result.is_err());
}
