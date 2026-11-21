# core

Shared domain types and errors used by every other crate. No I/O.

Public items:
- `LogEntry`, `Severity`, `SeveritySource`, `LogFilters`, `Direction`
- `Meta` (hosts and sources), `Stats` (counts per severity and per source)
- `LogStore` trait (query, meta, stats, tail) and `LogStream`. Implemented by `loki`.
- `CoreError`, `StoreError`

Dependencies: serde, chrono, thiserror, async-trait, futures.
Env vars: none.
