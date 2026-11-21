import type { ChatEvent, ChatFilters } from './types'

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value)
}

export function parseChatEvent(data: string): ChatEvent | null {
  let value: unknown
  try {
    value = JSON.parse(data)
  } catch {
    return null
  }
  if (!isRecord(value)) return null
  switch (value.type) {
    case 'delta':
      return typeof value.text === 'string' ? { type: 'delta', text: value.text } : null
    case 'done':
      return { type: 'done' }
    case 'error':
      return {
        type: 'error',
        message: typeof value.message === 'string' ? value.message : 'Unknown error',
      }
    case 'filters':
      return {
        type: 'filters',
        filters: isRecord(value.filters) ? (value.filters as ChatFilters) : {},
        count: typeof value.count === 'number' ? value.count : 0,
      }
    default:
      return null
  }
}

export interface SseParser {
  push: (chunk: string) => void
  end: () => void
}

/** Incremental `text/event-stream` parser; chunks may split lines or hold several events. */
export function createSseParser(onEvent: (event: ChatEvent) => void): SseParser {
  let buffer = ''
  let dataLines: string[] = []

  const dispatch = () => {
    if (dataLines.length === 0) return
    const event = parseChatEvent(dataLines.join('\n'))
    dataLines = []
    if (event) onEvent(event)
  }

  const handleLine = (raw: string) => {
    const line = raw.endsWith('\r') ? raw.slice(0, -1) : raw
    if (line === '') {
      dispatch()
      return
    }
    if (!line.startsWith('data:')) return
    const value = line.slice(5)
    dataLines.push(value.startsWith(' ') ? value.slice(1) : value)
  }

  return {
    push(chunk: string) {
      buffer += chunk
      let newline = buffer.indexOf('\n')
      while (newline >= 0) {
        handleLine(buffer.slice(0, newline))
        buffer = buffer.slice(newline + 1)
        newline = buffer.indexOf('\n')
      }
    },
    end() {
      if (buffer !== '') handleLine(buffer)
      buffer = ''
      dispatch()
    },
  }
}
