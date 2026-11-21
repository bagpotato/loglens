import { afterEach, describe, expect, it, vi } from 'vitest'
import { apiBase, getServerUrl, isNative, normalizeServerUrl, setServerUrl } from './index'

function goNative() {
  vi.stubGlobal('__TAURI_INTERNALS__', {})
}

afterEach(() => {
  vi.unstubAllGlobals()
})

describe('normalizeServerUrl', () => {
  it('accepts http(s) URLs and strips trailing slashes', () => {
    expect(normalizeServerUrl(' http://10.0.2.2:8080/ ')).toBe('http://10.0.2.2:8080')
    expect(normalizeServerUrl('https://logs.example.com/loglens/')).toBe(
      'https://logs.example.com/loglens',
    )
  })

  it('rejects other schemes and malformed input', () => {
    for (const raw of ['', 'logs.example.com', 'ftp://x', 'javascript:alert(1)', 'http://a/?q=1']) {
      expect(normalizeServerUrl(raw)).toBeNull()
    }
  })
})

describe('server storage', () => {
  it('stores the URL under loglens.server', () => {
    setServerUrl('http://10.0.2.2:8080')
    expect(localStorage.getItem('loglens.server')).toBe('http://10.0.2.2:8080')
    expect(getServerUrl()).toBe('http://10.0.2.2:8080')
    setServerUrl(null)
    expect(getServerUrl()).toBeNull()
  })

  it('returns null when storage throws', () => {
    vi.spyOn(Storage.prototype, 'getItem').mockImplementation(() => {
      throw new Error('denied')
    })
    expect(getServerUrl()).toBeNull()
    vi.restoreAllMocks()
  })
})

describe('apiBase', () => {
  it('is empty on the web even with a stored server', () => {
    setServerUrl('http://10.0.2.2:8080')
    expect(isNative()).toBe(false)
    expect(apiBase()).toBe('')
  })

  it('uses the stored server in the native app', () => {
    goNative()
    expect(isNative()).toBe(true)
    expect(apiBase()).toBe('')
    setServerUrl('http://10.0.2.2:8080')
    expect(apiBase()).toBe('http://10.0.2.2:8080')
  })
})
