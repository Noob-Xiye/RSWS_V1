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
  provider_type: string
  provider_id: number | null
  commission_rate: number
  download_count: number
  created_at: string
  updated_at: string
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

// 切换资源上下架状态
export async function toggleResourceActive(id: number, is_active: boolean): Promise<ApiResponse<Resource>> {
  return request.put(`/resource/${id}`, { is_active })
}
