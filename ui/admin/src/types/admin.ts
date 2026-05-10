// ========== 管理员类型 ==========

import type { ApiResponse, PaginatedData } from './api'

/** 管理员信息 */
export interface Admin {
  id: number
  email: string
  username: string
  avatar_url?: string
  role: AdminRole
  permissions: string[]
  is_active: boolean
  created_at: string
  last_login?: string
}

/** 管理员角色 */
export type AdminRole = 'super_admin' | 'admin' | 'moderator' | 'viewer'

/** 管理员列表响应 */
export interface AdminListData extends PaginatedData<Admin> {}
export type AdminListResponse = ApiResponse<AdminListData>

/** 管理员登录请求 */
export interface AdminLoginRequest {
  email: string
  password: string
}

/** 管理员登录响应 */
export interface AdminLoginResponse {
  admin: Admin
  token: string
  expires_at: string
}

/** 创建管理员参数 */
export interface CreateAdminRequest {
  email: string
  username: string
  password: string
  role: AdminRole
}

/** 更新管理员参数 */
export interface UpdateAdminRequest {
  username?: string
  role?: AdminRole
  is_active?: boolean
}