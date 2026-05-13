import request, { type ApiResponse, type PaginatedResponse, type PaginationParams } from './request'

// 对齐后端 SystemLog 结构体字段名
export interface SystemLog {
  id: number
  log_level: string  // 后端字段名（不是 level）
  module: string     // 后端字段名（不是 category）
  message: string
  context: any | null  // 后端是 Option<serde_json::Value>（不是 details）
  user_id: number | null
  admin_id: number | null
  ip_address: string | null   // 新增
  user_agent: string | null   // 新增
  request_id: string | null   // 新增
  created_at: string
}

// LogConfig 对齐后端 log_configs 表结构
export interface LogConfig {
  config_key: string
  config_value: string
  config_type: string | null
  description: string | null
  is_active: boolean
  created_at?: string
  updated_at?: string
}

export interface LogListParams extends PaginationParams {
  level?: string     // 后端 query_system_logs handler 接收 level（但功能未完整）
  module?: string    // 后端 service 接收但未实现
  start_time?: string
  end_time?: string
}

// 创建日志配置请求（对齐后端 SetLogConfigBody）
export interface CreateLogConfigRequest {
  config_key: string
  config_value: string
  config_type?: string
  description?: string
}

// 更新日志配置请求（对齐后端 UpdateLogConfigBody）
export interface UpdateLogConfigRequest {
  config_value: string
  config_type?: string
  description?: string
  is_active?: boolean
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

export async function createLogConfig(data: CreateLogConfigRequest): Promise<ApiResponse<LogConfig>> {
  return request.post('/admin/log-configs', data)
}

export async function updateLogConfig(key: string, data: UpdateLogConfigRequest): Promise<ApiResponse<LogConfig>> {
  return request.put(`/admin/log-configs/${key}`, data)
}

export async function deleteLogConfig(key: string): Promise<ApiResponse<void>> {
  return request.delete(`/admin/log-configs/${key}`)
}