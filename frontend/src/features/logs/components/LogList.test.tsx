import { fireEvent, render, screen } from '@testing-library/react'
import { afterAll, beforeAll, describe, expect, it, vi } from 'vitest'
import type { LogEntry } from '@/shared/types/log'
import { SEVERITIES } from '@/shared/types/severity'
import { LogList } from './LogList'

function makeEntries(count: number): LogEntry[] {
  const base = Date.parse('2026-11-21T10:00:00Z')
  return Array.from({ length: count }, (_, i) => ({
    id: `e${i}`,
    timestamp: new Date(base - i * 1000).toISOString(),
    host: `web${i % 3}`,
    source: 'ssh',
    message: `message number ${i}`,
    severity: SEVERITIES[i % 4],
    severity_source: i % 5 === 0 ? ('ai' as const) : ('rule' as const),
    reason: i % 5 === 0 ? 'Repeated auth failures' : undefined,
  }))
}

// jsdom has no layout: give the scroll viewport and every row a size.
const sizes = { clientHeight: 800, clientWidth: 1000, offsetHeight: 64, offsetWidth: 1000 }
const originals = new Map<string, PropertyDescriptor | undefined>()

beforeAll(() => {
  for (const [name, value] of Object.entries(sizes)) {
    originals.set(name, Object.getOwnPropertyDescriptor(HTMLElement.prototype, name))
    Object.defineProperty(HTMLElement.prototype, name, { configurable: true, get: () => value })
  }
})

afterAll(() => {
  for (const [name, descriptor] of originals) {
    if (descriptor) Object.defineProperty(HTMLElement.prototype, name, descriptor)
    else Reflect.deleteProperty(HTMLElement.prototype, name)
  }
})

describe('LogList', () => {
  it('virtualizes 10,000 entries: only a small window is mounted', () => {
    render(<LogList entries={makeEntries(10_000)} onSelect={() => undefined} />)
    const rows = screen.getAllByRole('listitem')
    expect(rows.length).toBeGreaterThan(0)
    expect(rows.length).toBeLessThan(100)
    expect(screen.getByText('message number 0')).toBeInTheDocument()
    expect(screen.queryByText('message number 9999')).not.toBeInTheDocument()
    const list = screen.getByRole('list')
    expect(parseInt(list.style.height, 10)).toBeGreaterThan(10_000 * 10)
  })

  it('marks AI-classified rows and reports the selected entry', () => {
    const onSelect = vi.fn()
    render(<LogList entries={makeEntries(20)} onSelect={onSelect} />)
    const aiMarkers = screen.getAllByLabelText('Severity classified by AI')
    expect(aiMarkers.length).toBeGreaterThan(0)
    const row = screen.getByText('message number 0').closest('button')
    expect(row).toHaveAttribute('title', 'AI: Repeated auth failures')
    fireEvent.click(screen.getByText('message number 3'))
    expect(onSelect).toHaveBeenCalledWith(expect.objectContaining({ id: 'e3' }))
  })

  it('shows an empty state', () => {
    render(<LogList entries={[]} onSelect={() => undefined} />)
    expect(screen.getByText(/no logs match/i)).toBeInTheDocument()
  })
})
