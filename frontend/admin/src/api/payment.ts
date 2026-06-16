import request, { type ApiResponse } from './request'

/** 获取 USDT 收款地址（公开端点，无需认证） */
export async function getUsdtAddress(network: 'trc20' | 'erc20'): Promise<ApiResponse<{ address: string }>> {
  return request.get(`/payment/usdt/${network}`)
}

// ========== 支付方式管理 ==========
export interface PaymentMethod {
  id: number | string
  method_type: string
  method_name: string
  is_enabled: boolean
  config: Record<string, unknown>
  created_at: string
  updated_at: string
}

export interface CreatePaymentMethodParams {
  method_type: string
  method_name: string
  is_enabled?: boolean
  config?: Record<string, unknown>
}

/** 获取支付方式列表 */
export async function listPaymentMethods(): Promise<ApiResponse<PaymentMethod[]>> {
  return request.get('/admin/payment-methods')
}

/** 创建/更新支付方式（upsert，按 method_type 去重） */
export async function createPaymentMethod(data: CreatePaymentMethodParams): Promise<ApiResponse<{ success: boolean }>> {
  return request.post('/admin/payment-methods', data)
}

/** 删除/禁用支付方式（软删除） */
export async function deletePaymentMethod(id: number | string): Promise<ApiResponse<{ success: boolean }>> {
  return request.delete(`/admin/payment-methods/${id}`)
}
