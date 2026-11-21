import { describe, expect, it } from 'vitest'
import { userFromToken } from './session'

function token(payload: object): string {
  const encode = (value: object) =>
    btoa(JSON.stringify(value)).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '')
  return `${encode({ alg: 'HS256', typ: 'JWT' })}.${encode(payload)}.signature`
}

describe('userFromToken', () => {
  it('reads username and role from a valid token', () => {
    const t = token({ sub: 'admin', role: 'admin', exp: 2_000_000_000 })
    expect(userFromToken(t, 1_000_000_000_000)).toEqual({ username: 'admin', role: 'admin' })
  })

  it('rejects expired, malformed and missing tokens', () => {
    expect(userFromToken(token({ sub: 'admin', exp: 10 }), 20_000)).toBeNull()
    expect(userFromToken('not-a-jwt')).toBeNull()
    expect(userFromToken(null)).toBeNull()
  })
})
