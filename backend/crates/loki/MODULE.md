# loki

`LogStore` implementation backed by Grafana Loki (HTTP + WebSocket tail).

Public items:
- `LokiStore::new(base_url)` implementing `loglens_core::LogStore`
- `build_query`, `build_selector` (LogFilters -> LogQL)
- `parse_entry(labels, ts_ns, line)` (Loki value -> `LogEntry`)

Endpoints: `/loki/api/v1/query_range`, `/loki/api/v1/query`,
`/loki/api/v1/label/{host,source}/values`, `/loki/api/v1/tail` (ws/wss).
Query default range: last 24 h. Errors map to `StoreError`.

Dependencies: core, reqwest, tokio-tungstenite, serde_json, sha2, regex, chrono.
Env vars: none read here. `api` passes `LOGLENS_LOKI_URL` (via config) to `LokiStore::new`.
