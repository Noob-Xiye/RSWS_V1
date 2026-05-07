import request, { type ApiResponse, type PaginatedResponse, type PaginationParams } from './request'

export interface SystemLog {
  id: number
  level: 'info' | 'warn' | 'error'
  category: string
  message: string
  details: string | null
  user_id: number | null
  admin_id: number | null
  created_at: string
}

export interface LogConfig {
  key: string
  value: string
  description: string
  is_active: boolean
}

export interface LogListParams extends PaginationParams {
  level?: string
  category?: string
  start_date?: string
  end_date?: string
}

// 查询系统日志
export async function querySystemLogs(params?: LogListParams): Promise<ApiResponse<PaginatedResponse<SystemLog>>> {
  return request.get('/admin/logs/system', { params })
}

// 日志配置管理
export async function listLogConfigs(): Promise<ApiResponse<LogConfig[]>> {
  return request.get('/admin/log-configs')
}

export async function getLogConfig(key: string): Promise<ApiResponse<LogConfig>> {
  return request.get(`/admin/log-configs/${key}`)
}

export async function createLogConfig(data: { key: string; value: string; description?: string }): Promise<ApiResponse<LogConfig>> {
  return request.post('/admin/log-configs', data)
}

export async function updateLogConfig(key: string, data: { value: string; description?: string; is_active?: boolean }): Promise<ApiResponse<LogConfig>> {
  return request.put(`/admin/log-configs/${key}`, data)
}

export async function deleteLogConfig(key: string): Promise<ApiResponse<void>> {
  return request.delete(`/admin/log-configs/${key}`)
}