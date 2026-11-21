# features/logs

Log browser: filters, severity chips, virtualized newest-first list, live tail and a detail drawer.

Public (`index.ts`): `LogsPage` (route `/logs`).

URL state (`useLogFilters`): `host`, `source`, `severity` (comma list; default `critical,important` when absent),
`q`, `start`, `end` (RFC3339), `live=1`, `focus=<RFC3339>` (scrolls to and highlights the closest entry).

Internals: `api.ts` (`GET /api/logs`, `GET /api/meta`), `useLogs.ts` (TanStack Query), `useTail.ts`
(`WS /api/logs/tail`, subprotocols `bearer, <jwt>`, backoff reconnect, cap 10,000), `entries.ts` (merge/cap/focus helpers),
`components/` (FilterBar, SearchInput, TimeRangePicker, SeverityChips, LogList, LogRow, LogDetail).

Dependencies: `@/shared/*`, `@/features/ai` (ExplainButton, ExplainResult), @tanstack/react-query, @tanstack/react-virtual, react-router.
Env vars: none.
