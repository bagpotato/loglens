export interface AuthUser {
  username: string
  role: string
}

interface TokenClaims {
  sub?: unknown
  role?: unknown
  exp?: unknown
}

function decodePayload(token: string): TokenClaims | null {
  const part = token.split('.')[1]
  if (!part) return null
  try {
    const base64 = part.replace(/-/g, '+').replace(/_/g, '/')
    const padded = base64.padEnd(base64.length + ((4 - (base64.length % 4)) % 4), '=')
    const json = new TextDecoder().decode(Uint8Array.from(atob(padded), (c) => c.charCodeAt(0)))
    const value: unknown = JSON.parse(json)
    return value && typeof value === 'object' ? (value as TokenClaims) : null
  } catch {
    return null
  }
}

export function userFromToken(token: string | null, nowMs: number = Date.now()): AuthUser | null {
  if (!token) return null
  const claims = decodePayload(token)
  if (!claims || typeof claims.sub !== 'string') return null
  if (typeof claims.exp === 'number' && claims.exp * 1000 <= nowMs) return null
  return { username: claims.sub, role: typeof claims.role === 'string' ? claims.role : 'admin' }
}
