import request, { type ApiResponse, type PaginatedResponse, type PaginationParams } from './request'

export interface DashboardStats {
  total_users: number
  total_resources: number
  total_orders: number
  total_revenue: string
  pending_orders: number
  today_orders: number
  today_revenue: string
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