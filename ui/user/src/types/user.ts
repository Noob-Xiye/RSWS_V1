// ========== 用户类型 (User 端) ==========

import type { ApiResponse } from './api'

/** 用户详细信息 */
export interface UserProfile {
  id: number
  email: string
  username: string
  nickname?: string
  avatar_url?: string | null
  bio?: string
  balance: string  // 单位: 分
  is_active: boolean
  is_email_verified: boolean
  created_at: string
  updated_at: string
  last_login?: string
}

/** 更新用户资料请求 */
export interface UpdateProfileRequest {
  username?: string
  nickname?: string
  bio?: string
}

/** 修改密码请求 */
export interface ChangePasswordRequest {
  old_password: string
  new_password: string
}

/** 充值请求 */
export interface RechargeRequest {
  amount: string  // 单位: 分
  payment_method: 'paypal' | 'usdt_trc20' | 'usdt_erc20'
}

/** 充值响应 */
export interface RechargeData {
  order_id: number
  order_no: string
  amount: string
  payment_url?: string  // PayPal 支付链接 或 USDT 地址
  qr_code?: string  // USDT 支付二维码 Base64
  wallet_address?: string  // USDT 钱包地址
  expires_at: string
}

export type UserProfileResponse = ApiResponse<UserProfile>
export type UpdateProfileResponse = ApiResponse<UserProfile>
export type ChangePasswordResponse = ApiResponse<{ changed: boolean }>
export type RechargeResponse = ApiResponse<RechargeData>