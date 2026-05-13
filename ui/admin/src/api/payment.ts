import request, { type ApiResponse } from './request'

/** 获取 USDT 收款地址（公开端点，无需认证） */
export async function getUsdtAddress(network: 'trc20' | 'erc20'): Promise<ApiResponse<{ address: string }>> {
  return request.get(`/payment/usdt/${network}`)
}
