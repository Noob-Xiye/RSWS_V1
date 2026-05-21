import axios from 'axios'
import { getApiKey, getUserId, removeApiKey, removeUserId } from '@/utils/storage'
import { generateSignParams } from '@/utils/signature'

const BASE_URL = import.meta.env.VITE_API_URL || '/api/v1'

const request = axios.create({
  baseURL: BASE_URL,
  timeout: 30000,
  headers: { 'Content-Type': 'application/json' }
})

// 请求拦截器 - 添加 API Key 签名 (Cregis 方案)
request.interceptors.request.use(async (config) => {
  const apiKey = getApiKey()
  const userId = getUserId()
  
  if (apiKey && userId) {
    // 生成签名参数 (Cregis 方案)
    // 注意：只传 user_id，不传 api_key
    // 注意：不传 body，后端只读 query params 签名
    // 提取请求路径（不含 query params）用于签名防篡改
    const requestPath = config.url || ''
    const signParams = generateSignParams({
      userId,
      apiKey,
      path: requestPath,
    })
    
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
}, (error) => Promise.reject(error))

// 响应拦截器 - 统一错误处理
request.interceptors.response.use(
  (response) => {
    const data = response.data
    // 检查业务层错误码（后端返回 200，但 code !== 0）
    if (data && data.code !== undefined && data.code !== 0) {
      // 401 = 未登录 / 认证失败 → 清除本地状态并跳转登录页
      if (data.code === 401 || (data.msg && data.msg.includes('未登录'))) {
        removeApiKey()
        removeUserId()
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
    
    // 401 未授权，清除登录信息并跳转登录页
    if (response?.status === 401) {
      removeApiKey()
      removeUserId()
      if (window.location.pathname !== '/login') {
        window.location.href = '/login'
      }
    }
    
    // 网络错误（后端挂掉）也触发登出
    if (!response) {
      removeApiKey()
      removeUserId()
      if (window.location.pathname !== '/login') {
        window.location.href = '/login'
      }
    }
    
    // 返回错误信息
    return Promise.reject(response?.data || { message: '网络错误' })
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

export interface PaginatedResponse<T> {
  items: T[]
  total: number
  page: number
  page_size: number
}
