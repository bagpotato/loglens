# features/ai

AI explanations of single log entries and a streaming chat over the logs.

Public (`index.ts`):

- `ExplainButton({ entry, onResult })` calls `POST /api/ai/explain`; `ExplainResult({ markdown })` renders it (code blocks with Copy).
- `ChatPanel({ onClose? })` side panel on desktop, full screen on mobile; `ChatPage` full-screen route wrapper.
- `useChatStream()` -> `{ messages, isStreaming, error, context, send, reset }` (`POST /api/ai/chat`, SSE `data:` lines -> `ChatEvent`).

Internals: `api.ts`, `sse.ts` (incremental SSE parser), `linkify.ts` (`[RFC3339]` citations -> `/logs?start&end&severity&focus` links), `types.ts`, `components/`.

Dependencies: `@/shared/*`, react-markdown, react-router. Env vars: none.
