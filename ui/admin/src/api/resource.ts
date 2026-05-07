import request, { type ApiResponse, type PaginatedResponse, type PaginationParams } from './request'

export interface Resource {
  id: number
  title: string
  description: string
  price: string
  category: string
  status: 'draft' | 'pending' | 'approved' | 'rejected'
  creator_id: number
  creator_name: string
  download_count: number
  created_at: string
  updated_at: string
}

export interface ResourceListParams extends PaginationParams {
  title?: string
  category?: string
  status?: string
  creator_id?: number
}

// 获取资源列表
export async function listResources(params?: ResourceListParams): Promise<ApiResponse<PaginatedResponse<Resource>>> {
  return request.get('/resource', { params })
}

// 获取资源详情
export async function getResource(id: number): Promise<ApiResponse<Resource>> {
  return request.get(`/resource/${id}`)
}

// 更新资源状态（审核）
export async function updateResourceStatus(id: number, status: 'approved' | 'rejected', reason?: string): Promise<ApiResponse<Resource>> {
  return request.put(`/resource/${id}`, { status, reason })
}

// 删除资源
export async function deleteResource(id: number): Promise<ApiResponse<void>> {
  return request.delete(`/resource/${id}`)
}