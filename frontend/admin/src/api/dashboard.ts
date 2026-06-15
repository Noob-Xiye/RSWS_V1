import request, { type ApiResponse, type PaginatedResponse, type PaginationParams } from './request'

export interface DailyOrderCount {
  date: string   // YYYY-MM-DD
  count: number
}

export interface DashboardStats {
  // 用户统计
  total_users: number
  new_users_30d: number
  // 订单统计
  total_orders: number
  completed_orders: number
  // 收入统计（单位：分，前端除以100转元）
  total_revenue: number
  revenue_30d: number
  // 资源统计
  total_resources: number
  active_resources: number
  new_resources_30d: number
  // 过去30天订单趋势
  orders_trend: DailyOrderCount[]
}

export interface RevenueChart {
  dates: string[]
  revenues: number[]
}

// 获取仪表盘统计数据
export async function getDashboardStats(): Promise<ApiResponse<DashboardStats>> {
  return request.get('/admin/dashboard/stats')
}

// 获取收入图表数据
export async function getRevenueChart(days?: number): Promise<ApiResponse<RevenueChart>> {
  return request.get('/admin/dashboard/revenue-chart', { params: { days } })
}