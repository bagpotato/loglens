# loglens-api
HTTP/WebSocket/SSE server (axum). Binary wires config, storage, auth, Loki, AI, classifier and alerts (ntfy poller every 30 s, 10 min dedup).
Public: `AppState`, `build_router`, `ApiError` (`{"error": ...}`; store/AI errors -> 502), `LogQuery`, `StatsQuery`, `parse_range`.
Depends on: core, config, storage, auth, loki, ai, severity, alerts.
Env (via `Settings::from_env`): LOGLENS_BIND, LOGLENS_LOKI_URL, LOGLENS_DB_URL, LOGLENS_ADMIN_USER, LOGLENS_ADMIN_PASSWORD, LOGLENS_JWT_SECRET,
LOGLENS_AI_PROVIDER, LOGLENS_AI_URL, LOGLENS_AI_MODEL, LOGLENS_AI_KEY, LOGLENS_AI_MASK, LOGLENS_NTFY_URL, LOGLENS_NTFY_TOPIC, LOGLENS_CORS_ORIGINS; RUST_LOG (default `info`).
CORS (outermost layer): listed origins (native app WebView; `*` = any) may GET/POST with Authorization + Content-Type.

| Route | Auth | Purpose |
|---|---|---|
| GET /api/health | no | `{"status":"ok"}` |
| POST /api/auth/login | no | `{username,password}` -> `{token,username,role}` |
| GET /api/logs | yes | host, source, severity (comma list), q, start/end (RFC3339), limit -> `{"entries"}` |
| WS /api/logs/tail | yes (subprotocol `bearer, <jwt>`) | live enriched LogEntry JSON per text frame |
| GET /api/meta | yes | `{hosts, sources}` |
| GET /api/stats | yes | `range=<n>m\|h\|d` (default 24h, max 30d) |
| POST /api/ai/explain | yes | `{entry}` -> `{"explanation"}` |
| POST /api/ai/chat | yes | `{question, history}` -> SSE of ChatEvent JSON |
