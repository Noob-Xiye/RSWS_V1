// ========== 认证类型 (User 端) ==========

import type { ApiResponse } from './api'

/** 登录方式 */
export type LoginType = 'password' | 'code'

/** 登录请求 */
export interface LoginRequest {
  login: string      // 邮箱或用户名
  password?: string
  verification_code?: string
  login_type: LoginType
}

/** 注册请求 */
export interface RegisterRequest {
  email: string
  password: string
  username: string
}

/** 验证码请求 */
export interface SendCodeRequest {
  email: string
  type: 'login' | 'register' | 'reset_password'
}

/** 认证响应数据 */
export interface AuthData {
  success: boolean
  user_info?: UserInfo
  session_data?: SessionData
  message?: string
}

/** 用户基本信息 */
export interface UserInfo {
  id: number
  email: string
  username: string
  nickname?: string
  avatar_url?: string | null
  is_active: boolean
  created_at?: string
  updated_at?: string
}

/** Session 数据 */
export interface SessionData {
  api_key: string
  expires_at?: string
}

/** 验证邮箱请求 */
export interface VerifyEmailRequest {
  email: string
  code: string
}

/** 重置密码请求 */
export interface ResetPasswordRequest {
  email: string
  code: string
  new_password: string
}

// API 响应类型
export type LoginResponse = ApiResponse<AuthData>
export type RegisterResponse = ApiResponse<AuthData>
export type SendCodeResponse = ApiResponse<{ sent: boolean }>
export type VerifyEmailResponse = ApiResponse<{ verified: boolean }>
export type ResetPasswordResponse = ApiResponse<{ reset: boolean }>