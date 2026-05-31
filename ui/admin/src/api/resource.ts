import request, { type ApiResponse, type PaginatedResponse, type PaginationParams } from './request'
import { adminListCategories, type Category } from './category'

export interface Resource {
  id: number
  user_id: number
  title: string
  description: string | null
  price: number
  category_id: number | null
  file_url: string | null
  thumbnail_url: string | null
  is_active: boolean
  detail_description: string | null
  specifications: any
  usage_guide: string | null
  precautions: string | null
  display_images: string[] | null
  supported_os: string[] | null
  provider_type: string
  provider_id: number | null
  commission_rate: number
  download_count: number
  created_at: string
  updated_at: string
}

/** 创建/更新资源请求体 */
export interface CreateResourceRequest {
  title: string
  description?: string | null
  price: number
  category_id?: number | null
  file_url?: string | null
  thumbnail_url?: string | null
  detail_description?: string | null
  specifications?: Record<string, any> | null
  usage_guide?: string | null
  precautions?: string | null
  display_images?: string[] | null
  supported_os?: string[] | null
  is_active?: boolean
}

/** 资源列表项（含关联的展示字段） */
export interface ResourceListItem extends Resource {
  /** category_name 由前端从分类列表关联填充 */
  category_name?: string
}

export interface ResourceListParams extends PaginationParams {
  category_id?: number
  search?: string
}

/** 获取分类下拉选项（供资源筛选使用） */
export async function getCategoryOptions(): Promise<Category[]> {
  const res = await adminListCategories()
  if (res.code === 0 && res.data) {
    return res.data.categories
  }
  return []
}

// 获取资源列表（管理员，全部资源）
export async function listPlatformResources(params?: ResourceListParams): Promise<ApiResponse<PaginatedResponse<Resource>>> {
  return request.get('/admin/resources', { params })
}

// 获取资源列表
export async function listResources(params?: ResourceListParams): Promise<ApiResponse<PaginatedResponse<Resource>>> {
  return request.get('/resource', { params })
}

// 获取资源详情
export async function getResource(id: number): Promise<ApiResponse<Resource>> {
  return request.get(`/resource/${id}`)
}

// 删除资源（软删除）
export async function deleteResource(id: number): Promise<ApiResponse<void>> {
  return request.delete(`/resource/${id}`)
}

// 切换资源上下架状态（用户端，更新资源自身）
export async function toggleResourceActive(id: number, is_active: boolean): Promise<ApiResponse<Resource>> {
  return request.put(`/resource/${id}`, { is_active })
}

// 管理员切换资源上下架状态
export async function togglePlatformResourceActive(id: number): Promise<ApiResponse<Resource>> {
  return request.put(`/admin/resources/${id}/toggle-active`)
}

// 创建资源
export async function createResource(data: Partial<CreateResourceRequest>): Promise<ApiResponse<Resource>> {
  return request.post('/resource', data)
}

// 更新资源
export async function updateResource(id: number, data: Partial<CreateResourceRequest>): Promise<ApiResponse<Resource>> {
  return request.put(`/resource/${id}`, data)
}

// 管理员创建平台资源
export async function createPlatformResource(data: Partial<CreateResourceRequest>): Promise<ApiResponse<Resource>> {
  return request.post('/admin/resources', data)
}

// 管理员更新资源
export async function updatePlatformResource(id: number, data: Partial<CreateResourceRequest>): Promise<ApiResponse<Resource>> {
  return request.put(`/admin/resources/${id}`, data)
}

// 管理员删除资源
export async function deletePlatformResource(id: number): Promise<ApiResponse<void>> {
  return request.delete(`/admin/resources/${id}`)
}

// 管理员切换资源上下架
export async function togglePlatformResourceActive(id: number): Promise<ApiResponse<Resource>> {
  return request.put(`/admin/resources/${id}/toggle-active`)
}
