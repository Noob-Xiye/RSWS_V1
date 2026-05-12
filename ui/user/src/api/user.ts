import request, { type ApiResponse } from './request'

// ========== 用户相关类型 ==========

export interface User {
  id: number
  email: string
  username: string
  nickname?: string
  avatar_url?: string | null
  balance?: string
  is_active: boolean
  created_at: string
  updated_at: string
}

export interface LoginRequest {
  login: string  // username 或 email
  password?: string
  verification_code?: string
  login_type: 'password' | 'code'
}

/**
 * 后端 LoginResponse（与管理员端 AdminLoginResponse 统一扁平结构）
 */
export interface LoginResponse {
  user?: Partial<User>
  api_key?: string
  expires_at?: string
}

export interface RegisterRequest {
  email: string
  password: string
  username: string
  verification_code?: string
}

/**
 * 后端 RegisterResponse（与 LoginResponse 统一结构）
 */
export interface RegisterResponse {
  user?: Partial<User>
  message?: string
}

export interface UpdateProfileRequest {
  nickname?: string
  avatar_url?: string
}

export interface ChangePasswordRequest {
  old_password: string
  new_password: string
}

export interface SendCodeRequest {
  email: string
  scene: 'register' | 'login' | 'reset_password'
}

// ========== API 函数 ==========

/**
 * 用户登录
 * 返回 ApiResponse<LoginResponse>，由 code === 0 判断成功
 */
export async function login(data: LoginRequest): Promise<ApiResponse<LoginResponse>> {
  return request.post('/user/login', data)
}

/**
 * 用户注册
 * 返回 ApiResponse<RegisterResponse>，由 code === 0 判断成功
 */
export async function register(data: RegisterRequest): Promise<ApiResponse<RegisterResponse>> {
  return request.post('/user/register', data)
}

/**
 * 获取当前用户信息
 */
export async function getUserInfo(): Promise<ApiResponse<User>> {
  return request.get('/user/info')
}

/**
 * 更新用户资料
 */
export async function updateProfile(data: UpdateProfileRequest): Promise<ApiResponse<User>> {
  return request.put('/user/profile', data)
}

/**
 * 修改密码
 */
export async function changePassword(data: ChangePasswordRequest): Promise<ApiResponse<{ message: string }>> {
  return request.post('/user/change-password', data)
}

/**
 * 发送验证码
 */
export async function sendVerificationCode(data: SendCodeRequest): Promise<ApiResponse<{ success: boolean; message: string }>> {
  return request.post('/user/send-code', data)
}
