import request from '@/api/request'
import type { AxiosResponse } from 'axios'

/**
 * OSS 瀛樺偍閰嶇疆鎺ュ彛
 */
export interface OssStorageConfig {
  provider: string // 'local' | 's3' | 'minio' | 'aliyun-oss' | 'tencent-cos'
  endpoint: string
  bucket: string
  access_key: string
  secret_key: string
  region: string
  prefix: string
  custom_domain: string
  is_active: boolean
}

/**
 * 鑾峰彇 OSS 瀛樺偍閰嶇疆
 */
export function getOssConfig(): Promise<AxiosResponse<OssStorageConfig>> {
  return request({
    url: '/admin/oss-configs',
    method: 'get'
  })
}

/**
 * 淇濆瓨 OSS 瀛樺偍閰嶇疆
 */
export function saveOssConfig(
  config: OssStorageConfig
): Promise<AxiosResponse<void>> {
  return request({
    url: '/admin/oss-configs',
    method: 'post',
    data: config
  })
}

/**
 * 娴嬭瘯 OSS 杩炴帴
 */
export function testOssConnection(
  config: OssStorageConfig
): Promise<AxiosResponse<{ success: boolean; message: string }>> {
  return request({
    url: '/admin/oss-config/test',
    method: 'post',
    data: config
  })
}
