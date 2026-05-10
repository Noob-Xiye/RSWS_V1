// ========== 订单类型 ==========

import type { ApiResponse, PaginatedData } from './api'

/** 订单状态 */
export type OrderStatus = 'pending' | 'paid' | 'completed' | 'cancelled' | 'refunded'

/** 支付方式 */
export type PaymentMethod = 'paypal' | 'usdt_trc20' | 'usdt_erc20'

/** 订单信息 */
export interface Order {
  id: number
  order_no: string
  user_id: number
  user_name: string
  user_email?: string
  resource_id: number
  resource_title: string
  resource_cover?: string
  amount: string  // 单位: 分
  status: OrderStatus
  payment_method: PaymentMethod | null
  transaction_id?: string  // PayPal transaction ID
  tx_hash?: string    // USDT transaction hash
  note?: string
  created_at: string
  paid_at?: string
  completed_at?: string
  cancelled_at?: string
  refunded_at?: string
}

/** 订单列表数据 */
export interface OrderListData extends PaginatedData<Order> {}
export type OrderListResponse = ApiResponse<OrderListData>

/** 订单查询参数 */
export interface OrderListParams {
  page?: number
  page_size?: number
  order_no?: string
  user_id?: number
  status?: OrderStatus
  payment_method?: PaymentMethod
  start_date?: string
  end_date?: string
}

/** 订单操作响应 */
export interface OrderActionResponse {
  id: number
  status: OrderStatus
  message?: string
}