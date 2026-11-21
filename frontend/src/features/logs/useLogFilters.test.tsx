import type { ReactNode } from 'react'
import { act, renderHook } from '@testing-library/react'
import { MemoryRouter, useLocation } from 'react-router'
import { describe, expect, it } from 'vitest'
import { buildLogsParams, parseLogsParams, useLogFilters } from './useLogFilters'

function wrapperFor(url: string) {
  return function Wrapper({ children }: { children: ReactNode }) {
    return <MemoryRouter initialEntries={[url]}>{children}</MemoryRouter>
  }
}

function useFiltersAndLocation() {
  return { ...useLogFilters(), location: useLocation() }
}

describe('parseLogsParams / buildLogsParams', () => {
  it('defaults to critical + important when severity is absent', () => {
    const state = parseLogsParams(new URLSearchParams(''))
    expect(state.filters).toEqual({ severity: ['critical', 'important'] })
    expect(state.live).toBe(false)
    expect(state.focus).toBeUndefined()
  })

  it('keeps an explicit empty severity list (all severities)', () => {
    expect(parseLogsParams(new URLSearchParams('severity=')).filters.severity).toEqual([])
  })

  it('round-trips every field through the URL', () => {
    const state = {
      filters: {
        host: 'web1',
        source: 'ssh',
        severity: ['warning' as const, 'normal' as const],
        q: 'failed password',
        start: '2026-11-21T09:00:00.000Z',
        end: '2026-11-21T10:00:00.000Z',
      },
      live: true,
      focus: '2026-11-21T09:30:00Z',
    }
    const params = buildLogsParams(state)
    expect(params.get('severity')).toBe('warning,normal')
    expect(params.get('live')).toBe('1')
    expect(parseLogsParams(new URLSearchParams(params.toString()))).toEqual(state)
  })

  it('drops unknown severities and invalid times', () => {
    const state = parseLogsParams(new URLSearchParams('severity=critical,bogus&start=yesterday'))
    expect(state.filters).toEqual({ severity: ['critical'] })
  })
})

describe('useLogFilters', () => {
  it('reads the URL and writes patches back to it', () => {
    const { result } = renderHook(useFiltersAndLocation, {
      wrapper: wrapperFor('/logs?host=web1&focus=2026-11-21T10:00:00Z'),
    })
    expect(result.current.filters).toEqual({ host: 'web1', severity: ['critical', 'important'] })
    expect(result.current.focus).toBe('2026-11-21T10:00:00Z')

    act(() => result.current.setFilters({ source: 'nginx', severity: ['critical'] }))
    const search = new URLSearchParams(result.current.location.search)
    expect(search.get('host')).toBe('web1')
    expect(search.get('source')).toBe('nginx')
    expect(search.get('severity')).toBe('critical')
    expect(search.get('focus')).toBeNull()
    expect(result.current.filters.source).toBe('nginx')
  })

  it('toggles live mode and clears the end of the range', () => {
    const { result } = renderHook(useFiltersAndLocation, {
      wrapper: wrapperFor('/logs?end=2026-11-21T10:00:00Z'),
    })
    act(() => result.current.setLive(true))
    expect(result.current.live).toBe(true)
    expect(result.current.filters.end).toBeUndefined()
    expect(result.current.location.search).toContain('live=1')
  })
})
