// ========== 本地存储工具 ==========
//
// Cregis 单密钥方案：
// - api_key: 签名密钥，仅本地存储用于计算签名，不随请求传输
// - user_id: 公开标识符，随请求传输（后端用它查库获取 api_key 验签）

const KEY_PREFIX = 'rsws_user_'

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

// User ID（公开标识符，随请求传输）
export function setUserId(userId: string): void {
  localStorage.setItem(`${KEY_PREFIX}user_id`, userId)
}

export function getUserId(): string | null {
  return localStorage.getItem(`${KEY_PREFIX}user_id`)
}

export function removeUserId(): void {
  localStorage.removeItem(`${KEY_PREFIX}user_id`)
}

// 清除所有用户相关存储
export function clearAll(): void {
  localStorage.removeItem(`${KEY_PREFIX}api_key`)
  localStorage.removeItem(`${KEY_PREFIX}user_id`)
}
