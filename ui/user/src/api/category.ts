import axios from 'axios'

export interface Category {
  id: number
  name: string
  description?: string
  sort_order: number
  is_active: boolean
  created_at?: string
  updated_at?: string
}

export const getCategoryList = async (): Promise<Category[]> => {
  const response = await axios.get('/categories')
  return response.data.data || response.data
}
