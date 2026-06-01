import request, { type ApiResponse, type PaginatedResponse } from './request'

// ========== 订单相关 ==========

export interface Order {
  id: number
  user_id: number
  resource_id: number
  amount: number  // 单位：分，前端显示时 ÷100 转元
  status: OrderStatus
  payment_method: string | null
  created_at: string
  updated_at: string
  expired_at: string | null
  // 后端 OrderDetail JOIN 返回的资源标题
  resource_title?: string
}

export type OrderStatus = 'pending' | 'paid' | 'completed' | 'cancelled' | 'refunded'

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
  message?: string
}

// 获取订单列表
export async function listOrders(params?: { page?: number; page_size?: number }): Promise<ApiResponse<PaginatedResponse<Order>>> {
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
export async function checkOrderStatus(id: number): Promise<ApiResponse<{ id: number; status: OrderStatus; confirmations?: number; required_confirmations?: number }>> {
  return request.get(`/order/${id}/status`)
}

// ========== 支付相关 ==========

export interface UsdtAddressResponse {
  network: string
  address: string
  contract?: string
}

// 获取 USDT 收款地址
export async function getUsdtAddress(network: 'tron' | 'ethereum' = 'tron'): Promise<ApiResponse<UsdtAddressResponse>> {
  return request.get(`/payment/usdt/${network}`)
}

// 发起订单支付（获取支付链接）
export interface InitiatePaymentResponse {
  payment_method: string
  // PayPal
  paypal_order_id?: string
  approve_url?: string
  // USDT
  network?: string
  address?: string
  amount?: string
}

export async function initiatePayment(orderId: number): Promise<ApiResponse<InitiatePaymentResponse>> {
  return request.post(`/order/${orderId}/pay`)
}
