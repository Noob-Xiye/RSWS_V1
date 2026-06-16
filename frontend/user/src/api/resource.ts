import request, { type ApiResponse } from './request'

export interface Resource {
  id: number | string
  title: string
  description: string | null
  price: number
  category_id: number | null
  thumbnail_url: string | null
  cover_image?: string | null
  is_active: boolean
  detail_description?: string | null
  specifications?: any
  usage_guide?: string | null
  precautions?: string | null
  display_images?: string[]
  download_count: number
  provider_type?: string
  commission_rate?: number
  created_at: string
  updated_at: string
}

export interface ResourceListResponse {
  items: Resource[]
  total: number
  page: number
  page_size: number
  total_pages: number
}

/** 资源详情（购买后有 file_url） */
export type ResourceDetail = Resource & {
  file_url?: string | null
}

export interface PurchaseCheckResponse {
  purchased: boolean
  resource_id: number | string
}

export interface DownloadInfo {
  file_url: string
  file_name: string
}

/** 获取资源列表（公开端点） */
export async function listResources(params?: {
  page?: number
  page_size?: number
  category_id?: number
  search?: string
}): Promise<ApiResponse<ResourceListResponse>> {
  return request.get('/resource', { params })
}

/** 获取资源详情（公开端点） */
export async function getResource(id: number | string): Promise<ApiResponse<ResourceDetail>> {
  return request.get(`/resource/${id}`)
}

/** 检查用户是否已购买某资源 */
export async function checkPurchase(resourceId: number | string): Promise<ApiResponse<PurchaseCheckResponse>> {
  return request.get(`/resource/${resourceId}/purchase-check`)
}

/** 获取资源下载信息（需要已购买） */
export async function getDownloadInfo(resourceId: number | string): Promise<ApiResponse<DownloadInfo>> {
  return request.get(`/resource/${resourceId}/download`)
}

// ==================== 文件分块上传 ====================

export interface InitUploadResponse {
  upload_id: string
  chunk_size: number
  total_chunks: number
}

export interface CompleteUploadResponse {
  file_url: string
  file_size: number
}

export async function initUpload(filename: string, totalSize: number): Promise<ApiResponse<InitUploadResponse>> {
  return request.post('/upload/init', { filename, total_size: totalSize })
}

export async function uploadChunk(uploadId: string, chunkIndex: number, data: ArrayBuffer): Promise<ApiResponse<{ chunk_index: number; received: number }>> {
  const bytes = new Uint8Array(data)
  let binary = ''
  for (let i = 0; i < bytes.byteLength; i++) binary += String.fromCharCode(bytes[i])
  return request.post('/upload/chunk', {
    upload_id: uploadId,
    chunk_index: chunkIndex,
    data: btoa(binary),
  })
}

export async function completeUpload(uploadId: string, filename: string): Promise<ApiResponse<CompleteUploadResponse>> {
  return request.post('/upload/complete', { upload_id: uploadId, filename })
}

/** 大文件分块上传（封装完整流程） */
export async function uploadLargeFile(file: File, onProgress?: (percent: number) => void): Promise<string> {
  const initRes = await initUpload(file.name, file.size)
  if (initRes.code !== 0 || !initRes.data) throw new Error(initRes.msg || '初始化上传失败')
  const { upload_id, chunk_size, total_chunks } = initRes.data

  for (let i = 0; i < total_chunks; i++) {
    const start = i * chunk_size
    const end = Math.min(start + chunk_size, file.size)
    const chunk = await file.slice(start, end).arrayBuffer()
    const res = await uploadChunk(upload_id, i, chunk)
    if (res.code !== 0) throw new Error(res.msg || `分块 ${i} 上传失败`)
    onProgress?.(Math.round(((i + 1) / total_chunks) * 100))
  }

  const completeRes = await completeUpload(upload_id, file.name)
  if (completeRes.code !== 0 || !completeRes.data) throw new Error(completeRes.msg || '合并文件失败')
  return completeRes.data.file_url
}
