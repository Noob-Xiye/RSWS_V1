// ========== API 签名工具 ==========
import CryptoJS from 'crypto-js'

/**
 * 生成 API 签名 (HMAC-SHA256)
 */
export function generateSignature(options: {
  method: string
  path: string
  timestamp: number
  nonce: string
  body?: unknown
  secret: string
}): string {
  const { method, path, timestamp, nonce, body, secret } = options
  
  // 构建签名字符串: method + path + timestamp + nonce + body
  const bodyStr = body ? JSON.stringify(body) : ''
  const message = `${method.toUpperCase()}${path}${timestamp}${nonce}${bodyStr}`
  
  // 使用 HMAC-SHA256
  return CryptoJS.HmacSHA256(message, secret).toString()
}

/**
 * 生成随机字符串
 */
export function generateNonce(length: number = 16): string {
  const chars = 'abcdefghijklmnopqrstuvwxyz0123456789'
  let result = ''
  for (let i = 0; i < length; i++) {
    result += chars.charAt(Math.floor(Math.random() * chars.length))
  }
  return result
}

/**
 * 获取当前时间戳 (毫秒)
 */
export function getTimestamp(): number {
  return Date.now()
}

/**
 * 请求头选项接口
 */
export interface RequestHeaders {
  'X-API-Key': string
  'X-Timestamp': string
  'X-Nonce': string
  'X-Signature': string
}

/**
 * 生成请求头 (同步)
 */
export function generateRequestHeaders(options: {
  apiKey: string
  apiSecret: string
  method: string
  path: string
  body?: unknown
}): RequestHeaders {
  const timestamp = getTimestamp()
  const nonce = generateNonce()
  const signature = generateSignature({
    method: options.method,
    path: options.path,
    timestamp,
    nonce,
    body: options.body,
    secret: options.apiSecret,
  })
  
  return {
    'X-API-Key': options.apiKey,
    'X-Timestamp': timestamp.toString(),
    'X-Nonce': nonce,
    'X-Signature': signature,
  }
}