// ========== API 签名工具 (Cregis 方案) ==========
import CryptoJS from 'crypto-js'

/**
 * 生成 API 签名 (Cregis 方案)
 * 
 * 算法：
 * 1. 收集所有参数（排除 sign 本身）
 * 2. 按 key ASCII 升序排序
 * 3. 拼接参数字符串：api_secret + key1 + value1 + key2 + value2 + ...
 * 4. MD5 计算并转小写 hex
 */
export function generateSignature(params: Record<string, string>, apiSecret: string): string {
  // 1. 获取所有 key（排除 sign），排序
  const keys = Object.keys(params)
    .filter(key => key !== 'sign')
    .sort()
  
  // 2. 按 ASCII 顺序拼接 key + value
  const paramStr = keys.map(key => key + params[key]).join('')
  
  // 3. 拼接 api_secret（拼在前面，与 Cregis 方案一致）
  const signStr = apiSecret + paramStr
  
  // 4. MD5 + 小写 hex
  return CryptoJS.MD5(signStr).toString()
}

/**
 * 获取当前时间戳 (毫秒)
 */
export function getTimestamp(): number {
  return Date.now()
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
 * 请求签名参数
 */
export interface SignParams {
  apiKey: string
  apiSecret: string
  path: string
  body?: Record<string, unknown>
}

/**
 * 生成签名请求参数
 * 
 * @param options 包含 apiKey, apiSecret, path, body
 * @returns 签名参数对象 { api_key, timestamp, nonce, sign }
 */
export function generateSignParams(options: SignParams): Record<string, string> {
  const timestamp = getTimestamp()
  const nonce = generateNonce()
  
  // 构建参数字典（包含 api_key, timestamp, nonce）
  const params: Record<string, string> = {
    api_key: options.apiKey,
    timestamp: timestamp.toString(),
    nonce: nonce,
  }
  
  // 如果有 body，添加 body 参数
  if (options.body && Object.keys(options.body).length > 0) {
    // 注意：后端会从查询参数收集其他参数，这里将 body 转换为 key-value 字符串
    // 实际使用时，可能需要将 body 序列化后作为参数传递
    const bodyStr = JSON.stringify(options.body)
    params.body = bodyStr
  }
  
  // 计算签名
  const sign = generateSignature(params, options.apiSecret)
  
  return {
    api_key: options.apiKey,
    timestamp: timestamp.toString(),
    nonce: nonce,
    sign: sign,
  }
}

/**
 * 构建带签名的 URL
 * 
 * @param baseUrl 基础 URL
 * @param params 签名参数
 * @returns 带查询参数的完整 URL
 */
export function buildSignedUrl(baseUrl: string, path: string, params: Record<string, string>): string {
  const url = new URL(path, baseUrl)
  Object.entries(params).forEach(([key, value]) => {
    url.searchParams.append(key, value)
  })
  return url.toString()
}
