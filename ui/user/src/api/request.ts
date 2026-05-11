import axios from 'axios'
import { getApiKey, getApiSecret, removeApiKey, removeApiSecret } from '@/utils/storage'
import { generateSignParams } from '@/utils/signature'

const BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:5173/api/v1'

const request = axios.create({
  baseURL: BASE_URL,
  timeout: 30000,
  headers: { 'Content-Type': 'application/json' }
})

// 请求拦截器 - 添加 API Key 签名 (Cregis 方案)
request.interceptors.request.use(async (config) => {
  const apiKey = getApiKey()
  const apiSecret = getApiSecret()
  
  if (apiKey && apiSecret) {
    // 生成签名参数 (Cregis 方案)
    const signParams = generateSignParams({
      apiKey,
      apiSecret,
      path: config.url || '/',
      body: config.data,
    })
    
    // 将签名参数添加到查询参数
    config.params = {
      ...config.params,
      ...signParams,
    }
  }
  
  return config
}, (error) => Promise.reject(error))

// 响应拦截器 - 统一错误处理
request.interceptors.response.use(
  (response) => response.data,
  (error) => {
    const { response } = error
    
    // 401 未授权，清除登录信息并跳转登录页
    if (response?.status === 401) {
      removeApiKey()
      removeApiSecret()
      window.location.href = '/login'
    }
    
    // 返回错误信息
    return Promise.reject(response?.data || { message: '网络错误' })
  }
)

export default request

export interface ApiResponse<T = unknown> {
  success: boolean
  data?: T
  message?: string
}

export interface PaginatedResponse<T> {
  items: T[]
  total: number
  page: number
  page_size: number
}
