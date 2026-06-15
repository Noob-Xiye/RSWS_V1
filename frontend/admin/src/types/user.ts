// ========== 用户类型 ==========

import type { ApiResponse, PaginatedData } from './api'

/** 用户状态 */
export type UserStatus = 'active' | 'inactive' | 'banned'

/** 用户信息 */
export interface User {
  id: number
  email: string
  username: string
  nickname?: string
  avatar_url?: string | null
  balance: string  // 单位: 分
  status: UserStatus
  is_email_verified: boolean
  created_at: string
  updated_at: string
  last_login?: string
}

/** 用户列表数据 */
export interface UserListData extends PaginatedData<User> {}
export type UserListResponse = ApiResponse<UserListData>

/** 用户查询参数 */
export interface UserListParams {
  page?: number
  page_size?: number
  email?: string
  username?: string
  status?: UserStatus
  is_email_verified?: boolean
  start_date?: string
  end_date?: string
}

/** 用户充值记录 */
export interface UserBalanceLog {
  id: number
  user_id: number
  amount: string  // 单位: 分 (正值充值, 负值扣减)
  balance_before: string
  balance_after: string
  reason: string
  created_at: string
}

export interface UserBalanceLogListData extends PaginatedData<UserBalanceLog> {}
export type UserBalanceLogListResponse = ApiResponse<UserBalanceLogListData>