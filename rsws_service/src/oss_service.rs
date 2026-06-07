//! OSS 存储服务
//!
//! 提供统一的对象存储接口，支持：
//! - 本地文件系统
//! - AWS S3
//! - MinIO (S3 兼容)
//! - 阿里云 OSS (S3 兼容)
//! - 腾讯云 COS (S3 兼容)

use async_trait::async_trait;
use bytes::Bytes;
use chrono::{DateTime, Utc};
use rsws_common::error::RswsError;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tracing::info;

// ==================== 错误类型 ====================

/// 存储错误
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("S3 error: {0}")]
    S3(String),

    #[error("Config error: {0}")]
    Config(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Already exists: {0}")]
    AlreadyExists(String),
}

impl From<StorageError> for RswsError {
    fn from(err: StorageError) -> Self {
        match err {
            StorageError::Io(e) => RswsError::internal(format!("Storage IO error: {}", e)),
            StorageError::S3(e) => RswsError::internal(format!("Storage S3 error: {}", e)),
            StorageError::Config(e) => RswsError::business_with_message(
                rsws_common::error_code::ErrorCode::INVALID_PARAMETER,
                e,
            ),
            StorageError::NotFound(e) => RswsError::business_with_message(
                rsws_common::error_code::ErrorCode::RESOURCE_NOT_FOUND,
                e,
            ),
            StorageError::AlreadyExists(e) => RswsError::business_with_message(
                rsws_common::error_code::ErrorCode::INVALID_PARAMETER,
                e,
            ),
        }
    }
}

// ==================== 存储抽象 ====================

/// 文件元信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub key: String,
    pub size: u64,
    pub content_type: Option<String>,
    pub last_modified: Option<DateTime<Utc>>,
    pub etag: Option<String>,
}

/// 上传结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadResult {
    pub key: String,
    pub url: String,
    pub size: u64,
    pub etag: Option<String>,
}

/// 存储后端抽象
#[async_trait]
pub trait StorageBackend: Send + Sync + std::fmt::Debug {
    /// 上传文件
    async fn upload(
        &self,
        key: &str,
        data: Bytes,
        content_type: Option<&str>,
    ) -> Result<UploadResult, StorageError>;

    /// 下载文件
    async fn download(&self, key: &str) -> Result<Bytes, StorageError>;

    /// 删除文件
    async fn delete(&self, key: &str) -> Result<(), StorageError>;

    /// 检查文件是否存在
    async fn exists(&self, key: &str) -> Result<bool, StorageError>;

    /// 获取文件元信息
    async fn metadata(&self, key: &str) -> Result<FileMetadata, StorageError>;

    /// 生成预签名 URL（用于私有文件下载）
    async fn presign_url(&self, key: &str, expires_in_secs: u64) -> Result<String, StorageError>;

    /// 列出文件（前缀搜索）
    async fn list(
        &self,
        prefix: Option<&str>,
        max_keys: Option<i32>,
    ) -> Result<Vec<FileMetadata>, StorageError>;

    /// 获取公开访问 URL
    fn public_url(&self, key: &str) -> String;
}

// ==================== 本地存储实现 ====================

/// 本地文件系统存储
#[derive(Debug, Clone)]
pub struct LocalStorage {
    pub base_dir: PathBuf,
    pub base_url: String,
}

impl LocalStorage {
    /// 创建本地存储实例
    pub fn new(base_dir: PathBuf, base_url: String) -> Result<Self, StorageError> {
        // 确保目录存在
        if !base_dir.exists() {
            std::fs::create_dir_all(&base_dir).map_err(StorageError::Io)?;
        }

        Ok(Self { base_dir, base_url })
    }

    /// 构建本地文件路径
    fn file_path(&self, key: &str) -> PathBuf {
        self.base_dir.join(key)
    }
}

#[async_trait]
impl StorageBackend for LocalStorage {
    async fn upload(
        &self,
        key: &str,
        data: Bytes,
        _content_type: Option<&str>,
    ) -> Result<UploadResult, StorageError> {
        let path = self.file_path(key);

        // 确保父目录存在
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent).map_err(StorageError::Io)?;
            }
        }

        // 写入文件
        tokio::fs::write(&path, &data)
            .await
            .map_err(StorageError::Io)?;

        let url = self.public_url(key);
        let size = data.len() as u64;

        info!("Local file uploaded: {} -> {}", key, path.display());

        Ok(UploadResult {
            key: key.to_string(),
            url,
            size,
            etag: None,
        })
    }

    async fn download(&self, key: &str) -> Result<Bytes, StorageError> {
        let path = self.file_path(key);
        let data = tokio::fs::read(&path).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                StorageError::NotFound(format!("File not found: {}", key))
            } else {
                StorageError::Io(e)
            }
        })?;

        Ok(Bytes::from(data))
    }

    async fn delete(&self, key: &str) -> Result<(), StorageError> {
        let path = self.file_path(key);

        if !path.exists() {
            return Err(StorageError::NotFound(format!("File not found: {}", key)));
        }

        tokio::fs::remove_file(&path)
            .await
            .map_err(StorageError::Io)?;

        info!("Local file deleted: {}", path.display());

        Ok(())
    }

    async fn exists(&self, key: &str) -> Result<bool, StorageError> {
        let path = self.file_path(key);
        Ok(path.exists())
    }

    async fn metadata(&self, key: &str) -> Result<FileMetadata, StorageError> {
        let path = self.file_path(key);
        let metadata = tokio::fs::metadata(&path).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                StorageError::NotFound(format!("File not found: {}", key))
            } else {
                StorageError::Io(e)
            }
        })?;

        let last_modified = metadata.modified().ok().and_then(|t| {
            let duration = t.duration_since(std::time::UNIX_EPOCH).ok()?;
            DateTime::from_timestamp(duration.as_secs() as i64, duration.subsec_nanos())
        });

        Ok(FileMetadata {
            key: key.to_string(),
            size: metadata.len(),
            content_type: None,
            last_modified,
            etag: None,
        })
    }

    async fn presign_url(&self, key: &str, _expires_in_secs: u64) -> Result<String, StorageError> {
        // 本地存储不支持预签名 URL，直接返回公开 URL
        Ok(self.public_url(key))
    }

    async fn list(
        &self,
        prefix: Option<&str>,
        max_keys: Option<i32>,
    ) -> Result<Vec<FileMetadata>, StorageError> {
        let dir = match prefix {
            Some(p) => self.base_dir.join(p),
            None => self.base_dir.clone(),
        };

        let mut result = Vec::new();
        let mut entries = tokio::fs::read_dir(&dir).await.map_err(StorageError::Io)?;

        let mut count = 0;
        let max = max_keys.unwrap_or(i32::MAX);

        while let Some(entry) = entries.next_entry().await.map_err(StorageError::Io)? {
            if count >= max {
                break;
            }

            let metadata = entry.metadata().await.map_err(StorageError::Io)?;

            if metadata.is_file() {
                let path = entry.path();
                let key = path
                    .strip_prefix(&self.base_dir)
                    .unwrap()
                    .to_string_lossy()
                    .replace("\\", "/");

                result.push(FileMetadata {
                    key,
                    size: metadata.len(),
                    content_type: None,
                    last_modified: metadata.modified().ok().and_then(|t| {
                        DateTime::from_timestamp(
                            t.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64,
                            0,
                        )
                    }),
                    etag: None,
                });

                count += 1;
            }
        }

        Ok(result)
    }

    fn public_url(&self, key: &str) -> String {
        format!("{}/{}", self.base_url.trim_end_matches('/'), key)
    }
}

// ==================== S3 兼容存储实现 ====================

/// S3 兼容存储（支持 AWS S3、MinIO、阿里云 OSS、腾讯云 COS）
#[derive(Debug, Clone)]
pub struct S3Storage {
    pub client: aws_sdk_s3::Client,
    pub bucket: String,
    pub prefix: String,
    pub endpoint: Option<String>,
    pub region: String,
    pub custom_domain: Option<String>,
}

impl S3Storage {
    /// 创建 S3 存储实例
    pub async fn new(
        config: &crate::config_service::OssStorageConfig,
    ) -> Result<Self, StorageError> {
        let region = aws_config::Region::new(config.region.clone());

        // 构建 S3 配置
        let mut s3_config_builder = aws_sdk_s3::config::Builder::new()
            .region(region)
            .credentials_provider(aws_sdk_s3::config::Credentials::new(
                &config.access_key,
                &config.secret_key,
                None,
                None,
                "rsws",
            ));

        // 如果有自定义 endpoint（MinIO、OSS、COS）
        if !config.endpoint.is_empty() {
            s3_config_builder = s3_config_builder.endpoint_url(
                url::Url::parse(&config.endpoint)
                    .map_err(|e| StorageError::Config(format!("Invalid endpoint URL: {}", e)))?,
            );
        }

        let s3_config = s3_config_builder.build();
        let s3_client = aws_sdk_s3::Client::from_conf(s3_config);

        Ok(Self {
            client: s3_client,
            bucket: config.bucket.clone(),
            prefix: config.prefix.clone(),
            endpoint: if config.endpoint.is_empty() {
                None
            } else {
                Some(config.endpoint.clone())
            },
            region: config.region.clone(),
            custom_domain: config.custom_domain.clone(),
        })
    }

    /// 构建完整 key（带前缀）
    fn full_key(&self, key: &str) -> String {
        if self.prefix.is_empty() {
            key.to_string()
        } else {
            format!("{}/{}", self.prefix.trim_end_matches('/'), key)
        }
    }
}

#[async_trait]
impl StorageBackend for S3Storage {
    async fn upload(
        &self,
        key: &str,
        data: Bytes,
        content_type: Option<&str>,
    ) -> Result<UploadResult, StorageError> {
        let full_key = self.full_key(key);
        let size = data.len() as u64;

        let mut request = self
            .client
            .put_object()
            .bucket(&self.bucket)
            .key(&full_key)
            .body(data.into());

        if let Some(ct) = content_type {
            request = request.content_type(ct);
        }

        let response = request
            .send()
            .await
            .map_err(|e| StorageError::S3(format!("Failed to upload: {}", e)))?;

        let etag = response.e_tag().map(|s| s.trim_matches('"').to_string());

        let url = self.public_url(key);

        info!("S3 file uploaded: {} -> {}", key, full_key);

        Ok(UploadResult {
            key: key.to_string(),
            url,
            size,
            etag,
        })
    }

    async fn download(&self, key: &str) -> Result<Bytes, StorageError> {
        let full_key = self.full_key(key);

        let response = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(&full_key)
            .send()
            .await
            .map_err(|e| {
                if e.to_string().contains("NoSuchKey") {
                    StorageError::NotFound(format!("File not found: {}", key))
                } else {
                    StorageError::S3(format!("Failed to download: {}", e))
                }
            })?;

        let data = response
            .body
            .collect()
            .await
            .map_err(|e| StorageError::S3(format!("Failed to read body: {}", e)))?;

        Ok(data.into_bytes())
    }

    async fn delete(&self, key: &str) -> Result<(), StorageError> {
        let full_key = self.full_key(key);

        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(&full_key)
            .send()
            .await
            .map_err(|e| StorageError::S3(format!("Failed to delete: {}", e)))?;

        info!("S3 file deleted: {}", full_key);

        Ok(())
    }

    async fn exists(&self, key: &str) -> Result<bool, StorageError> {
        let full_key = self.full_key(key);

        let response = self
            .client
            .head_object()
            .bucket(&self.bucket)
            .key(&full_key)
            .send()
            .await;

        match response {
            Ok(_) => Ok(true),
            Err(e) => {
                if e.to_string().contains("NotFound") {
                    Ok(false)
                } else {
                    Err(StorageError::S3(format!(
                        "Failed to check existence: {}",
                        e
                    )))
                }
            }
        }
    }

    async fn metadata(&self, key: &str) -> Result<FileMetadata, StorageError> {
        let full_key = self.full_key(key);

        let response = self
            .client
            .head_object()
            .bucket(&self.bucket)
            .key(&full_key)
            .send()
            .await
            .map_err(|e| {
                if e.to_string().contains("NotFound") {
                    StorageError::NotFound(format!("File not found: {}", key))
                } else {
                    StorageError::S3(format!("Failed to get metadata: {}", e))
                }
            })?;

        let last_modified = None;

        Ok(FileMetadata {
            key: key.to_string(),
            size: response.content_length().unwrap_or(0) as u64,
            content_type: response.content_type().map(|s| s.to_string()),
            last_modified,
            etag: response.e_tag().map(|s| s.trim_matches('"').to_string()),
        })
    }

    async fn presign_url(&self, key: &str, expires_in_secs: u64) -> Result<String, StorageError> {
        let full_key = self.full_key(key);

        let presigned = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(&full_key)
            .presigned(
                aws_sdk_s3::presigning::PresigningConfig::expires_in(
                    std::time::Duration::from_secs(expires_in_secs),
                )
                .map_err(|e| StorageError::S3(format!("Failed to create presign config: {}", e)))?,
            )
            .await
            .map_err(|e| StorageError::S3(format!("Failed to presign URL: {}", e)))?;

        Ok(presigned.uri().to_string())
    }

    async fn list(
        &self,
        prefix: Option<&str>,
        max_keys: Option<i32>,
    ) -> Result<Vec<FileMetadata>, StorageError> {
        let mut list_request =
            self.client
                .list_objects_v2()
                .bucket(&self.bucket)
                .prefix(if let Some(p) = prefix {
                    format!("{}/{}", self.prefix.trim_end_matches('/'), p)
                } else {
                    self.prefix.clone()
                });

        if let Some(max) = max_keys {
            list_request = list_request.max_keys(max);
        }

        let response = list_request
            .send()
            .await
            .map_err(|e| StorageError::S3(format!("Failed to list objects: {}", e)))?;

        let mut result = Vec::new();

        for obj in response.contents() {
            let key = obj
                .key()
                .unwrap_or("")
                .strip_prefix(&self.prefix)
                .unwrap_or(obj.key().unwrap_or(""))
                .to_string();

            let last_modified = None;

            result.push(FileMetadata {
                key,
                size: obj.size().unwrap_or(0) as u64,
                content_type: None,
                last_modified,
                etag: obj.e_tag().map(|s| s.trim_matches('"').to_string()),
            });
        }

        Ok(result)
    }

    fn public_url(&self, key: &str) -> String {
        let full_key = self.full_key(key);

        // 如果有自定义域名，使用 CDN
        if let Some(ref domain) = self.custom_domain {
            return format!("{}/{}", domain.trim_end_matches('/'), full_key);
        }

        // 否则使用默认 endpoint
        if let Some(ref endpoint) = self.endpoint {
            format!("{}/{}", endpoint.trim_end_matches('/'), full_key)
        } else {
            // AWS S3 标准 URL
            format!(
                "https://{}.s3.{}.amazonaws.com/{}",
                self.bucket, self.region, full_key
            )
        }
    }
}

// ==================== 存储服务 ====================

/// 存储服务（统一入口）
#[derive(Debug)]
pub struct StorageService {
    backend: Arc<dyn StorageBackend>,
}

impl Clone for StorageService {
    fn clone(&self) -> Self {
        Self {
            backend: self.backend.clone(),
        }
    }
}

impl StorageService {
    /// 创建存储服务实例
    pub async fn new(
        config: &crate::config_service::OssStorageConfig,
    ) -> Result<Self, StorageError> {
        let backend: Arc<dyn StorageBackend> = match config.provider.as_str() {
            "local" => {
                // 本地存储：endpoint 字段存储本地路径
                let base_dir = PathBuf::from(&config.endpoint);
                // 从配置读取 base_url，或使用默认值
                let base_url = std::env::var("RSWS_UPLOAD_BASE_URL")
                    .unwrap_or_else(|_| "http://localhost:5170/uploads".to_string());
                Arc::new(LocalStorage::new(base_dir, base_url)?)
            }
            _ => {
                // S3 兼容存储（s3, minio, aliyun-oss, tencent-cos）
                Arc::new(S3Storage::new(config).await?)
            }
        };

        Ok(Self { backend })
    }

    /// 上传文件
    pub async fn upload(
        &self,
        key: &str,
        data: Bytes,
        content_type: Option<&str>,
    ) -> Result<UploadResult, RswsError> {
        self.backend
            .upload(key, data, content_type)
            .await
            .map_err(Into::into)
    }

    /// 下载文件
    pub async fn download(&self, key: &str) -> Result<Bytes, RswsError> {
        self.backend.download(key).await.map_err(Into::into)
    }

    /// 删除文件
    pub async fn delete(&self, key: &str) -> Result<(), RswsError> {
        self.backend.delete(key).await.map_err(Into::into)
    }

    /// 检查文件是否存在
    pub async fn exists(&self, key: &str) -> Result<bool, RswsError> {
        self.backend.exists(key).await.map_err(Into::into)
    }

    /// 获取文件元信息
    pub async fn metadata(&self, key: &str) -> Result<FileMetadata, RswsError> {
        self.backend.metadata(key).await.map_err(Into::into)
    }

    /// 生成预签名 URL
    pub async fn presign_url(&self, key: &str, expires_in_secs: u64) -> Result<String, RswsError> {
        self.backend
            .presign_url(key, expires_in_secs)
            .await
            .map_err(Into::into)
    }

    /// 列出文件
    pub async fn list(
        &self,
        prefix: Option<&str>,
        max_keys: Option<i32>,
    ) -> Result<Vec<FileMetadata>, RswsError> {
        self.backend
            .list(prefix, max_keys)
            .await
            .map_err(Into::into)
    }

    /// 获取公开访问 URL
    pub fn public_url(&self, key: &str) -> String {
        self.backend.public_url(key)
    }
}

// ==================== 单元测试 ====================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_local_storage() {
        let temp_dir = std::env::temp_dir().join("rsws_test_storage");
        let storage = LocalStorage::new(
            temp_dir.clone(),
            "http://localhost:5170/uploads".to_string(),
        )
        .unwrap();

        // 上传
        let data = Bytes::from("Hello, World!");
        let result = storage
            .upload("test.txt", data.clone(), Some("text/plain"))
            .await
            .unwrap();

        assert_eq!(result.key, "test.txt");
        assert!(result.url.contains("test.txt"));

        // 检查存在
        let exists = storage.exists("test.txt").await.unwrap();
        assert!(exists);

        // 下载
        let downloaded = storage.download("test.txt").await.unwrap();
        assert_eq!(downloaded, data);

        // 删除
        storage.delete("test.txt").await.unwrap();

        // 清理
        let _ = std::fs::remove_dir_all(temp_dir);
    }
}
