import { render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { Badge } from './Badge'

describe('Badge', () => {
  it('renders the severity label with severity colors', () => {
    render(<Badge severity="critical" />)
    const badge = screen.getByText('Critical')
    expect(badge).toHaveAttribute('data-severity', 'critical')
    expect(badge).toHaveClass('ui-badge')
  })

  it.each(['important', 'warning', 'normal'] as const)('uses the %s palette', (severity) => {
    render(<Badge severity={severity} />)
    expect(screen.getByText(new RegExp(severity, 'i'))).toHaveAttribute('data-severity', severity)
  })

  it('prefers children over the default label', () => {
    render(<Badge severity="warning">AI: warning</Badge>)
    expect(screen.getByText('AI: warning')).toBeInTheDocument()
    expect(screen.queryByText('Warning')).not.toBeInTheDocument()
  })

  it('falls back to neutral colors without a severity', () => {
    render(<Badge>ssh</Badge>)
    expect(screen.getByText('ssh')).not.toHaveAttribute('data-severity')
  })
})
