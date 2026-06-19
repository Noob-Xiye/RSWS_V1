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
    // axios 的 config.url 不含 baseURL，需要拼接完整路径以匹配后端 req.uri().path()
    const requestPath = (config.url ? `/api/v1/${config.url.replace(/^\//, '')}` : '')
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
// 只有在用户之前已登录（有 apiKey）的情况下，401 才跳转登录页
// 游客（无 apiKey）访问需要认证的接口时，不跳转，由组件自行处理
function shouldRedirectToLogin(): boolean {
  // 曾经登录过（本地有 apiKey）但现在 401，说明 session 过期
  return !!getApiKey()
}

request.interceptors.response.use(
  (response) => {
    const data = response.data
    // 检查业务层错误码（后端返回 200，但 code !== 0）
    if (data && data.code !== undefined && data.code !== 0) {
      // 仅当 token 过期（code 20001）时才清除 token 并跳转
      // 其他认证错误（如签名失败 code 20004）不清除 token，由组件处理
      if (data.code === 20001) {
        if (shouldRedirectToLogin()) {
          removeApiKey()
          removeUserId()
          if (window.location.pathname !== '/login') {
            window.location.href = '/login'
          }
        }
      }
      return Promise.reject(data)
    }
    return data
  },
  (error) => {
    const { response } = error

    // 401 未授权：仅当曾经登录过且是 token 过期才跳转
    // 注意：后端签名验证失败也返回 HTTP 401，但不应清除 token
    if (response?.status === 401) {
      const data = response?.data
      // 仅当业务 code 是 20001（token 过期）时才清除 token
      if (data && data.code === 20001 && shouldRedirectToLogin()) {
        removeApiKey()
        removeUserId()
        if (window.location.pathname !== '/login') {
          window.location.href = '/login'
        }
      }
      // 其他 401 错误（如签名失败）不清除 token，由组件处理
    }

    // 网络错误（后端挂掉）：不跳转登录页，游客不应因网络问题被踢出
    // 仅打印错误，由组件处理 loading/error 状态
    if (!response) {
      console.warn('[request] 网络错误，后端可能未启动:', error.message)
    }

    // 返回错误信息
    return Promise.reject(response?.data || { message: '网络错误', code: -1 })
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
