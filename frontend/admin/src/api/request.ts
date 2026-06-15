import axios from 'axios'
import { getApiKey, getAdminId, removeApiKey, removeAdminId } from '@/utils/storage'
import { generateSignParams } from '@/utils/signature'

// API 基础地址
const BASE_URL = import.meta.env.VITE_API_URL || '/api/v1'

// 创建 axios 实例
const request = axios.create({
  baseURL: BASE_URL,
  timeout: 30000,
  headers: {
    'Content-Type': 'application/json'
  }
})

// 请求拦截器 - 添加 API Key 签名 (Cregis 方案)
request.interceptors.request.use(async (config) => {
  const apiKey = getApiKey()
  const adminId = getAdminId()
  
  if (apiKey && adminId) {
    // 生成签名参数 (Cregis 方案)
    // 注意：只传 admin_id（后端存为 user_id），不传 api_key
    // 注意：不传 body，后端只读 query params 签名
    // 提取请求路径（不含 query params）用于签名防篡改
    // axios 的 config.url 不含 baseURL，需要拼接完整路径以匹配后端 req.uri().path()
    const requestPath = (config.url ? `/api/v1/${config.url.replace(/^\//, '')}` : '')
    const signParams = generateSignParams({
      adminId,
      apiKey,
      path: requestPath,
    })
    
    // Debug: print sign calculation details
    console.log('[SIGN] apiKey:', apiKey, 'adminId:', adminId)
    console.log('[SIGN] signParams:', JSON.stringify(signParams))
    
    // 将签名参数添加到查询参数
    // 注意：不包含 api_key！api_key 只用于计算签名
    config.params = {
      ...config.params,
      user_id: signParams.user_id,
      timestamp: signParams.timestamp,
      nonce: signParams.nonce,
      sign: signParams.sign,
    }
  }
  
  return config
}, (error) => {
  return Promise.reject(error)
})

// 响应拦截器
request.interceptors.response.use(
  (response) => {
    const data = response.data
    // 检查业务层错误码（后端返回 200，但 code !== 0）
    if (data && data.code !== undefined && data.code !== 0) {
      // 401 = 未登录 / 认证失败 → 清除本地状态并跳转登录页
      if (data.code === 401 || (data.msg && data.msg.includes('未登录'))) {
        removeApiKey()
        removeAdminId()
        // 避免重复跳转
        if (window.location.pathname !== '/login') {
          window.location.href = '/login'
        }
      }
      return Promise.reject(data)
    }
    return data
  },
  (error) => {
    const { response } = error
    if (response) {
      // HTTP 401 但需要看业务层 code 决定是否清除登录态
      // 签名验证失败(code=20004)不应清除 localStorage，只有真正 session 过期才清除
      const data = response.data
      if (response.status === 401) {
        // 只有真正的 session 过期（如 token 失效）才清除登录态
        // 签名失败不应清除，避免用户被迫重新登录
        if (data && (data.code === 401 || (data.msg && data.msg.includes('未登录')) || (data.msg && data.msg.includes('session')))) {
          removeApiKey()
          removeAdminId()
          if (window.location.pathname !== '/login') {
            window.location.href = '/login'
          }
        }
        return Promise.reject(data || { message: '认证失败' })
      }
      return Promise.reject(response.data || { message: '请求失败' })
    }
    // 网络错误 - 不清除登录态，不跳转登录页
    return Promise.reject({ message: '网络错误，请检查网络连接' })
  }
)

export default request

/**
 * 后端统一响应格式
 * 
 * 后端返回: { code: number, msg: string, data?: T, request_id?: string }
 * - code === 0: 成功
 * - code !== 0: 失败
 */
export interface ApiResponse<T = unknown> {
  /** 错误码 (0 = 成功) */
  code: number
  /** 消息 */
  msg: string
  /** 响应数据 */
  data?: T
  /** 请求 ID */
  request_id?: string
}

/** 判断响应是否成功 */
export function isSuccess<T>(res: ApiResponse<T>): res is ApiResponse<T> & { data: T } {
  return res.code === 0
}

// 分页响应
export interface PaginatedResponse<T> {
  items: T[]
  total: number
  page: number
  page_size: number
  total_pages: number
}

// 分页查询参数
export interface PaginationParams {
  page?: number
  page_size?: number
}
