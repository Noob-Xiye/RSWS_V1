import request, { type ApiResponse, type PaginatedResponse, type PaginationParams } from './request'

/** 管理员视图用户信息（对齐后端 AdminUserView） */
export interface AdminUser {
  id: number
  email: string
  username: string
  nickname: string
  avatar_url: string | null
  is_active: boolean
  created_at: string
}

export interface UserListParams extends PaginationParams {
  email?: string
  username?: string
  is_active?: boolean
}

/** 获取用户列表 */
export async function listUsers(params?: UserListParams): Promise<ApiResponse<PaginatedResponse<AdminUser>>> {
  return request.get('/admin/user', { params })
}

/** 禁用用户 */
export async function deactivateUser(id: number): Promise<ApiResponse<void>> {
  return request.post(`/admin/user/${id}/deactivate`)
}

/** 启用用户 */
export async function activateUser(id: number): Promise<ApiResponse<void>> {
  return request.post(`/admin/user/${id}/activate`)
}

// ==================== 用户 API Key 管理 ====================

export interface UserApiKey {
  id: number
  user_id: number
  api_key: string
  name: string
  permissions: string[]
  rate_limit: number
  last_used_at: string | null
  expires_at: string | null
  is_active: boolean
  created_at: string
  updated_at: string
}

export interface CreateApiKeyParams {
  name: string
  permissions: string[]
  rate_limit?: number
  expires_in_days?: number
}

/** 获取用户的 API Key 列表 */
export async function listUserApiKeys(userId: number): Promise<ApiResponse<{ items: UserApiKey[] }>> {
  return request.get(`/admin/users/${userId}/api-keys`)
}

/** 为用户创建 API Key */
export async function createUserApiKey(userId: number, data: CreateApiKeyParams): Promise<ApiResponse<UserApiKey>> {
  return request.post(`/admin/users/${userId}/api-keys`, data)
}

/** 删除用户的 API Key */
export async function deleteUserApiKey(userId: number, keyId: number): Promise<ApiResponse<{ deleted: boolean }>> {
  return request.delete(`/admin/users/${userId}/api-keys/${keyId}`)
}

/** 切换用户 API Key 状态 */
export async function toggleUserApiKey(userId: number, keyId: number): Promise<ApiResponse<{ is_active: boolean }>> {
  return request.put(`/admin/users/${userId}/api-keys/${keyId}`)
}

