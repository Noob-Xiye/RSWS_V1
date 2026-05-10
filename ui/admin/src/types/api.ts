// ========== API 通用类型 ==========

/** API 响应基础结构 */
export interface ApiResponse<T = unknown> {
  success: boolean
  data?: T
  message?: string
  code?: number
}

/** 带分页的响应 */
export interface PaginatedData<T> {
  items: T[]
  total: number
  page: number
  page_size: number
  total_pages: number
}

export type PaginatedResponse<T> = ApiResponse<PaginatedData<T>>

/** 分页查询参数 */
export interface PaginationParams {
  page?: number
  page_size?: number
}

/** 通用列表查询参数 */
export interface ListParams extends PaginationParams {
  search?: string
  start_date?: string
  end_date?: string
  sort_by?: string
  sort_order?: 'asc' | 'desc'
}

/** 布尔值字符串转换 */
export type BooleanString = 'true' | 'false'

/** 时间戳字符串 (ISO 8601) */
export type Timestamp = string