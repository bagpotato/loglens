import { useState, type FormEvent } from 'react'
import { Navigate, useLocation, useNavigate } from 'react-router'
import { ApiError } from '@/shared/api/client'
import { getServerUrl, isNative, normalizeServerUrl, setServerUrl } from '@/shared/platform'
import { Button } from '@/shared/ui/Button'
import { Logo } from '@/shared/ui/Logo'
import { Spinner } from '@/shared/ui/Spinner'
import { useAuth } from './useAuth'

function redirectTarget(state: unknown): string {
  if (state && typeof state === 'object' && 'from' in state && typeof state.from === 'string') {
    return state.from.startsWith('/login') ? '/' : state.from
  }
  return '/'
}

function errorText(error: unknown): string {
  if (error instanceof ApiError && error.status === 401) return 'Invalid username or password.'
  if (error instanceof ApiError) return `Login failed: ${error.message}`
  return 'Cannot reach the server. Check your connection and try again.'
}

export function LoginPage() {
  const { login, isAuthenticated } = useAuth()
  const navigate = useNavigate()
  const location = useLocation()
  const [username, setUsername] = useState('')
  const [password, setPassword] = useState('')
  const [native] = useState(isNative)
  const [server, setServer] = useState(() => getServerUrl() ?? '')
  const [error, setError] = useState<string | null>(null)
  const [submitting, setSubmitting] = useState(false)
  const target = redirectTarget(location.state)

  if (isAuthenticated && !submitting) return <Navigate to={target} replace />

  async function onSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    setError(null)
    if (native) {
      const url = normalizeServerUrl(server)
      if (url === null) {
        setError('Enter a valid server URL starting with http:// or https://.')
        return
      }
      setServerUrl(url)
      setServer(url)
    }
    setSubmitting(true)
    try {
      await login(username.trim(), password)
      navigate(target, { replace: true })
    } catch (err) {
      setError(errorText(err))
      setSubmitting(false)
    }
  }

  return (
    <main className="flex min-h-dvh items-center justify-center bg-bg px-4 py-10">
      <div className="w-full max-w-sm">
        <div className="mb-8 flex flex-col items-start gap-5">
          <Logo className="h-14" />
          <div>
            <h1 className="text-3xl font-bold tracking-tight text-fg">Sign in to LogLens</h1>
            <p className="mt-1 text-base text-muted">Your server logs, explained.</p>
          </div>
          <div aria-hidden="true" className="flex h-1 w-full gap-1">
            <span className="h-full w-1/2 bg-(--sev-critical-solid)" />
            <span className="h-full w-1/4 bg-(--sev-important-solid)" />
            <span className="h-full w-1/8 bg-(--sev-warning-solid)" />
            <span className="h-full flex-1 bg-(--sev-normal-solid)" />
          </div>
        </div>
        <form onSubmit={onSubmit} noValidate className="ui-panel space-y-4 p-5">
          {native && (
            <div>
              <label htmlFor="server" className="ui-label">
                Server URL
              </label>
              <input
                id="server"
                name="server"
                type="url"
                inputMode="url"
                autoComplete="url"
                autoCapitalize="none"
                spellCheck={false}
                placeholder="http://192.168.1.10:8080"
                required
                value={server}
                onChange={(e) => setServer(e.target.value)}
                className="ui-input ui-input--touch font-mono"
              />
            </div>
          )}
          <div>
            <label htmlFor="username" className="ui-label">
              Username
            </label>
            <input
              id="username"
              name="username"
              autoComplete="username"
              autoCapitalize="none"
              spellCheck={false}
              required
              value={username}
              onChange={(e) => setUsername(e.target.value)}
              className="ui-input ui-input--touch"
            />
          </div>
          <div>
            <label htmlFor="password" className="ui-label">
              Password
            </label>
            <input
              id="password"
              name="password"
              type="password"
              autoComplete="current-password"
              required
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              className="ui-input ui-input--touch"
            />
          </div>
          {error && (
            <p role="alert" className="ui-alert">
              {error}
            </p>
          )}
          <Button
            type="submit"
            className="h-11 w-full"
            disabled={
              submitting ||
              username.trim() === '' ||
              password === '' ||
              (native && server.trim() === '')
            }
          >
            {submitting ? <Spinner label="Signing in" /> : 'Sign in'}
          </Button>
        </form>
      </div>
    </main>
  )
}
