# shared/platform

Detects the native (Tauri) shell and holds the user-configured server URL.

Public items (`index.ts`):

- `isNative()` — true inside the Tauri app (`window.__TAURI_INTERNALS__` present).
- `normalizeServerUrl(raw)` — validated `http(s)://host[:port][/path]` without trailing slash, or `null`.
- `getServerUrl()` / `setServerUrl(url | null)` — localStorage `loglens.server` (errors swallowed).
- `apiBase()` — `''` on the web (same-origin `/api`), the stored server URL in the native app.

Dependencies: none. Env vars: none.
