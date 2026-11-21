import { act, renderHook } from '@testing-library/react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import type { LogFilters } from '@/shared/types/filters'

const apiGet = vi.fn()

vi.mock('@/shared/api/client', async () => {
  const actual = await vi.importActual<typeof import('@/shared/api/client')>('@/shared/api/client')
  return {
    ...actual,
    apiGet: (...args: unknown[]) => apiGet(...args) as unknown,
    getToken: () => 'expired.jwt.token',
    wsUrl: () => 'ws://test/api/logs/tail',
  }
})

class RejectedSocket {
  static instances = 0
  onopen: (() => void) | null = null
  onmessage: ((e: MessageEvent<unknown>) => void) | null = null
  onclose: (() => void) | null = null
  constructor() {
    RejectedSocket.instances += 1
    setTimeout(() => this.onclose?.(), 0)
  }
  close() {}
}

const filters: LogFilters = { severity: ['critical'] }

describe('useTail', () => {
  beforeEach(() => {
    vi.useFakeTimers()
    RejectedSocket.instances = 0
    apiGet.mockReset()
    vi.stubGlobal('WebSocket', RejectedSocket)
  })
  afterEach(() => {
    vi.useRealTimers()
    vi.unstubAllGlobals()
  })

  it('stops reconnecting when the session is no longer valid', async () => {
    const { ApiError } = await import('@/shared/api/client')
    apiGet.mockRejectedValue(new ApiError(401, 'unauthorized'))
    const { useTail } = await import('./useTail')
    const { result } = renderHook(() => useTail(filters, true))

    await act(async () => {
      await vi.advanceTimersByTimeAsync(60_000)
    })
    expect(apiGet).toHaveBeenCalledTimes(1)
    expect(RejectedSocket.instances).toBe(1)
    expect(result.current.status).toBe('idle')
  })

  it('keeps reconnecting when the server is just unreachable', async () => {
    apiGet.mockRejectedValue(new TypeError('network down'))
    const { useTail } = await import('./useTail')
    renderHook(() => useTail(filters, true))

    await act(async () => {
      await vi.advanceTimersByTimeAsync(5_000)
    })
    expect(RejectedSocket.instances).toBeGreaterThan(1)
  })
})
