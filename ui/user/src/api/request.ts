import axios from 'axios'
import { getApiKey } from '@/utils/storage'

const BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:5173/api/v1'

const request = axios.create({
  baseURL: BASE_URL,
  timeout: 30000,
  headers: { 'Content-Type': 'application/json' }
})

request.interceptors.request.use((config) => {
  const apiKey = getApiKey()
  if (apiKey) config.headers['X-API-Key'] = apiKey
  return config
}, (error) => Promise.reject(error))

request.interceptors.response.use(
  (response) => response.data,
  (error) => {
    const { response } = error
    if (response?.status === 401) {
      localStorage.removeItem('rsws_user_api_key')
      window.location.href = '/login'
    }
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