import { act, renderHook, waitFor } from '@testing-library/react'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { useChatStream } from './useChatStream'

function sseResponse(chunks: string[]): Response {
  const encoder = new TextEncoder()
  const body = new ReadableStream<Uint8Array>({
    start(controller) {
      chunks.forEach((chunk) => controller.enqueue(encoder.encode(chunk)))
      controller.close()
    },
  })
  return new Response(body, { status: 200, headers: { 'Content-Type': 'text/event-stream' } })
}

afterEach(() => {
  vi.unstubAllGlobals()
})

describe('useChatStream', () => {
  it('streams deltas into the assistant message and posts history', async () => {
    const fetchMock = vi.fn(() =>
      Promise.resolve(
        sseResponse([
          'data: {"type":"filters","filters":{},"count":3}\n\ndata: {"type":"delta","text":"Two "}\n\n',
          'data: {"type":"delta","text":"attempts."}\n\ndata: {"type":"done"}\n\n',
        ]),
      ),
    )
    vi.stubGlobal('fetch', fetchMock)
    const { result } = renderHook(() => useChatStream())

    act(() => result.current.send('Any SSH brute force?'))
    await waitFor(() => expect(result.current.isStreaming).toBe(false))

    expect(result.current.messages).toEqual([
      { role: 'user', content: 'Any SSH brute force?' },
      { role: 'assistant', content: 'Two attempts.' },
    ])
    expect(result.current.context?.count).toBe(3)
    expect(result.current.error).toBeNull()
    const init = fetchMock.mock.calls[0] as unknown as [string, RequestInit]
    expect(init[0]).toBe('/api/ai/chat')
    expect(JSON.parse(String(init[1].body))).toEqual({
      question: 'Any SSH brute force?',
      history: [],
    })
  })

  it('reports error events and resets', async () => {
    vi.stubGlobal('fetch', () =>
      Promise.resolve(sseResponse(['data: {"type":"error","message":"AI offline"}\n\n'])),
    )
    const { result } = renderHook(() => useChatStream())
    act(() => result.current.send('hello'))
    await waitFor(() => expect(result.current.error).toBe('AI offline'))
    act(() => result.current.reset())
    expect(result.current.messages).toEqual([])
    expect(result.current.error).toBeNull()
  })
})
