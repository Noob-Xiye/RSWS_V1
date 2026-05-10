// ========== 分类类型 (User 端) ==========

import type { ApiResponse } from './api'

/** 分类信息 */
export interface Category {
  id: number
  name: string
  parent_id?: number
  icon?: string
  sort_order: number
  is_active: boolean
  resource_count?: number
}

export type CategoryListResponse = ApiResponse<Category[]>

/** 分类树节点 (用于级联选择) */
export interface CategoryTree extends Category {
  children?: CategoryTree[]
}