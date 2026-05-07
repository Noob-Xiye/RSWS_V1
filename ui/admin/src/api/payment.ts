import request, { type ApiResponse } from './request'

export interface UsdtAddress {
  address: string
  network: 'trc20' | 'erc20'
  is_active: boolean
  label?: string
}

export interface PaymentConfig {
  paypal_client_id: string
  paypal_mode: 'sandbox' | 'live'
  usdt_addresses: UsdtAddress[]
}

// 获取 USDT 充值地址
export async function getUsdtAddress(network: 'trc20' | 'erc20'): Promise<ApiResponse<{ address: string }>> {
  return request.get(`/payment/usdt/${network}`)
}

// 获取支付配置（假设的 API，可能需要后端添加）
export async function getPaymentConfig(): Promise<ApiResponse<PaymentConfig>> {
  return request.get('/admin/payment-config')
}

// 更新 USDT 地址
export async function updateUsdtAddress(network: 'trc20' | 'erc20', address: string): Promise<ApiResponse<void>> {
  return request.put('/admin/payment-config/usdt', { network, address })
}