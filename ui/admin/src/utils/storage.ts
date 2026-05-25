// ========== 本地存储工具 ==========
//
// Cregis 单密钥方案：
// - api_key: 签名密钥，仅本地存储用于计算签名，不随请求传输
// - admin_id: 公开标识符，随请求传输（后端统一用 user_id 字段）

const KEY_PREFIX = 'rsws_admin_'

// API Key（用于签名，不随请求传输）
export function setApiKey(key: string): void {
  localStorage.setItem(`${KEY_PREFIX}api_key`, key)
}

export function getApiKey(): string | null {
  return localStorage.getItem(`${KEY_PREFIX}api_key`)
}

export function removeApiKey(): void {
  localStorage.removeItem(`${KEY_PREFIX}api_key`)
}

// Admin ID（公开标识符，随请求传输）
export function setAdminId(adminId: string): void {
  localStorage.setItem(`${KEY_PREFIX}admin_id`, adminId)
}

export function getAdminId(): string | null {
  return localStorage.getItem(`${KEY_PREFIX}admin_id`)
}

export function removeAdminId(): void {
  localStorage.removeItem(`${KEY_PREFIX}admin_id`)
}

// Admin Info（登录后获取的管理员信息，用于显示名称等）
export function setAdminInfo(info: Record<string, unknown>): void {
  localStorage.setItem(`${KEY_PREFIX}admin_info`, JSON.stringify(info))
}

export function getAdminInfo(): Record<string, unknown> | null {
  const raw = localStorage.getItem(`${KEY_PREFIX}admin_info`)
  if (!raw) return null
  try { return JSON.parse(raw) } catch { return null }
}

export function removeAdminInfo(): void {
  localStorage.removeItem(`${KEY_PREFIX}admin_info`)
}

// 清除所有存储
export function clearAll(): void {
  localStorage.removeItem(`${KEY_PREFIX}api_key`)
  localStorage.removeItem(`${KEY_PREFIX}admin_id`)
  localStorage.removeItem(`${KEY_PREFIX}admin_info`)
}