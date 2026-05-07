import request, { type ApiResponse, type PaginatedResponse, type PaginationParams } from './request'

export type OrderStatus = 'pending' | 'paid' | 'completed' | 'cancelled' | 'refunded'
export type PaymentMethod = 'paypal' | 'usdt_trc20' | 'usdt_erc20'

export interface Order {
  id: number
  order_no: string
  user_id: number
  user_name: string
  resource_id: number
  resource_title: string
  amount: string
  status: OrderStatus
  payment_method: PaymentMethod | null
  transaction_id: string | null
  created_at: string
  paid_at: string | null
  completed_at: string | null
}

export interface OrderListParams extends PaginationParams {
  order_no?: string
  user_id?: number
  status?: OrderStatus
  payment_method?: PaymentMethod
  start_date?: string
  end_date?: string
}

// 获取订单列表
export async function listOrders(params?: OrderListParams): Promise<ApiResponse<PaginatedResponse<Order>>> {
  return request.get('/order', { params })
}

// 获取订单详情
export async function getOrder(id: number): Promise<ApiResponse<Order>> {
  return request.get(`/order/${id}`)
}

// 取消订单
export async function cancelOrder(id: number): Promise<ApiResponse<void>> {
  return request.post(`/order/${id}/cancel`)
}

// 退款
export async function refundOrder(id: number): Promise<ApiResponse<void>> {
  return request.post(`/order/${id}/refund`)
}