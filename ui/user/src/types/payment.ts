// ========== 支付类型 (User 端) ==========

import type { ApiResponse } from './api'

/** PayPal 支付信息 */
export interface PayPalInfo {
  mode: 'sandbox' | 'live'
  is_active: boolean
}

/** USDT 支付信息 */
export interface UsdtPaymentInfo {
  network: 'trc20' | 'erc20'
  address: string
  qr_code?: string  // Base64
  confirmations: number
}

/** 支付配置响应 */
export interface PaymentConfig {
  paypal?: PayPalInfo
  usdt: {
    trc20?: UsdtPaymentInfo
    erc20?: UsdtPaymentInfo
  }
}

export type PaymentConfigResponse = ApiResponse<PaymentConfig>

/** 支付状态检查响应 */
export interface PaymentStatusData {
  status: 'pending' | 'paid' | 'completed' | 'cancelled' | 'refunded'
  amount: string
  paid_at?: string
  transaction_id?: string
  tx_hash?: string
}

export type PaymentStatusResponse = ApiResponse<PaymentStatusData>