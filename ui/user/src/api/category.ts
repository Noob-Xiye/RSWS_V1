import request, { type ApiResponse } from './request'

/** 分类信息（用户端简化版） */
export interface Category {
  id: number
  name: string
  description?: string
  sort_order: number
  is_active: boolean
  created_at?: string
  updated_at?: string
}

/** 获取活跃分类列表（公开端点，无需签名，但走统一 request 保证路径正确） */
export async function getCategoryList(): Promise<Category[]> {
  const res = await request.get<{ categories: Category[] }>('/categories')
  if (res.code === 0 && res.data) {
    return res.data.categories
  }
  return []
}
