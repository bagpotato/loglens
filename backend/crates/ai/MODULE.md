# loglens-ai

Purpose: talk to an LLM (Ollama or OpenAI-compatible) and build LogLens AI features: log explanations, chat over logs, question-to-filter planning, masking of sensitive data.

Public items:
- `AiProvider` trait (`complete`, `stream`), `ChatMessage`, `Role`, `AiError`, `TextStream` (provider.rs)
- `OllamaProvider` (`POST {url}/api/chat`, NDJSON stream), `OpenAiProvider` (`POST {url}/v1/chat/completions`, SSE stream, Bearer key) in providers/
- `build_provider(&AiSettings) -> Arc<dyn AiProvider>` (factory.rs); timeouts: connect 10 s, complete 120 s, stream idle 300 s
- `Masker` (`new`, `mask`, `unmask`): IPv4/IPv6 and usernames -> `<IP_n>`, `<USER_n>`; `unmask_stream(Masker, TextStream)` restores split placeholders
- `explain` (entry + 10 lines before/after), `chat` + `ChatEvent` (SSE events), `plan_query` (question -> LogFilters, fallback last 24 h)
- Prompts are plain files in `src/prompts/*.md` exposed as `EXPLAIN_PROMPT`, `CHAT_PROMPT`, `CLASSIFY_PROMPT`, `PLAN_QUERY_PROMPT`

Dependencies: loglens-core (LogEntry, LogFilters, LogStore), loglens-config (AiSettings), reqwest, regex, futures, serde_json.

Env vars (read by loglens-config):
- `LOGLENS_AI_PROVIDER` = `ollama` (default) | `openai`
- `LOGLENS_AI_URL` base URL (default `http://localhost:11434`; `/v1` suffix optional for openai)
- `LOGLENS_AI_MODEL` (default `llama3.1:8b`), `LOGLENS_AI_KEY` (optional Bearer key)
- `LOGLENS_AI_MASK` = `true` (default) | `false`: mask IPs/usernames before sending logs to the AI
