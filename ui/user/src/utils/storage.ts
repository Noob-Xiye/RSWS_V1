// ========== 本地存储工具 ==========

const KEY_PREFIX = 'rsws_user_'

export function setApiKey(key: string): void {
  localStorage.setItem(`${KEY_PREFIX}api_key`, key)
}

export function getApiKey(): string | null {
  return localStorage.getItem(`${KEY_PREFIX}api_key`)
}

export function removeApiKey(): void {
  localStorage.removeItem(`${KEY_PREFIX}api_key`)
  localStorage.removeItem(`${KEY_PREFIX}api_secret`)
}

export function removeApiSecret(): void {
  localStorage.removeItem(`${KEY_PREFIX}api_secret`)
}

// API Secret 用于签名
export function setApiSecret(secret: string): void {
  localStorage.setItem(`${KEY_PREFIX}api_secret`, secret)
}

export function getApiSecret(): string | null {
  return localStorage.getItem(`${KEY_PREFIX}api_secret`)
}

// Token (如果使用 session token)
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
  localStorage.removeItem(`${KEY_PREFIX}api_secret`)
  localStorage.removeItem(`${KEY_PREFIX}token`)
}