import request, { type ApiResponse } from './request'

// ========== 订单相关 ==========

export interface Order {
  id: number
  user_id: number
  resource_id: number
  amount: number  // 后端是 i64（分），前端显示时需转换
  payment_method?: 'paypal' | 'usdt_trc20' | 'usdt_erc20'
  status: OrderStatus
  expired_at?: string
  created_at: string
  updated_at: string
  // 后端 JOIN 返回的资源标题
  resource_title?: string
}

export type OrderStatus = 'pending' | 'paid' | 'completed' | 'cancelled' | 'failed' | 'refunded'

export interface OrderListResponse {
  items: Order[]
  total: number
  page: number
  limit: number
  total_pages: number
}

export interface CreateOrderRequest {
  resource_id: number
  payment_method: 'paypal' | 'usdt_trc20' | 'usdt_erc20'
}

export interface CreateOrderResponse {
  id: number
  resource_id: number
  amount: number
  payment_method: string
  status: OrderStatus
  expired_at?: string
  // PayPal 支付相关
  paypal_order_id?: string
  approve_url?: string
  message?: string  // PayPal 不可用时的提示
}

export interface OrderStatusResponse {
  id: number
  status: OrderStatus
  confirmations: number
  required_confirmations: number
}

export interface DownloadInfo {
  file_url: string
  file_name: string
}

// 获取订单列表
export async function listOrders(params?: { page?: number; limit?: number }): Promise<ApiResponse<OrderListResponse>> {
  return request.get('/order', { params })
}

// 获取单个订单详情
export async function getOrder(id: number): Promise<ApiResponse<Order>> {
  return request.get(`/order/${id}`)
}

// 创建订单（发起支付）
export async function createOrder(data: CreateOrderRequest): Promise<ApiResponse<CreateOrderResponse>> {
  return request.post('/order', data)
}

// 取消订单
export async function cancelOrder(id: number): Promise<ApiResponse<{ id: number; status: string }>> {
  return request.post(`/order/${id}/cancel`)
}

// 轮询订单状态（USDT 支付用）
export async function checkOrderStatus(id: number): Promise<ApiResponse<OrderStatusResponse>> {
  return request.get(`/order/${id}/status`)
}

// 获取已购买资源的下载信息
export async function getDownloadInfo(resourceId: number): Promise<ApiResponse<DownloadInfo>> {
  return request.get(`/resource/${resourceId}/download`)
}

// ========== 支付相关 ==========

export interface UsdtAddressResponse {
  network: string
  address: string
  contract: string
}

// 获取 USDT 收款地址
export async function getUsdtAddress(network: 'tron' | 'ethereum' = 'tron'): Promise<ApiResponse<UsdtAddressResponse>> {
  return request.get(`/payment/usdt/${network}`)
}