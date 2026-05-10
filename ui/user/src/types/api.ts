// ========== API 通用类型 (User 端) ==========

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