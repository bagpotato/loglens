import { afterEach, describe, expect, it, vi } from 'vitest'
import {
  ApiError,
  UNAUTHORIZED_EVENT,
  apiFetchRaw,
  apiGet,
  apiPost,
  getToken,
  setToken,
  wsUrl,
} from './client'
import { setServerUrl } from '@/shared/platform'

function jsonResponse(body: unknown, status = 200): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: { 'Content-Type': 'application/json' },
  })
}

function mockFetch(response: Response) {
  const fn = vi.fn<typeof fetch>().mockImplementation(() => Promise.resolve(response.clone()))
  vi.stubGlobal('fetch', fn)
  return fn
}

function sentHeaders(fn: ReturnType<typeof mockFetch>): Headers {
  return new Headers(fn.mock.calls[0][1]?.headers)
}

afterEach(() => {
  vi.unstubAllGlobals()
})

describe('token storage', () => {
  it('stores, reads and clears the token under loglens.token', () => {
    setToken('abc')
    expect(localStorage.getItem('loglens.token')).toBe('abc')
    expect(getToken()).toBe('abc')
    setToken(null)
    expect(getToken()).toBeNull()
  })
})

describe('apiGet', () => {
  it('adds the bearer token and drops empty params', async () => {
    setToken('jwt-1')
    const fn = mockFetch(jsonResponse({ entries: [] }))
    const result = await apiGet<{ entries: unknown[] }>('/api/logs', {
      severity: 'critical,important',
      host: undefined,
      q: '',
    })
    expect(result).toEqual({ entries: [] })
    expect(fn.mock.calls[0][0]).toBe('/api/logs?severity=critical%2Cimportant')
    expect(sentHeaders(fn).get('Authorization')).toBe('Bearer jwt-1')
  })

  it('sends no Authorization header without a token', async () => {
    const fn = mockFetch(jsonResponse({ status: 'ok' }))
    await apiGet('/api/health')
    expect(sentHeaders(fn).has('Authorization')).toBe(false)
  })
})

describe('apiPost', () => {
  it('sends a JSON body', async () => {
    const fn = mockFetch(jsonResponse({ ok: true }))
    await apiPost('/api/ai/explain', { entry: { id: '1' } })
    const init = fn.mock.calls[0][1]
    expect(init?.method).toBe('POST')
    expect(init?.body).toBe('{"entry":{"id":"1"}}')
    expect(sentHeaders(fn).get('Content-Type')).toBe('application/json')
  })
})

describe('errors', () => {
  it('throws ApiError with status and server message', async () => {
    mockFetch(jsonResponse({ error: 'loki unavailable' }, 502))
    const error = await apiGet('/api/logs').catch((e: unknown) => e)
    expect(error).toBeInstanceOf(ApiError)
    expect(error).toMatchObject({ status: 502, message: 'loki unavailable' })
  })

  it('falls back to the status text for non-JSON bodies', async () => {
    mockFetch(new Response('boom', { status: 500, statusText: 'Internal Server Error' }))
    await expect(apiFetchRaw('/api/x')).rejects.toMatchObject({
      status: 500,
      message: 'Internal Server Error',
    })
  })

  it('on 401 clears the token and dispatches loglens:unauthorized', async () => {
    setToken('expired')
    const listener = vi.fn()
    window.addEventListener(UNAUTHORIZED_EVENT, listener)
    mockFetch(jsonResponse({ error: 'unauthorized' }, 401))
    await expect(apiGet('/api/meta')).rejects.toMatchObject({ status: 401 })
    expect(getToken()).toBeNull()
    expect(listener).toHaveBeenCalledTimes(1)
    window.removeEventListener(UNAUTHORIZED_EVENT, listener)
  })
})

describe('wsUrl', () => {
  it('builds a ws URL on the current host', () => {
    expect(wsUrl('/api/logs/tail', { severity: 'critical', q: undefined })).toBe(
      `ws://${window.location.host}/api/logs/tail?severity=critical`,
    )
  })
})

describe('native app base URL', () => {
  function goNative(server: string | null) {
    vi.stubGlobal('__TAURI_INTERNALS__', {})
    setServerUrl(server)
  }

  it('keeps relative paths on the web even with a stored server', async () => {
    setServerUrl('http://10.0.2.2:8080')
    const fn = mockFetch(jsonResponse({ status: 'ok' }))
    await apiGet('/api/health')
    expect(fn.mock.calls[0][0]).toBe('/api/health')
  })

  it('prefixes requests with the stored server URL', async () => {
    goNative('http://10.0.2.2:8080')
    const fn = mockFetch(jsonResponse({ ok: true }))
    await apiGet('/api/logs', { q: 'ssh' })
    await apiPost('/api/auth/login', { username: 'admin' })
    await apiFetchRaw('/api/ai/chat', { method: 'POST' })
    expect(fn.mock.calls.map((call) => call[0])).toEqual([
      'http://10.0.2.2:8080/api/logs?q=ssh',
      'http://10.0.2.2:8080/api/auth/login',
      'http://10.0.2.2:8080/api/ai/chat',
    ])
  })

  it('builds ws and wss URLs from the stored server', () => {
    goNative('http://10.0.2.2:8080')
    expect(wsUrl('/api/logs/tail', { host: 'web1' })).toBe(
      'ws://10.0.2.2:8080/api/logs/tail?host=web1',
    )
    setServerUrl('https://logs.example.com/loglens')
    expect(wsUrl('/api/logs/tail')).toBe('wss://logs.example.com/loglens/api/logs/tail')
  })

  it('falls back to the page origin without a stored server', () => {
    goNative(null)
    expect(wsUrl('/api/logs/tail')).toBe(`ws://${window.location.host}/api/logs/tail`)
  })
})
