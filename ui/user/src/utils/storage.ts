// ========== 本地存储工具 ==========
//
// 新设计（Cregis 方案）：
// - 只存储 api_key（签名密钥），不存储 api_secret
// - 前端用 api_key 计算签名，请求中不传输 api_key 本身

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

// Token（如果使用 session token）
export function setToken(token: string): void {
  localStorage.setItem(`${KEY_PREFIX}token`, token)
}

export function getToken(): string | null {
  return localStorage.getItem(`${KEY_PREFIX}token`)
}

export function removeToken(): void {
  localStorage.removeItem(`${KEY_PREFIX}token`)
}

// 清除所有用户相关存储
export function clearAll(): void {
  localStorage.removeItem(`${KEY_PREFIX}api_key`)
  localStorage.removeItem(`${KEY_PREFIX}user_id`)
  localStorage.removeItem(`${KEY_PREFIX}token`)
}