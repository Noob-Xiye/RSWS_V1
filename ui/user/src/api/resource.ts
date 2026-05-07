import request, { type ApiResponse } from './request'

export interface Resource {
  id: number
  title: string
  description: string
  price: string
  category: string
  cover_image: string | null
  download_count: number
  creator_name: string
  created_at: string
}

export async function listResources(params?: { page?: number; page_size?: number; category?: string; keyword?: string }): Promise<ApiResponse<{ items: Resource[]; total: number }>> {
  return request.get('/resource', { params })
}

export async function getResource(id: number): Promise<ApiResponse<Resource & { content?: string; file_url?: string }>> {
  return request.get(`/resource/${id}`)
}