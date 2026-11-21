import { useEffect, useMemo, useState } from 'react'
import { ApiError, apiGet, getToken, wsUrl } from '@/shared/api/client'
import type { LogFilters } from '@/shared/types/filters'
import type { LogEntry } from '@/shared/types/log'
import { isSeverity } from '@/shared/types/severity'
import { tailParams } from './api'
import { isLogEntry, prependEntries } from './entries'

export type TailStatus = 'idle' | 'connecting' | 'open' | 'reconnecting'

const FLUSH_MS = 200
const MAX_BACKOFF_MS = 30_000

/** False when the session is gone (401); the client then logs the user out. */
async function sessionStillValid(): Promise<boolean> {
  try {
    await apiGet('/api/meta')
    return true
  } catch (err) {
    return !(err instanceof ApiError && err.status === 401)
  }
}

function parseMessage(data: unknown): LogEntry | null {
  if (typeof data !== 'string') return null
  try {
    const value: unknown = JSON.parse(data)
    return isLogEntry(value) ? value : null
  } catch {
    return null
  }
}

/** Live entries from `WS /api/logs/tail`, newest first, reset whenever the filters change. */
export function useTail(
  filters: LogFilters,
  enabled: boolean,
): { entries: LogEntry[]; status: TailStatus } {
  const { host, source, q } = filters
  const severityKey = filters.severity.join(',')
  const params = useMemo(
    () => tailParams({ host, source, q, severity: severityKey.split(',').filter(isSeverity) }),
    [host, source, q, severityKey],
  )
  const key = JSON.stringify(params)
  const [buffer, setBuffer] = useState<{ key: string; entries: LogEntry[] }>({
    key: '',
    entries: [],
  })
  const [conn, setConn] = useState<{ key: string; status: TailStatus }>({
    key: '',
    status: 'idle',
  })

  useEffect(() => {
    if (!enabled) return
    let socket: WebSocket | null = null
    let attempt = 0
    let closed = false
    let retryTimer: ReturnType<typeof setTimeout> | undefined
    let flushTimer: ReturnType<typeof setTimeout> | undefined
    let pending: LogEntry[] = []

    const flush = () => {
      flushTimer = undefined
      const batch = pending
      pending = []
      if (batch.length === 0) return
      setBuffer((prev) => ({
        key,
        entries: prependEntries(prev.key === key ? prev.entries : [], batch),
      }))
    }

    const scheduleReconnect = () => {
      if (closed) return
      const delay = Math.min(MAX_BACKOFF_MS, 1000 * 2 ** attempt)
      attempt += 1
      retryTimer = setTimeout(connect, delay)
    }

    const stop = () => {
      closed = true
      setConn({ key, status: 'idle' })
    }

    function connect() {
      const token = getToken()
      if (!token) {
        stop()
        return
      }
      let opened = false
      socket = new WebSocket(wsUrl('/api/logs/tail', params), ['bearer', token])
      socket.onopen = () => {
        opened = true
        attempt = 0
        setConn({ key, status: 'open' })
      }
      socket.onmessage = (event: MessageEvent<unknown>) => {
        const entry = parseMessage(event.data)
        if (!entry) return
        pending.push(entry)
        flushTimer ??= setTimeout(flush, FLUSH_MS)
      }
      socket.onclose = () => {
        if (closed) return
        setConn({ key, status: 'reconnecting' })
        if (opened) {
          scheduleReconnect()
          return
        }
        // Rejected handshake: an expired token would otherwise retry forever.
        void sessionStillValid().then((valid) => (valid ? scheduleReconnect() : stop()))
      }
    }

    connect()
    return () => {
      closed = true
      clearTimeout(retryTimer)
      clearTimeout(flushTimer)
      socket?.close()
    }
  }, [enabled, key, params])

  const entries = enabled && buffer.key === key ? buffer.entries : []
  const status: TailStatus = !enabled ? 'idle' : conn.key === key ? conn.status : 'connecting'
  return { entries, status }
}
