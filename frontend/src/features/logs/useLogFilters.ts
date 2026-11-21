import { useCallback, useMemo } from 'react'
import { useSearchParams } from 'react-router'
import type { LogFilters } from '@/shared/types/filters'
import { isSeverity, type Severity } from '@/shared/types/severity'

export const DEFAULT_SEVERITIES: readonly Severity[] = ['critical', 'important']

export interface LogsViewState {
  filters: LogFilters
  live: boolean
  focus?: string
}

const TEXT_KEYS = ['host', 'source', 'q', 'start', 'end'] as const

function validTime(value: string | null): string | undefined {
  return value && !Number.isNaN(Date.parse(value)) ? value : undefined
}

export function parseLogsParams(params: URLSearchParams): LogsViewState {
  const raw = params.get('severity')
  const severity =
    raw === null
      ? [...DEFAULT_SEVERITIES]
      : raw
          .split(',')
          .map((s) => s.trim())
          .filter(isSeverity)
  const filters: LogFilters = { severity: [...new Set(severity)] }
  const host = params.get('host')
  const source = params.get('source')
  const q = params.get('q')
  if (host) filters.host = host
  if (source) filters.source = source
  if (q) filters.q = q
  const start = validTime(params.get('start'))
  const end = validTime(params.get('end'))
  if (start) filters.start = start
  if (end) filters.end = end
  const focus = validTime(params.get('focus'))
  return { filters, live: params.get('live') === '1', ...(focus ? { focus } : {}) }
}

export function buildLogsParams(state: LogsViewState): URLSearchParams {
  const params = new URLSearchParams()
  for (const key of TEXT_KEYS) {
    const value = state.filters[key]
    if (value) params.set(key, value)
  }
  params.set('severity', state.filters.severity.join(','))
  if (state.live) params.set('live', '1')
  if (state.focus) params.set('focus', state.focus)
  return params
}

export interface UseLogFilters extends LogsViewState {
  setFilters: (patch: Partial<LogFilters>, options?: { replace?: boolean }) => void
  setLive: (live: boolean) => void
}

/** Filters, live mode and focused timestamp, kept in the URL so a copied link reopens the view. */
export function useLogFilters(): UseLogFilters {
  const [params, setParams] = useSearchParams()
  const state = useMemo(() => parseLogsParams(params), [params])

  const setFilters = useCallback(
    (patch: Partial<LogFilters>, options?: { replace?: boolean }) => {
      setParams((prev) => {
        const current = parseLogsParams(prev)
        return buildLogsParams({ filters: { ...current.filters, ...patch }, live: current.live })
      }, options)
    },
    [setParams],
  )

  const setLive = useCallback(
    (live: boolean) => {
      setParams((prev) => {
        const current = parseLogsParams(prev)
        const filters = live ? { ...current.filters, end: undefined } : current.filters
        return buildLogsParams({ filters, live })
      })
    },
    [setParams],
  )

  return { ...state, setFilters, setLive }
}
