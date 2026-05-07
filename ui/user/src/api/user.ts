import request, { type ApiResponse } from './request'

export interface User {
  id: number
  email: string
  username: string
  balance: string
}

export async function login(email: string, password: string): Promise<ApiResponse<{ user: User; api_key: string }>> {
  return request.post('/user/login', { email, password })
}

export async function register(email: string, password: string, username?: string): Promise<ApiResponse<{ user: User; api_key: string }>> {
  return request.post('/user/register', { email, password, username })
}

export async function getUserInfo(): Promise<ApiResponse<User>> {
  return request.get('/user')
}