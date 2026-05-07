export function setApiKey(key: string): void {
  localStorage.setItem('rsws_user_api_key', key)
}

export function getApiKey(): string | null {
  return localStorage.getItem('rsws_user_api_key')
}

export function removeApiKey(): void {
  localStorage.removeItem('rsws_user_api_key')
}