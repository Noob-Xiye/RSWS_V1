import request, { type ApiResponse } from './request'

// 邮件配置（对齐后端 email_configs 表）
export interface EmailConfig {
  provider: string
  host: string
  port: number
  username: string
  use_tls: boolean
  from_email: string
  from_name: string
  reply_to: string | null
}

export interface UpdateEmailConfigRequest {
  provider?: string
  host?: string
  port?: number
  username?: string
  password?: string  // 仅更新时传，不回传
  use_tls?: boolean
  from_email?: string
  from_name?: string
  reply_to?: string
}

// 获取邮件配置（不含密码）
export async function getEmailConfig(): Promise<ApiResponse<EmailConfig>> {
  return request.get('/admin/email-configs')
}

// 更新邮件配置（upsert）
export async function updateEmailConfig(data: UpdateEmailConfigRequest): Promise<ApiResponse<void>> {
  return request.put('/admin/email-configs', data)
}
