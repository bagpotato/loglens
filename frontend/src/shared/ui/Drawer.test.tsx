import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import { Drawer } from './Drawer'

describe('Drawer', () => {
  it('renders nothing when closed', () => {
    render(
      <Drawer open={false} onClose={() => undefined} title="Details">
        body
      </Drawer>,
    )
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument()
  })

  it('is a labelled dialog that focuses its first control', () => {
    render(
      <Drawer open onClose={() => undefined} title="Details">
        <button type="button">Inner</button>
      </Drawer>,
    )
    expect(screen.getByRole('dialog', { name: 'Details' })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Close' })).toHaveFocus()
  })

  it('closes on Escape, backdrop click and close button', async () => {
    const user = userEvent.setup()
    const onClose = vi.fn()
    render(
      <Drawer open onClose={onClose} title="Details">
        body
      </Drawer>,
    )
    await user.keyboard('{Escape}')
    await user.click(screen.getByTestId('drawer-backdrop'))
    await user.click(screen.getByRole('button', { name: 'Close' }))
    expect(onClose).toHaveBeenCalledTimes(3)
  })
})
