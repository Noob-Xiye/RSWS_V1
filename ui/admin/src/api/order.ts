import request, { type ApiResponse, type PaginatedResponse, type PaginationParams } from './request'

export type OrderStatus = 'pending' | 'paid' | 'completed' | 'cancelled' | 'refunded'
export type PaymentMethod = 'paypal' | 'usdt_trc20' | 'usdt_erc20'

/** 管理员订单详情（对齐后端 AdminOrderDetail） */
export interface AdminOrder {
  id: number
  user_id: number
  user_name: string | null
  user_email: string | null
  resource_id: number
  resource_title: string | null
  amount: number          // 分(cents)，显示时除以100
  status: string
  payment_method: string | null
  created_at: string
  updated_at: string
  expired_at: string | null
}

export interface AdminOrderListParams extends PaginationParams {
  status?: string
  user_id?: number
  payment_method?: string
}

/** 管理员获取全部订单列表 */
export async function adminListOrders(params?: AdminOrderListParams): Promise<ApiResponse<PaginatedResponse<AdminOrder>>> {
  return request.get('/admin/orders', { params })
}

// 退款
export async function refundOrder(id: number): Promise<ApiResponse<void>> {
  return request.post(`/order/${id}/refund`)
}

// 完成订单
export async function completeOrder(id: number): Promise<ApiResponse<void>> {
  return request.post(`/order/${id}/complete`)
}
