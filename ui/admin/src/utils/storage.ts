// ========== 本地存储工具 ==========

const KEY_PREFIX = 'rsws_admin_'
const TOKEN_KEY = `${KEY_PREFIX}token`
const API_KEY_KEY = `${KEY_PREFIX}api_key`
const API_SECRET_KEY = `${KEY_PREFIX}api_secret`

export function setToken(token: string): void {
  localStorage.setItem(TOKEN_KEY, token)
}

export function getToken(): string | null {
  return localStorage.getItem(TOKEN_KEY)
}

export function removeToken(): void {
  localStorage.removeItem(TOKEN_KEY)
}

export function setApiKey(key: string): void {
  localStorage.setItem(API_KEY_KEY, key)
}

export function getApiKey(): string | null {
  return localStorage.getItem(API_KEY_KEY)
}

export function removeApiKey(): void {
  localStorage.removeItem(API_KEY_KEY)
  localStorage.removeItem(API_SECRET_KEY)
}

// 单独移除 API Secret
// 注意：removeApiKey 会同时移除 secret，如果只需要移除 secret 使用此函数
export function removeApiSecret(): void {
  localStorage.removeItem(API_SECRET_KEY)
}

// API Secret 用于签名
export function setApiSecret(secret: string): void {
  localStorage.setItem(API_SECRET_KEY, secret)
}

export function getApiSecret(): string | null {
  return localStorage.getItem(API_SECRET_KEY)
}

// 清除所有存储
export function clearAll(): void {
  localStorage.removeItem(TOKEN_KEY)
  localStorage.removeItem(API_KEY_KEY)
  localStorage.removeItem(API_SECRET_KEY)
}