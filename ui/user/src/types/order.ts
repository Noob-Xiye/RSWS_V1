// ========== 订单类型 (User 端) ==========

import type { ApiResponse, PaginatedData } from './api'

/** 订单状态 */
export type OrderStatus = 'pending' | 'paid' | 'completed' | 'cancelled' | 'refunded'

/** 支付方式 */
export type PaymentMethod = 'paypal' | 'usdt_trc20' | 'usdt_erc20'

/** 订单信息 */
export interface Order {
  id: number
  order_no: string
  resource_id: number
  resource_title: string
  resource_cover?: string
  amount: string  // 单位: 分
  status: OrderStatus
  payment_method: PaymentMethod | null
  transaction_id?: string
  tx_hash?: string
  created_at: string
  paid_at?: string
  completed_at?: string
  cancelled_at?: string
}

/** 订单列表数据 */
export interface OrderListData extends PaginatedData<Order> {}
export type OrderListResponse = ApiResponse<OrderListData>

/** 订单查询参数 */
export interface OrderListParams {
  page?: number
  page_size?: number
  status?: OrderStatus
}

/** 创建订单请求 */
export interface CreateOrderRequest {
  resource_id: number
  payment_method: PaymentMethod
}

/** 订单响应数据 */
export interface OrderData {
  order_id: number
  order_no: string
  amount: string
  payment_url?: string  // PayPal 支付链接
  wallet_address?: string  // USDT 地址
  qr_code?: string  // USDT 二维码
  expires_at: string
}

export type CreateOrderResponse = ApiResponse<OrderData>