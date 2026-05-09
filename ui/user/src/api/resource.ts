import request, { type ApiResponse } from './request'

export interface Resource {
  id: number
  user_id?: number
  title: string
  description: string | null
  price: number  // 后端用 i64，前端转为 number
  category_id: number | null
  file_url: string | null
  thumbnail_url: string | null
  cover_image?: string | null
  is_active: boolean
  detail_description?: string | null
  specifications?: any
  usage_guide?: string | null
  precautions?: string | null
  display_images?: string[]
  download_count: number
  provider_type?: string
  commission_rate?: number
  created_at: string
  updated_at: string
}

export interface ResourceListResponse {
  items: Resource[]
  total: number
  page: number
  page_size: number
  total_pages: number
}

// 资源详情（购买后才有的字段）
export type ResourceDetail = Omit<Resource, 'file_url'> & {
  file_url?: string | null
}

export interface PurchaseCheckResponse {
  purchased: boolean
  order_id?: number
}

export interface DownloadInfo {
  file_url: string
  file_name: string
}

export async function listResources(params?: {
  page?: number
  page_size?: number
  category_id?: number
  search?: string
}): Promise<ApiResponse<ResourceListResponse>> {
  return request.get('/resource', { params })
}

export async function getResource(id: number): Promise<ApiResponse<ResourceDetail>> {
  return request.get(`/resource/${id}`)
}

/**
 * 检查用户是否已购买某资源
 */
export async function checkPurchase(resourceId: number): Promise<ApiResponse<PurchaseCheckResponse>> {
  return request.get(`/resource/${resourceId}/purchase-check`)
}

/**
 * 获取资源下载信息（需要已购买）
 */
export async function getDownloadInfo(resourceId: number): Promise<ApiResponse<DownloadInfo>> {
  return request.get(`/resource/${resourceId}/download`)
}

// 以下为管理端 API（普通用户一般不直接调用）
export async function createResource(data: Partial<Resource>): Promise<ApiResponse<Resource>> {
  return request.post('/resource', data)
}

export async function updateResource(id: number, data: Partial<Resource>): Promise<ApiResponse<Resource>> {
  return request.put(`/resource/${id}`, data)
}

export async function deleteResource(id: number): Promise<ApiResponse<{ id: number }>> {
  return request.delete(`/resource/${id}`)
}