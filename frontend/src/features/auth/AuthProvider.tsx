import { useCallback, useEffect, useMemo, useRef, useState, type ReactNode } from 'react'
import { UNAUTHORIZED_EVENT, getToken, setToken } from '@/shared/api/client'
import { login as loginRequest } from './api'
import { AuthContext, type AuthContextValue } from './context'
import { userFromToken, type AuthUser } from './session'

export interface AuthProviderProps {
  children?: ReactNode
  onLogout?: () => void
}

function initialUser(): AuthUser | null {
  const token = getToken()
  const user = userFromToken(token)
  if (token && !user) setToken(null)
  return user
}

export function AuthProvider({ children, onLogout }: AuthProviderProps) {
  const [user, setUser] = useState<AuthUser | null>(initialUser)
  const onLogoutRef = useRef(onLogout)

  useEffect(() => {
    onLogoutRef.current = onLogout
  }, [onLogout])

  const logout = useCallback(() => {
    setToken(null)
    setUser(null)
    onLogoutRef.current?.()
  }, [])

  useEffect(() => {
    window.addEventListener(UNAUTHORIZED_EVENT, logout)
    return () => window.removeEventListener(UNAUTHORIZED_EVENT, logout)
  }, [logout])

  const login = useCallback(async (username: string, password: string) => {
    const response = await loginRequest(username, password)
    setToken(response.token)
    const next: AuthUser = { username: response.username, role: response.role }
    setUser(next)
    return next
  }, [])

  const value = useMemo<AuthContextValue>(
    () => ({ user, isAuthenticated: user !== null, login, logout }),
    [user, login, logout],
  )

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>
}
