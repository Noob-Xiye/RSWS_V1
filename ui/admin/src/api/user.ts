import request, { type ApiResponse, type PaginatedResponse, type PaginationParams } from './request'

/** 管理员视图用户信息（对齐后端 AdminUserView） */
export interface AdminUser {
  id: number
  email: string
  username: string
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
