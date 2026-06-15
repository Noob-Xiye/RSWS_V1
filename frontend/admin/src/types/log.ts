// ========== 日志类型 ==========

import type { ApiResponse, PaginatedData } from './api'

/** 日志级别 */
export type LogLevel = 'debug' | 'info' | 'warn' | 'error'

/** 日志模块 */
export type LogModule = 'auth' | 'payment' | 'resource' | 'order' | 'user' | 'system' | 'api'

/** 系统日志 */
export interface SystemLog {
  id: number
  level: LogLevel
  module: LogModule
  message: string
  details?: string
  user_id?: number
  ip_address?: string
  user_agent?: string
  created_at: string
}

/** 操作日志 */
export interface OperationLog {
  id: number
  admin_id: number
  admin_name: string
  action: string
  target_type: string
  target_id: number
  details?: string
  ip_address: string
  created_at: string
}

/** 请求日志 */
export interface RequestLog {
  id: number
  method: string
  path: string
  ip_address: string
  user_agent?: string
  api_key?: string
  status_code: number
  response_time: number  // 毫秒
  created_at: string
}

/** 支付日志 */
export interface PaymentLog {
  id: number
  order_id: number
  order_no: string
  payment_method: string
  amount: string
  status: string
  transaction_id?: string
  tx_hash?: string
  created_at: string
}

/** Webhook 日志 */
export interface WebhookLog {
  id: number
  source: string  // 'paypal' | 'tron' | 'ethereum'
  event_type: string
  event_id?: string
  data?: string
  status: 'pending' | 'processed' | 'failed'
  error_message?: string
  retry_count: number
  created_at: string
  processed_at?: string
}

// 日志列表数据
export interface SystemLogListData extends PaginatedData<SystemLog> {}
export interface OperationLogListData extends PaginatedData<OperationLog> {}
export interface RequestLogListData extends PaginatedData<RequestLog> {}
export interface PaymentLogListData extends PaginatedData<PaymentLog> {}
export interface WebhookLogListData extends PaginatedData<WebhookLog> {}

export type SystemLogListResponse = ApiResponse<SystemLogListData>
export type OperationLogListResponse = ApiResponse<OperationLogListData>
export type RequestLogListResponse = ApiResponse<RequestLogListData>
export type PaymentLogListResponse = ApiResponse<PaymentLogListData>
export type WebhookLogListResponse = ApiResponse<WebhookLogListData>

/** 日志查询参数 */
export interface LogListParams {
  page?: number
  page_size?: number
  level?: LogLevel
  module?: LogModule
  start_date?: string
  end_date?: string
  search?: string
}

export interface OperationLogListParams extends LogListParams {
  admin_id?: number
  action?: string
}

export interface RequestLogListParams extends LogListParams {
  method?: string
  path?: string
  status_code?: number
}

export interface PaymentLogListParams extends LogListParams {
  payment_method?: string
  status?: string
}