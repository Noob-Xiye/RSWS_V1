// ========== API 签名工具 (Cregis 方案) ==========
import CryptoJS from 'crypto-js'

/**
 * 生成 API 签名 (Cregis 方案)
 * 
 * 算法：
 * 1. 收集所有参数（排除 sign 本身）
 * 2. 按 key ASCII 升序排序
 * 3. 拼接参数字符串：apiKey + key1 + value1 + key2 + value2 + ...
 * 4. MD5 计算并转小写 hex
 * 
 * @param params 参数字典（排除 sign）
 * @param apiKey 签名密钥（不随请求传输）
 * @returns MD5 小写 hex 签名
 */
export function generateSignature(params: Record<string, string>, apiKey: string): string {
  // 1. 获取所有 key（排除 sign），排序
  const keys = Object.keys(params)
    .filter(key => key !== 'sign')
    .sort()
  
  // 2. 按 ASCII 顺序拼接 key + value
  const paramStr = keys.map(key => key + params[key]).join('')
  
  // 3. apiKey 拼在前面（Cregis 方案）
  const signStr = apiKey + paramStr
  
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
 * 请求签名参数接口
 */
export interface SignParams {
  /** 公开用户标识符（随请求传输） */
  userId: string
  /** 签名密钥（不随请求传输） */
  apiKey: string
  /** 请求路径（不含 query params，用于防路径篡改） */
  path?: string
}

/**
 * 生成签名请求参数
 * 
 * 只签名 query params（user_id, timestamp, nonce + 其他业务查询参数）
 * 不签名 body —— 后端 auth 中间件只从 query params 验签
 * 
 * @param options 包含 userId, apiKey
 * @returns 签名参数对象 { user_id, timestamp, nonce, sign }
 * 
 * 注意：当前不包含 path 参数签名。如需启用 path 签名防重放到其他端点，
 * 需前后端同步改造（后端 auth 中间件也要加 path 验证）
 */
export function generateSignParams(options: SignParams): Record<string, string> {
  const timestamp = getTimestamp()
  const nonce = generateNonce()
  
  // 构建参数字典（query params + _path 防路径篡改）
  const params: Record<string, string> = {
    user_id: options.userId,
    timestamp: timestamp.toString(),
    nonce: nonce,
  }
  
  // 包含路径参数（防路径篡改）
  if (options.path) {
    params._path = options.path
  }
  
  // 计算签名（使用 apiKey 作为签名密钥）
  const sign = generateSignature(params, options.apiKey)
  
  return {
    user_id: options.userId,
    timestamp: timestamp.toString(),
    nonce: nonce,
    sign: sign,
  }
}

/**
 * 构建带签名的 URL
 * 
 * @param baseUrl 基础 URL
 * @param path 路径
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
