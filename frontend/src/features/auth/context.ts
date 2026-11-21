import { createContext } from 'react'
import type { AuthUser } from './session'

export interface AuthContextValue {
  user: AuthUser | null
  isAuthenticated: boolean
  login: (username: string, password: string) => Promise<AuthUser>
  logout: () => void
}

export const AuthContext = createContext<AuthContextValue | null>(null)
