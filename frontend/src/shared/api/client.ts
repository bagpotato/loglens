import { apiBase } from '@/shared/platform'

const TOKEN_KEY = 'loglens.token'

export const UNAUTHORIZED_EVENT = 'loglens:unauthorized'

export type QueryParams = Record<string, string | undefined>

export class ApiError extends Error {
  status: number

  constructor(status: number, message: string) {
    super(message)
    this.name = 'ApiError'
    this.status = status
  }
}

export function getToken(): string | null {
  try {
    return localStorage.getItem(TOKEN_KEY)
  } catch {
    return null
  }
}

export function setToken(token: string | null): void {
  try {
    if (token === null) localStorage.removeItem(TOKEN_KEY)
    else localStorage.setItem(TOKEN_KEY, token)
  } catch {
    // storage unavailable (private mode); keep running without persistence
  }
}

function withQuery(path: string, params?: QueryParams): string {
  if (!params) return path
  const search = new URLSearchParams()
  for (const [key, value] of Object.entries(params)) {
    if (value !== undefined && value !== '') search.set(key, value)
  }
  const query = search.toString()
  return query ? `${path}?${query}` : path
}

async function errorMessage(response: Response): Promise<string> {
  try {
    const body: unknown = await response.json()
    if (body && typeof body === 'object' && 'error' in body && typeof body.error === 'string') {
      return body.error
    }
  } catch {
    // body is not JSON
  }
  return response.statusText || `HTTP ${response.status}`
}

export async function apiFetchRaw(path: string, init: RequestInit = {}): Promise<Response> {
  const headers = new Headers(init.headers)
  const token = getToken()
  if (token) headers.set('Authorization', `Bearer ${token}`)
  const response = await fetch(`${apiBase()}${path}`, { ...init, headers })
  if (!response.ok) {
    if (response.status === 401) {
      setToken(null)
      window.dispatchEvent(new Event(UNAUTHORIZED_EVENT))
    }
    throw new ApiError(response.status, await errorMessage(response))
  }
  return response
}

export async function apiGet<T>(path: string, params?: QueryParams): Promise<T> {
  const response = await apiFetchRaw(withQuery(path, params), {
    headers: { Accept: 'application/json' },
  })
  return (await response.json()) as T
}

export async function apiPost<T>(path: string, body: unknown): Promise<T> {
  const response = await apiFetchRaw(path, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json', Accept: 'application/json' },
    body: JSON.stringify(body),
  })
  return (await response.json()) as T
}

export function wsUrl(path: string, params?: QueryParams): string {
  const base = apiBase()
  const origin = base === '' ? window.location.origin : base
  return `${origin.replace(/^http/, 'ws')}${withQuery(path, params)}`
}
