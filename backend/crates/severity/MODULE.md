# loglens-severity

Purpose: assign a severity to log entries that no ingest rule matched, using message templates,
a SQLite cache and batched AI classification.

Public items:
- `normalize(&str) -> String`: replaces UUIDs, IPs (with port), hex, paths, emails and numbers with placeholders.
- `template_hash(&str) -> String`: first 16 hex chars of sha256(normalize(message)).
- `Classifier::new(pool, ai)`, `.with_timeout(Duration)` (default 8 s), `async enrich(&mut [LogEntry])`.
  Rule entries untouched; cache hits and AI verdicts set source `ai` + reason; failures leave entries as is.
  At most one in-flight AI request per template; batches of 20; on timeout the AI work keeps running
  in the background and its results are cached for the next call.
- `BatchError`: error of one AI batch (internal, logged, never returned by `enrich`).

Dependencies: loglens-core, loglens-storage (`severity_cache`), loglens-ai (`AiProvider`, `CLASSIFY_PROMPT`),
regex, sha2, serde_json, futures, tokio, tracing.

Env vars: none.
