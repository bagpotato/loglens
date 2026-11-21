import { apiPost } from '@/shared/api/client'

export interface LoginResponse {
  token: string
  username: string
  role: string
}

export function login(username: string, password: string): Promise<LoginResponse> {
  return apiPost<LoginResponse>('/api/auth/login', { username, password })
}
