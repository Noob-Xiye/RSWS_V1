import axios from 'axios'
import { getApiKey } from '@/utils/storage'

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

// 请求拦截器
request.interceptors.request.use(
  (config) => {
    const apiKey = getApiKey()
    if (apiKey) {
      config.headers['X-API-Key'] = apiKey
    }
    return config
  },
  (error) => {
    return Promise.reject(error)
  }
)

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
        localStorage.removeItem('rsws_admin_api_key')
        localStorage.removeItem('rsws_admin_token')
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