# config

Reads all `LOGLENS_*` environment variables once at startup.

Public items: `Settings::from_env()`, `Settings::from_lookup()`, `AiSettings`,
`AiProviderKind`, `NtfySettings`, `ConfigError`, `Settings::cors_origins_from_env()`,
`Settings::DEFAULT_CORS_ORIGINS`.

Dependencies: thiserror.
Env vars: LOGLENS_BIND, LOGLENS_LOKI_URL, LOGLENS_DB_URL, LOGLENS_ADMIN_USER,
LOGLENS_ADMIN_PASSWORD, LOGLENS_JWT_SECRET, LOGLENS_AI_PROVIDER, LOGLENS_AI_URL,
LOGLENS_AI_MODEL, LOGLENS_AI_KEY, LOGLENS_AI_MASK, LOGLENS_NTFY_URL, LOGLENS_NTFY_TOPIC,
LOGLENS_CORS_ORIGINS (comma list, default `http://tauri.localhost,https://tauri.localhost,tauri://localhost`).
Only `api` calls `from_env`; other crates receive the sub-structs.
