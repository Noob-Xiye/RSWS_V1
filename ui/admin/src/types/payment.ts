// ========== 支付配置类型 ==========

import type { ApiResponse } from './api'

/** PayPal 配置 */
export interface PayPalConfig {
  id: number
  mode: 'sandbox' | 'live'
  client_id: string
  client_secret?: string  // 只在创建/更新时返回
  webhook_id?: string
  is_active: boolean
  created_at: string
  updated_at: string
}

/** 区块链配置 */
export interface BlockchainConfig {
  id: number
  network: 'trc20' | 'erc20'
  chain: 'tron' | 'ethereum'
  name: string
  contract_address?: string
  wallet_address?: string
  confirmations: number
  is_active: boolean
  created_at: string
  updated_at: string
}

/** USDT 钱包 */
export interface UsdtWallet {
  network: 'trc20' | 'erc20'
  address: string
  qr_code?: string  // Base64 image
  created_at: string
  updated_at: string
}

export type PayPalConfigResponse = ApiResponse<PayPalConfig>
export type BlockchainConfigResponse = ApiResponse<BlockchainConfig>
export type UsdtWalletListResponse = ApiResponse<UsdtWallet[]>

/** 更新 PayPal 配置参数 */
export interface UpdatePayPalConfigRequest {
  mode?: 'sandbox' | 'live'
  client_id?: string
  client_secret?: string
  webhook_id?: string
  is_active?: boolean
}

/** 更新区块链配置参数 */
export interface UpdateBlockchainConfigRequest {
  wallet_address?: string
  contract_address?: string
  confirmations?: number
  is_active?: boolean
}