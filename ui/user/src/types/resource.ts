// ========== 资源类型 (User 端) ==========

import type { ApiResponse, PaginatedResponse, PaginatedData } from './api'

/** 资源信息 (用户端简化版) */
export interface Resource {
  id: number
  title: string
  description: string
  price: string  // 单位: 分
  category_id: number
  category_name?: string
  cover_image?: string | null
  file_url?: string
  file_size?: number
  download_count: number
  status: 'approved'
  tags?: string[]
  created_at: string
  updated_at: string
}

/** 资源列表数据 */
export interface ResourceListData extends PaginatedData<Resource> {}

/** 资源查询参数 */
export interface ResourceListParams {
  page?: number
  page_size?: number
  search?: string
  category_id?: number
  sort_by?: 'created_at' | 'download_count' | 'price'
  sort_order?: 'asc' | 'desc'
}

/** 资源详情响应 */
export interface ResourceDetail {
  id: number
  title: string
  description: string
  price: string
  category_id: number
  category_name: string
  cover_image?: string
  file_info?: {
    url: string
    size: number
    format: string
  }
  download_count: number
  creator: {
    id: number
    username: string
    avatar_url?: string
  }
  tags?: string[]
  created_at: string
  can_download: boolean  // 用户是否已购买
}

export type ResourceListResponse = ApiResponse<ResourceListData>
export type ResourceDetailResponse = ApiResponse<ResourceDetail>