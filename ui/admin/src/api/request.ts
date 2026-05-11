import axios from 'axios'
import { getApiKey, getAdminId, removeApiKey, removeAdminId } from '@/utils/storage'
import { generateSignParams } from '@/utils/signature'

// API 基础地址
const BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:5173/api/v1'

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
    const signParams = generateSignParams({
      adminId,
      apiKey,
      path: config.url || '/',
      body: config.data,
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
}, (error) => {
  return Promise.reject(error)
})

// 响应拦截器
request.interceptors.response.use(
  (response) => {
    return response.data
  },
  (error) => {
    const { response } = error
    if (response) {
      // 401 未授权，跳转登录
      if (response.status === 401) {
        removeApiKey()
        removeAdminId()
        window.location.href = '/login'
      }
      return Promise.reject(response.data || { message: '请求失败' })
    }
    return Promise.reject({ message: '网络错误' })
  }
)

export default request

// 通用响应类型
export interface ApiResponse<T = unknown> {
  success: boolean
  data?: T
  message?: string
  code?: number
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