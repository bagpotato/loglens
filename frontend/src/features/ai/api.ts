import { apiFetchRaw, apiPost } from '@/shared/api/client'
import type { LogEntry } from '@/shared/types/log'
import { createSseParser } from './sse'
import type { ChatEvent, ChatTurn, ExplainResponse } from './types'

export function explainEntry(entry: LogEntry): Promise<ExplainResponse> {
  return apiPost<ExplainResponse>('/api/ai/explain', { entry })
}

export async function streamChat(
  question: string,
  history: ChatTurn[],
  signal: AbortSignal,
  onEvent: (event: ChatEvent) => void,
): Promise<void> {
  const response = await apiFetchRaw('/api/ai/chat', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json', Accept: 'text/event-stream' },
    body: JSON.stringify({ question, history }),
    signal,
  })
  if (!response.body) throw new Error('Streaming responses are not supported by this browser')
  const reader = response.body.getReader()
  const decoder = new TextDecoder()
  const parser = createSseParser(onEvent)
  for (;;) {
    const { done, value } = await reader.read()
    if (done) break
    parser.push(decoder.decode(value, { stream: true }))
  }
  parser.push(decoder.decode())
  parser.end()
}
