import request, { type ApiResponse } from './request'

/** 分类信息 */
export interface Category {
  id: number
  name: string
  description: string | null
  parent_id: number | null
  path: string | null
  sort_order: number
  is_active: boolean
  resource_count?: number
  created_at: string
  updated_at: string | null
}

/** 创建分类请求 */
export interface CreateCategoryRequest {
  name: string
  description?: string
  parent_id?: number | null
  sort_order?: number
}

/** 更新分类请求 */
export interface UpdateCategoryRequest {
  name?: string
  description?: string
  parent_id?: number | null
  sort_order?: number
  is_active?: boolean
}

/** 排序项 */
export interface SortItem {
  id: number
  sort_order: number
}

// ========== 公开端点 ==========

/** 获取活跃分类列表（用户端用） */
export async function listCategories(): Promise<ApiResponse<{ categories: Category[] }>> {
  return request.get('/categories')
}

// ========== 管理端点 ==========

/** 获取所有分类（含已停用，管理员用） */
export async function adminListCategories(): Promise<ApiResponse<{ categories: Category[] }>> {
  return request.get('/admin/categories')
}

/** 创建分类 */
export async function createCategory(data: CreateCategoryRequest): Promise<ApiResponse<Category>> {
  return request.post('/admin/categories', data)
}

/** 更新分类 */
export async function updateCategory(id: number, data: UpdateCategoryRequest): Promise<ApiResponse<Category>> {
  return request.put(`/admin/categories/${id}`, data)
}

/** 删除分类 */
export async function deleteCategory(id: number): Promise<ApiResponse<void>> {
  return request.delete(`/admin/categories/${id}`)
}

/** 批量更新排序 */
export async function batchUpdateSort(orders: SortItem[]): Promise<ApiResponse<void>> {
  return request.put('/admin/categories/sort', { orders })
}
