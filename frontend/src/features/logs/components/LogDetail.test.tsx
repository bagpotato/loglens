import { fireEvent, render, screen } from '@testing-library/react'
import { MemoryRouter } from 'react-router'
import { afterEach, describe, expect, it, vi } from 'vitest'
import type { LogEntry } from '@/shared/types/log'
import { LogDetail } from './LogDetail'

const ENTRY: LogEntry = {
  id: '1732183200000000000-abcd1234',
  timestamp: '2026-11-21T10:00:00Z',
  host: 'web1',
  source: 'kernel',
  unit: 'kernel',
  message: 'Out of memory: Killed process 1234 (java)',
  severity: 'critical',
  severity_source: 'ai',
  reason: 'The kernel killed a process to free memory',
}

afterEach(() => {
  vi.unstubAllGlobals()
})

function renderDetail(entry: LogEntry | null = ENTRY) {
  return render(
    <MemoryRouter>
      <LogDetail entry={entry} onClose={() => undefined} />
    </MemoryRouter>,
  )
}

describe('LogDetail', () => {
  it('shows all fields, the AI reason and an Explain button', () => {
    renderDetail()
    expect(screen.getByRole('dialog')).toBeInTheDocument()
    expect(screen.getByText(ENTRY.message)).toBeInTheDocument()
    expect(screen.getByText('web1')).toBeInTheDocument()
    expect(screen.getByText(ENTRY.id)).toBeInTheDocument()
    expect(screen.getByText(/kernel killed a process/)).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /explain with ai/i })).toBeInTheDocument()
  })

  it('renders nothing when no entry is selected', () => {
    renderDetail(null)
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument()
  })

  it('shows the explanation after one tap on Explain', async () => {
    const fetchMock = vi.fn(() =>
      Promise.resolve(
        new Response(JSON.stringify({ explanation: '## What happened\n\n```\nfree -h\n```' }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        }),
      ),
    )
    vi.stubGlobal('fetch', fetchMock)
    renderDetail()
    fireEvent.click(screen.getByRole('button', { name: /explain with ai/i }))
    expect(await screen.findByText('What happened')).toBeInTheDocument()
    expect(screen.getByText('free -h')).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Copy' })).toBeInTheDocument()
    const [url, init] = fetchMock.mock.calls[0] as unknown as [string, RequestInit]
    expect(url).toBe('/api/ai/explain')
    expect(JSON.parse(String(init.body))).toEqual({ entry: ENTRY })
  })
})
