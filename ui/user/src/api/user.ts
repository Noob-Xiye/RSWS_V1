import request, { type ApiResponse } from './request'

// ========== 用户相关类型 ==========

/** 用户信息（对齐后端 UserInfo） */
export interface UserInfo {
  id: number
  email: string
  username: string
  nickname: string
  avatar_url: string | null
  is_active: boolean
}

/** 登录请求（对齐后端 LoginRequest） */
export interface LoginRequest {
  login_type: 'password' | 'code'
  username?: string
  password?: string
  email?: string
  verification_code?: string
}

/** 登录响应（对齐后端 LoginResponse 扁平结构） */
export interface LoginResponse {
  user?: UserInfo
  api_key?: string
  expires_at?: string
}

/** 注册请求（对齐后端 RegisterRequest） */
export interface RegisterRequest {
  username: string
  nickname: string
  email: string
  password: string
  verification_code: string
}

/** 注册响应（注册成功自动登录，结构同 LoginResponse） */
export interface RegisterResponse {
  user?: UserInfo
  api_key?: string
  expires_at?: string
  message?: string
}

/** 更新资料请求（对齐后端 UpdateProfileRequest） */
export interface UpdateProfileRequest {
  nickname?: string
  avatar_url?: string
}

/** 修改密码请求（对齐后端 ChangePasswordRequest） */
export interface ChangePasswordRequest {
  old_password: string
  new_password: string
}

/** 发送验证码请求（对齐后端 SendVerificationCodeRequest） */
export interface SendCodeRequest {
  email: string
  code_type: 'register' | 'login' | 'reset_password'
}

// ========== API 函数 ==========

/** 用户登录 */
export async function login(data: LoginRequest): Promise<ApiResponse<LoginResponse>> {
  return request.post('/user/login', data)
}

/** 用户注册 */
export async function register(data: RegisterRequest): Promise<ApiResponse<RegisterResponse>> {
  return request.post('/user/register', data)
}

/** 获取当前用户信息 */
export async function getUserInfo(): Promise<ApiResponse<UserInfo>> {
  return request.get('/user/info')
}

/** 更新用户资料 */
export async function updateProfile(data: UpdateProfileRequest): Promise<ApiResponse<UserInfo>> {
  return request.put('/user/profile', data)
}

/** 上传头像（base64 data URI） */
export async function uploadAvatar(avatarData: string): Promise<ApiResponse<{ avatar_url: string }>> {
  return request.post('/user/avatar', { avatar_data: avatarData })
}

/** 修改密码 */
export async function changePassword(data: ChangePasswordRequest): Promise<ApiResponse<{ message: string }>> {
  return request.post('/user/change-password', data)
}

/** 发送验证码 */
export async function sendVerificationCode(data: SendCodeRequest): Promise<ApiResponse<{ message: string }>> {
  return request.post('/user/send-code', data)
}
