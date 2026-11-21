import type { Severity } from '@/shared/types/severity'

export type ChatRole = 'user' | 'assistant'

export interface ChatTurn {
  role: ChatRole
  content: string
}

/** Filters as serialized by the backend query planner (optional fields may be null). */
export interface ChatFilters {
  host?: string | null
  source?: string | null
  severity?: Severity[]
  q?: string | null
  start?: string | null
  end?: string | null
  limit?: number | null
}

export type ChatEvent =
  | { type: 'filters'; filters: ChatFilters; count: number }
  | { type: 'delta'; text: string }
  | { type: 'done' }
  | { type: 'error'; message: string }

export interface ChatContext {
  filters: ChatFilters
  count: number
}

export interface ChatStream {
  messages: ChatTurn[]
  isStreaming: boolean
  error: string | null
  context: ChatContext | null
  send: (question: string) => void
  reset: () => void
}

export interface ExplainResponse {
  explanation: string
}
