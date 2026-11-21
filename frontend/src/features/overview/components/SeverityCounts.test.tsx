import { render, screen } from '@testing-library/react'
import { MemoryRouter } from 'react-router'
import { describe, expect, it } from 'vitest'
import { SeverityCounts } from './SeverityCounts'

function renderCounts(props: Parameters<typeof SeverityCounts>[0]) {
  return render(
    <MemoryRouter>
      <SeverityCounts {...props} />
    </MemoryRouter>,
  )
}

describe('SeverityCounts', () => {
  it('shows one card per severity, most urgent first, with counts', () => {
    renderCounts({ bySeverity: { critical: 3, important: 12, normal: 1500 } })
    const links = screen.getAllByRole('link')
    expect(links.map((l) => l.getAttribute('href'))).toEqual([
      '/logs?severity=critical',
      '/logs?severity=important',
      '/logs?severity=warning',
      '/logs?severity=normal',
    ])
    expect(screen.getByTestId('count-critical')).toHaveTextContent('3')
    expect(screen.getByTestId('count-important')).toHaveTextContent('12')
    expect(screen.getByTestId('count-warning')).toHaveTextContent('0')
    expect(screen.getByTestId('count-normal')).toHaveTextContent(/1.?500/)
  })

  it('shows placeholders while loading', () => {
    renderCounts({ loading: true })
    expect(screen.getByTestId('count-critical')).toHaveTextContent('—')
    expect(screen.getByRole('link', { name: 'Critical: loading events' })).toBeInTheDocument()
  })
})
