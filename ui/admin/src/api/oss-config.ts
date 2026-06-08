import request from '@/api/request'
import type { AxiosResponse } from 'axios'

/**
 * OSS 存储配置接口
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
 * 获取 OSS 存储配置
 */
export function getOssConfig(): Promise<AxiosResponse<OssStorageConfig>> {
  return request({
    url: '/api/v1/admin/oss-config',
    method: 'get'
  })
}

/**
 * 保存 OSS 存储配置
 */
export function saveOssConfig(
  config: OssStorageConfig
): Promise<AxiosResponse<void>> {
  return request({
    url: '/api/v1/admin/oss-config',
    method: 'post',
    data: config
  })
}

/**
 * 测试 OSS 连接
 */
export function testOssConnection(
  config: OssStorageConfig
): Promise<AxiosResponse<{ success: boolean; message: string }>> {
  return request({
    url: '/api/v1/admin/oss-config/test',
    method: 'post',
    data: config
  })
}
