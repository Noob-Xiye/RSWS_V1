import request, { type ApiResponse, type PaginatedResponse, type PaginationParams } from './request'

export interface User {
  id: number
  email: string
  username: string
  balance: string
  is_active: boolean
  created_at: string
  last_login?: string | null
}

export interface UserListParams extends PaginationParams {
  email?: string
  username?: string
  is_active?: boolean
}

// 获取用户列表
export async function listUsers(params?: UserListParams): Promise<ApiResponse<PaginatedResponse<User>>> {
  return request.get('/user', { params })
}

// 获取用户详情
export async function getUser(id: number): Promise<ApiResponse<User>> {
  return request.get(`/user/${id}`)
}

// 禁用用户
export async function deactivateUser(id: number): Promise<ApiResponse<void>> {
  return request.delete(`/user/${id}`)
}

// 启用用户
export async function activateUser(id: number): Promise<ApiResponse<void>> {
  return request.post(`/user/${id}/activate`)
}