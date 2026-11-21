import { describe, expect, it } from 'vitest'
import { createSseParser, parseChatEvent } from './sse'
import type { ChatEvent } from './types'

function collect(chunks: string[]): ChatEvent[] {
  const events: ChatEvent[] = []
  const parser = createSseParser((event) => events.push(event))
  chunks.forEach((chunk) => parser.push(chunk))
  parser.end()
  return events
}

describe('createSseParser', () => {
  it('parses several events delivered in one chunk', () => {
    const events = collect([
      'data: {"type":"filters","filters":{"severity":["critical"]},"count":42}\n\n' +
        'data: {"type":"delta","text":"Hello"}\n\n' +
        'data: {"type":"done"}\n\n',
    ])
    expect(events).toEqual([
      { type: 'filters', filters: { severity: ['critical'] }, count: 42 },
      { type: 'delta', text: 'Hello' },
      { type: 'done' },
    ])
  })

  it('reassembles events split mid-line and mid-JSON across chunks', () => {
    const events = collect([
      'da',
      'ta: {"type":"del',
      'ta","text":"Hel',
      'lo "}\n',
      '\ndata: {"type":"delta","text":"world"}\r\n\r',
      '\ndata: {"type":"done"}\n\n',
    ])
    expect(events).toEqual([
      { type: 'delta', text: 'Hello ' },
      { type: 'delta', text: 'world' },
      { type: 'done' },
    ])
  })

  it('flushes a final event without a trailing blank line', () => {
    expect(collect(['data: {"type":"error","message":"boom"}'])).toEqual([
      { type: 'error', message: 'boom' },
    ])
  })

  it('ignores comments, other fields and invalid payloads', () => {
    const events = collect([
      ': keep-alive\n\nevent: message\ndata: not json\n\ndata: {"type":"unknown"}\n\n',
      'id: 1\ndata: {"type":"delta","text":"ok"}\n\n',
    ])
    expect(events).toEqual([{ type: 'delta', text: 'ok' }])
  })
})

describe('parseChatEvent', () => {
  it('rejects malformed deltas', () => {
    expect(parseChatEvent('{"type":"delta","text":5}')).toBeNull()
    expect(parseChatEvent('[]')).toBeNull()
  })
})
