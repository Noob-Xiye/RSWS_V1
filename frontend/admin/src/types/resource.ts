// ========== 资源类型 ==========

import type { ApiResponse, PaginatedData } from './api'

/** 资源状态 */
export type ResourceStatus = 'draft' | 'pending' | 'approved' | 'rejected' | 'hidden'

/** 资源信息 */
export interface Resource {
  id: number
  title: string
  description: string
  price: string  // 单位: 分
  category_id: number
  category_name?: string
  cover_image?: string | null
  file_url?: string
  file_size?: number  // 字节
  download_count: number
  status: ResourceStatus
  creator_id: number
  creator_name?: string
  tags?: string[]
  created_at: string
  updated_at: string
  published_at?: string
}

/** 资源列表数据 */
export interface ResourceListData extends PaginatedData<Resource> {}
export type ResourceListResponse = ApiResponse<ResourceListData>

/** 资源查询参数 */
export interface ResourceListParams {
  page?: number
  page_size?: number
  title?: string
  category_id?: number
  status?: ResourceStatus
  creator_id?: number
  start_date?: string
  end_date?: string
}

/** 资源审核操作 */
export interface ResourceAuditRequest {
  status: 'approved' | 'rejected'
  reason?: string
}

/** 资源统计 */
export interface ResourceStats {
  total: number
  pending: number
  approved: number
  rejected: number
  downloads_today: number
  downloads_total: number
}

/** 分类信息 */
export interface Category {
  id: number
  name: string
  parent_id?: number
  icon?: string
  sort_order: number
  is_active: boolean
  resource_count?: number
  created_at: string
}

export type CategoryListResponse = ApiResponse<Category[]>