import request, { type ApiResponse } from './request'

export interface User {
  id: number
  email: string
  username: string
  nickname?: string
  avatar_url?: string | null
  is_active: boolean
  created_at?: string
  updated_at?: string
}

export interface LoginResponse {
  user: User
  api_key: string
  api_secret?: string
  expires_at?: string
}

export interface RegisterResponse {
  user: User
  api_key?: string
}

export interface UpdateProfileRequest {
  nickname?: string
  avatar_url?: string
}

export interface ChangePasswordRequest {
  old_password: string
  new_password: string
}

// 登录（使用邮箱+密码）
export async function login(email: string, password: string): Promise<ApiResponse<LoginResponse>> {
  return request.post('/user/login', {
    login_type: 'password',
    email,
    password,
  })
}

// 注册
export async function register(email: string, password: string, username: string): Promise<ApiResponse<RegisterResponse>> {
  return request.post('/user/register', {
    email,
    password,
    username,
    nickname: username, // 默认昵称等于用户名
  })
}

// 获取当前用户信息
export async function getUserInfo(): Promise<ApiResponse<User>> {
  return request.get('/user')
}

// 更新用户资料
export async function updateProfile(data: UpdateProfileRequest): Promise<ApiResponse<User>> {
  return request.put('/user/profile', data)
}

// 修改密码
export async function changePassword(data: ChangePasswordRequest): Promise<ApiResponse<{ message: string }>> {
  return request.put('/user/password', data)
}