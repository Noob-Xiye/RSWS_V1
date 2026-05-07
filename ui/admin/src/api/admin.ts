import request, { type ApiResponse } from './request'

export interface AdminInfo {
  id: number
  email: string
  username: string
  role: string
  is_active: boolean
  created_at: string
}

export interface LoginRequest {
  email: string
  password: string
}

export interface LoginResponse {
  token: string
  api_key: string
  admin: AdminInfo
}

export interface ApiKeyInfo {
  id: number
  key: string
  name: string
  is_active: boolean
  created_at: string
  expires_at: string | null
}

// 管理员登录（不需要 API Key）
export async function adminLogin(email: string, password: string): Promise<ApiResponse<LoginResponse>> {
  return request.post('/admin/login', { email, password })
}

// 获取当前管理员信息
export async function getAdminInfo(): Promise<ApiResponse<AdminInfo>> {
  return request.get('/admin')
}

// 获取管理员列表
export async function listAdmins(params?: { page?: number; page_size?: number }): Promise<ApiResponse<AdminInfo[]>> {
  return request.get('/admin/list', { params })
}

// 创建管理员
export async function createAdmin(data: { email: string; password: string; username?: string; role?: string }): Promise<ApiResponse<AdminInfo>> {
  return request.post('/admin/create', data)
}

// 停用管理员
export async function deactivateAdmin(id: number): Promise<ApiResponse<void>> {
  return request.post(`/admin/${id}/deactivate`)
}

// API Key 管理
export async function listApiKeys(): Promise<ApiResponse<ApiKeyInfo[]>> {
  return request.get('/admin/api-keys')
}

export async function createApiKey(data: { name: string; expires_at?: string }): Promise<ApiResponse<ApiKeyInfo>> {
  return request.post('/admin/api-keys', data)
}

export async function deleteApiKey(keyId: number): Promise<ApiResponse<void>> {
  return request.delete(`/admin/api-keys/${keyId}`)
}