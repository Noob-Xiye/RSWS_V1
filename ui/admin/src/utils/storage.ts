const TOKEN_KEY = 'rsws_admin_token'
const API_KEY_KEY = 'rsws_admin_api_key'

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
}

export function clearAll(): void {
  localStorage.removeItem(TOKEN_KEY)
  localStorage.removeItem(API_KEY_KEY)
}