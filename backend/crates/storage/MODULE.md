# loglens-storage

SQLite persistence (SQLx, runtime queries) for app data. Logs live in Loki, not here.

- Public: `connect(db_url)` (creates file, WAL, runs migrations), `connect_in_memory()`,
  `SqlitePool`, `DEFAULT_TENANT`, `StorageError` (`Database`, `Migration`, `Conflict`, `InvalidData`).
- `users`: `User`, `find_by_username`, `create` (Conflict on duplicate), `count`.
- `severity_cache`: `CachedSeverity`, `get_many`, `put` (upsert). Severity stored lowercase.
- `alert_state`: `last_sent`, `mark_sent` (upsert).
- Migrations in `migrations/`, embedded with `sqlx::migrate!`. Every table has
  `tenant_id INTEGER NOT NULL DEFAULT 1`; every query takes a tenant id.
- Depends on: loglens-core, sqlx, chrono, thiserror.
- Env vars: none (db_url comes from `LOGLENS_DB_URL` via loglens-config in the api).
