import { act, render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { MemoryRouter, Route, Routes } from 'react-router'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { UNAUTHORIZED_EVENT, getToken } from '@/shared/api/client'
import { getServerUrl, setServerUrl } from '@/shared/platform'
import { AuthProvider } from './AuthProvider'
import { LoginPage } from './LoginPage'
import { useAuth } from './useAuth'

function Home() {
  const { user } = useAuth()
  return <p>Welcome {user?.username}</p>
}

function renderApp() {
  return render(
    <AuthProvider>
      <MemoryRouter initialEntries={['/login']}>
        <Routes>
          <Route path="/login" element={<LoginPage />} />
          <Route path="/" element={<Home />} />
        </Routes>
      </MemoryRouter>
    </AuthProvider>,
  )
}

function respond(body: unknown, status: number) {
  const fn = vi.fn<typeof fetch>().mockResolvedValue(
    new Response(JSON.stringify(body), {
      status,
      headers: { 'Content-Type': 'application/json' },
    }),
  )
  vi.stubGlobal('fetch', fn)
  return fn
}

async function submit(username: string, password: string) {
  const user = userEvent.setup()
  await user.type(screen.getByLabelText('Username'), username)
  await user.type(screen.getByLabelText('Password'), password)
  await user.click(screen.getByRole('button', { name: 'Sign in' }))
}

afterEach(() => {
  vi.unstubAllGlobals()
})

describe('LoginPage', () => {
  it('stores the token and navigates home on success', async () => {
    const fn = respond({ token: 'jwt-123', username: 'admin', role: 'admin' }, 200)
    renderApp()
    await submit('admin', 'secret')
    expect(await screen.findByText('Welcome admin')).toBeInTheDocument()
    expect(getToken()).toBe('jwt-123')
    expect(fn).toHaveBeenCalledWith('/api/auth/login', expect.objectContaining({ method: 'POST' }))
    expect(fn.mock.calls[0][1]?.body).toBe('{"username":"admin","password":"secret"}')
  })

  it('shows an error and stays on the form on bad credentials', async () => {
    respond({ error: 'invalid credentials' }, 401)
    renderApp()
    await submit('admin', 'wrong')
    expect(await screen.findByRole('alert')).toHaveTextContent('Invalid username or password.')
    expect(getToken()).toBeNull()
    expect(screen.getByLabelText('Password')).toBeInTheDocument()
  })

  it('shows a connection error when the server is unreachable', async () => {
    vi.stubGlobal('fetch', vi.fn<typeof fetch>().mockRejectedValue(new TypeError('offline')))
    renderApp()
    await submit('admin', 'secret')
    expect(await screen.findByRole('alert')).toHaveTextContent('Cannot reach the server')
  })
})

describe('useAuth', () => {
  it('logs out when the API reports unauthorized', async () => {
    respond({ token: 'jwt-123', username: 'admin', role: 'admin' }, 200)
    renderApp()
    await submit('admin', 'secret')
    expect(await screen.findByText('Welcome admin')).toBeInTheDocument()
    act(() => {
      window.dispatchEvent(new Event(UNAUTHORIZED_EVENT))
    })
    expect(screen.getByText(/^Welcome\s*$/)).toBeInTheDocument()
    expect(getToken()).toBeNull()
  })
})

describe('LoginPage in the native app', () => {
  it('hides the server field on the web', () => {
    renderApp()
    expect(screen.queryByLabelText('Server URL')).not.toBeInTheDocument()
  })

  it('rejects an invalid server URL without calling the API', async () => {
    vi.stubGlobal('__TAURI_INTERNALS__', {})
    const fn = respond({ token: 'jwt-123', username: 'admin', role: 'admin' }, 200)
    renderApp()
    await userEvent.setup().type(screen.getByLabelText('Server URL'), 'ftp://logs')
    await submit('admin', 'secret')
    expect(await screen.findByRole('alert')).toHaveTextContent('valid server URL')
    expect(fn).not.toHaveBeenCalled()
  })

  it('logs in against the entered server and remembers it', async () => {
    vi.stubGlobal('__TAURI_INTERNALS__', {})
    const fn = respond({ token: 'jwt-123', username: 'admin', role: 'admin' }, 200)
    renderApp()
    await userEvent.setup().type(screen.getByLabelText('Server URL'), 'http://10.0.2.2:8080/')
    await submit('admin', 'secret')
    expect(await screen.findByText('Welcome admin')).toBeInTheDocument()
    expect(fn.mock.calls[0][0]).toBe('http://10.0.2.2:8080/api/auth/login')
    expect(getServerUrl()).toBe('http://10.0.2.2:8080')
  })

  it('prefills the remembered server', () => {
    vi.stubGlobal('__TAURI_INTERNALS__', {})
    setServerUrl('http://192.168.1.10:8080')
    renderApp()
    expect(screen.getByLabelText('Server URL')).toHaveValue('http://192.168.1.10:8080')
  })
})
