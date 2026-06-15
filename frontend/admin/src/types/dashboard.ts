// ========== Dashboard 数据类型 ==========

import type { ApiResponse } from './api'

/** Dashboard 统计数据 */
export interface DashboardStats {
  // 用户统计
  total_users: number
  new_users_30d: number

  // 资源统计
  total_resources: number
  active_resources: number
  new_resources_30d: number

  // 订单统计
  total_orders: number
  completed_orders: number
  pending_orders: number

  // 收入统计 (单位: 分)
  total_revenue: number
  revenue_30d: number

  // 日订单趋势
  orders_trend: DailyOrderCount[]
}

/** 日订单数量 */
export interface DailyOrderCount {
  date: string      // YYYY-MM-DD
  count: number
  revenue: number // 单位: 分
}

export type DashboardStatsResponse = ApiResponse<DashboardStats>

/** 收入趋势数据 */
export interface RevenueTrend {
  date: string
  amount: number  // 单位: 分
  count: number
}

/** 热门资源 */
export interface TopResource {
  id: number
  title: string
  download_count: number
  revenue: number  // 单位: 分
}

/** 活跃用户 */
export interface ActiveUser {
  id: number
  username: string
  email: string
  order_count: number
  total_spent: number  // 单位: 分
}