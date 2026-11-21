# shared

Cross-feature building blocks. No feature logic; features import from here, never from another feature's internals.

- `types/` — `Severity`, `SeveritySource`, `SEVERITIES`, `SEVERITY_LABEL`, `isSeverity`, `LogEntry`, `LogsResponse`, `Meta`, `Stats`, `LogFilters` (mirror of the backend JSON contract).
- `api/client.ts` — `ApiError`, `getToken`/`setToken` (localStorage `loglens.token`), `apiGet`, `apiPost`, `apiFetchRaw`, `wsUrl`, `UNAUTHORIZED_EVENT` (`loglens:unauthorized`, fired on any 401 after the token is cleared).
- `ui/` — `Button`, `Badge`, `Drawer` (side panel on desktop, bottom sheet on mobile), `Spinner`.
- `hooks/` — `useMediaQuery(query)`, `useIsDesktop()` (min-width 1024px).

Dependencies: react. Env vars: none (API is same-origin under `/api`; Vite proxies it to `localhost:8080` in dev).
