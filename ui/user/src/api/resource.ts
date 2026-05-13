import request, { type ApiResponse } from './request'

export interface Resource {
  id: number
  title: string
  description: string | null
  price: number
  category_id: number | null
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

/** 资源详情（购买后有 file_url） */
export type ResourceDetail = Resource & {
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

/** 获取资源列表（公开端点） */
export async function listResources(params?: {
  page?: number
  page_size?: number
  category_id?: number
  search?: string
}): Promise<ApiResponse<ResourceListResponse>> {
  return request.get('/resource', { params })
}

/** 获取资源详情（公开端点） */
export async function getResource(id: number): Promise<ApiResponse<ResourceDetail>> {
  return request.get(`/resource/${id}`)
}

/** 检查用户是否已购买某资源 */
export async function checkPurchase(resourceId: number): Promise<ApiResponse<PurchaseCheckResponse>> {
  return request.get(`/resource/${resourceId}/purchase-check`)
}

/** 获取资源下载信息（需要已购买） */
export async function getDownloadInfo(resourceId: number): Promise<ApiResponse<DownloadInfo>> {
  return request.get(`/resource/${resourceId}/download`)
}
