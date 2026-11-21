import { isNative } from './native'

const SERVER_KEY = 'loglens.server'

export function normalizeServerUrl(raw: string): string | null {
  const value = raw.trim()
  if (value === '') return null
  let url: URL
  try {
    url = new URL(value)
  } catch {
    return null
  }
  if (url.protocol !== 'http:' && url.protocol !== 'https:') return null
  if (url.search !== '' || url.hash !== '' || url.username !== '' || url.password !== '') {
    return null
  }
  return `${url.origin}${url.pathname}`.replace(/\/+$/, '')
}

export function getServerUrl(): string | null {
  try {
    const stored = localStorage.getItem(SERVER_KEY)
    return stored === null ? null : normalizeServerUrl(stored)
  } catch {
    return null
  }
}

export function setServerUrl(url: string | null): void {
  try {
    if (url === null) localStorage.removeItem(SERVER_KEY)
    else localStorage.setItem(SERVER_KEY, url)
  } catch {
    // storage unavailable; the URL lasts only for this session
  }
}

export function apiBase(): string {
  if (!isNative()) return ''
  return getServerUrl() ?? ''
}
