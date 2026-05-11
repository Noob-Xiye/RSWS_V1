import request, { type ApiResponse } from './request'

// ========== Admin 管理员 ==========
export interface AdminInfo {
  id: number
  email: string
  username: string
  avatar_url?: string
  role: string
  permissions: string[]
}

export interface AdminListResponse {
  items: AdminInfo[]
  total: number
  page: number
  page_size: number
  total_pages: number
}

// 登录响应（匹配后端 AdminLoginResponse）
export interface LoginResponse {
  admin: AdminInfo
  token: string
  expires_at: string
}

export interface CreateAdminParams {
  email: string
  username: string
  password: string
  role: string
}

export interface ApiKeyInfo {
  id: number
  key: string
  name: string
  is_active: boolean
  created_at: string
  expires_at: string | null
}

export interface AdminApiKeyResponse {
  id: number
  name: string
  api_key: string
  permissions: string[]
  rate_limit: number | null
  expires_at: string | null
  is_active: boolean
  created_at: string
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
export async function listAdmins(params?: { page?: number; page_size?: number; role?: string }): Promise<ApiResponse<AdminListResponse>> {
  return request.get('/admin/list', { params })
}

// 创建管理员
export async function createAdmin(data: CreateAdminParams): Promise<ApiResponse<AdminInfo>> {
  return request.post('/admin/create', data)
}

// 停用/启用管理员
export async function deactivateAdmin(id: number): Promise<ApiResponse<void>> {
  return request.post(`/admin/${id}/deactivate`)
}

export async function activateAdmin(id: number): Promise<ApiResponse<void>> {
  return request.post(`/admin/${id}/activate`)
}

export async function resetAdminPassword(id: number, newPassword: string): Promise<ApiResponse<void>> {
  return request.post(`/admin/${id}/reset-password`, { password: newPassword })
}

// ========== API Key 管理 ==========
export async function listApiKeys(): Promise<ApiResponse<AdminApiKeyResponse[]>> {
  return request.get('/admin/api-keys')
}

export async function createApiKey(data: { name: string; permissions?: string[]; rate_limit?: number; expires_in_days?: number }): Promise<ApiResponse<AdminApiKeyResponse>> {
  return request.post('/admin/api-keys', data)
}

export async function deleteApiKey(keyId: number): Promise<ApiResponse<{ deleted: boolean }>> {
  return request.delete(`/admin/${keyId}/api-keys`)
}

export async function toggleApiKey(keyId: number, isActive: boolean): Promise<ApiResponse<void>> {
  return request.put(`/admin/api-keys/${keyId}`, { is_active: isActive })
}
// ========== USDT 钱包管理 ==========
export interface UsdtWallet {
  network: string
  address: string
  created_at: string
  updated_at: string
}

// 列出所有 USDT 钱包
export async function listUsdtWallets(): Promise<ApiResponse<UsdtWallet[]>> {
  return request.get('/admin/usdt-wallets')
}

// 更新 USDT 钱包地址（按 network）
export async function updateUsdtWallet(network: string, address: string): Promise<ApiResponse<UsdtWallet>> {
  return request.put(`/admin/usdt-wallets/${network}`, { address })
}
