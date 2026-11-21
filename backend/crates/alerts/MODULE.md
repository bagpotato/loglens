# alerts

Push notifications for critical logs. A Tokio task polls the `LogStore` for
`severity=critical` entries, deduplicates by message template and notifies via ntfy.

Public items:
- `Alert`, `Notifier` trait, `AlertError`
- `NtfyNotifier::new(&NtfySettings)`: POST `{url}/{topic}`, Priority urgent, Tags rotating_light
- `Dedup::new(pool, window)` / `is_due`, `record_sent` (after delivery), `should_send` (uses `storage::alert_state`; `DEFAULT_WINDOW` 10 min)
- `Poller::new(store, notifier, dedup)`, `poll_once()`, `spawn(every)` (30 s in production)

Dependencies: core, config, storage, severity (`template_hash`), reqwest, tokio, tracing.
Env vars (read by `config`, passed as `NtfySettings`): LOGLENS_NTFY_URL (unset disables alerts),
LOGLENS_NTFY_TOPIC (default `loglens-alerts`).
