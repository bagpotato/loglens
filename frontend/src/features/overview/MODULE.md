# features/overview

Home page: what happened in the last 24 h at a glance.

Public items (`index.ts`): `OverviewPage`.

Internals:

- `api.ts` — `fetchStats` (`GET /api/stats?range=24h`), `fetchLatestCritical` (`GET /api/logs?severity=critical&limit=10`), query hooks `useStats` (refresh 60 s) and `useLatestCritical` (refresh 30 s).
- `components/SeverityCounts` — one card per severity, links to `/logs?severity=<x>`.
- `components/TopSources` — sources ranked by event count, links to `/logs?source=<x>`.
- `components/LatestCritical` — last 10 critical entries, links to `/logs?severity=critical&host=&source=`.
- `components/AiSummary` — "Summarize last 24 h" sends a fixed question through `useChatStream` and renders the streamed markdown.

Dependencies: `@/features/ai` (`useChatStream`), `@/shared/*`, TanStack Query, react-router, react-markdown.
Env vars: none.
