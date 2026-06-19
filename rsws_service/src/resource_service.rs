//! 资源服务

use crate::oss_service::StorageService;
use crate::order_service::OrderService;
use rsws_common::error::RswsError;
use rsws_common::error_code::ErrorCode;
use rsws_db::ResourceRepository;
use rsws_model::resource::{
    CreateResourceRequest, Resource, ResourceDetailResponse, UpdateResourceRequest,
    OWNER_TYPE_USER,
};
use rust_decimal::Decimal;
use std::sync::Arc;
use tracing::{info, warn};

/// 资源服务
pub struct ResourceService {
    resource_repo: Arc<ResourceRepository>,
    order_service: Option<Arc<OrderService>>,
    config_service: Option<crate::config_service::ConfigService>,
}

impl ResourceService {
    /// 创建资源服务实例
    pub fn new(resource_repo: Arc<ResourceRepository>) -> Self {
        Self {
            resource_repo,
            order_service: None,
            config_service: None,
        }
    }

    /// 创建资源服务实例（带 OSS 配置）
    pub fn with_oss(
        resource_repo: Arc<ResourceRepository>,
        config_service: crate::config_service::ConfigService,
    ) -> Self {
        Self {
            resource_repo,
            order_service: None,
            config_service: Some(config_service),
        }
    }

    /// 设置订单服务（用于购买检查）
    pub fn set_order_service(&mut self, order_service: Arc<OrderService>) {
        self.order_service = Some(order_service);
    }

    /// 获取 OSS 存储服务（如果配置了）
    async fn get_storage_service(&self) -> Option<StorageService> {
        if let Some(ref config_service) = self.config_service {
            match config_service.get_storage_config().await {
                Ok(config) => match StorageService::new(&config).await {
                    Ok(service) => return Some(service),
                    Err(e) => {
                        warn!("Failed to create storage service: {}", e);
                        return None;
                    }
                },
                Err(e) => {
                    warn!("Failed to get storage config: {}", e);
                    return None;
                }
            }
        }
        None
    }

    /// 从 URL 提取文件 key
    fn extract_key_from_url(&self, url: &str) -> Option<String> {
        // 处理不同存储后端的 URL 格式
        // 本地：http://host:port/uploads/resources/20240601/12345678.zip
        // S3：https://bucket.s3.region.amazonaws.com/resources/20240601/12345678.zip
        // 自定义域名：https://cdn.example.com/resources/20240601/12345678.zip

        // 简单处理：取最后一个 "/" 之后的路径作为 key 的前缀
        // 实际应该根据配置的 endpoint 和 custom_domain 来解析
        if let Some(pos) = url.rfind("/resources/") {
            return Some(url[pos + 1..].to_string());
        }
        None
    }

    /// 获取资源
    pub async fn get(&self, resource_id: i64) -> Result<Option<Resource>, RswsError> {
        self.resource_repo.get_by_id(resource_id).await
    }

    /// 获取资源详情（含购买状态和付费内容截断）
    ///
    /// - 未登录或未购买且资源有价格时，截断 `detail_description` 至 25% 并隐藏 `file_url`
    /// - 已购买或免费资源返回完整内容
    pub async fn get_detail(
        &self,
        user_id: Option<i64>,
        resource_id: i64,
    ) -> Result<Option<ResourceDetailResponse>, RswsError> {
        let resource = match self.resource_repo.get_by_id(resource_id).await? {
            Some(r) => r,
            None => return Ok(None),
        };

        // 检查是否已购买
        let is_purchased = if let Some(uid) = user_id {
            if let Some(ref order_service) = self.order_service {
                order_service.check_purchased(uid, resource_id).await.unwrap_or(false)
            } else {
                false
            }
        } else {
            false
        };

        // 付费资源且未购买：截断 detail_description 并隐藏 file_url
        let is_paid = resource.price > Decimal::ZERO;
        let (detail_description, file_url) = if is_paid && !is_purchased {
            let truncated = resource
                .detail_description
                .as_ref()
                .map(|content| {
                    let len = content.chars().count();
                    let cutoff = (len as f64 * 0.25).floor() as usize;
                    if cutoff >= len {
                        content.clone()
                    } else {
                        let truncated: String = content.chars().take(cutoff).collect();
                        format!("{}...", truncated)
                    }
                });
            (truncated, None) // 隐藏 file_url 防止直接下载绕过
        } else {
            (resource.detail_description.clone(), resource.file_url.clone())
        };

        Ok(Some(ResourceDetailResponse {
            id: resource.id,
            title: resource.title,
            description: resource.description,
            price: resource.price,
            category_id: resource.category_id,
            file_url,
            thumbnail_url: resource.thumbnail_url,
            is_active: resource.is_active,
            detail_description,
            specifications: resource.specifications,
            usage_guide: resource.usage_guide,
            precautions: resource.precautions,
            display_images: resource.display_images,
            owner_type: resource.owner_type,
            provider_id: resource.provider_id,
            supported_os: resource.supported_os,
            commission_rate: resource.commission_rate,
            download_count: resource.download_count,
            created_at: resource.created_at,
            updated_at: resource.updated_at,
            is_purchased,
        }))
    }

    /// 获取资源列表
    pub async fn list(
        &self,
        category_id: Option<i64>,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<Resource>, i64), RswsError> {
        self.resource_repo
            .get_list(category_id, page, page_size)
            .await
    }

    /// 搜索资源列表
    pub async fn search(
        &self,
        category_id: Option<i64>,
        search: Option<&str>,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<Resource>, i64), RswsError> {
        self.resource_repo
            .get_list_with_search(category_id, search, page, page_size)
            .await
    }

    /// 递增资源下载计数
    pub async fn increment_download_count(&self, resource_id: i64) -> Result<(), RswsError> {
        self.resource_repo
            .increment_download_count(resource_id)
            .await
    }

    /// 创建资源
    pub async fn create(
        &self,
        req: CreateResourceRequest,
        owner_type: &str,
        provider_id: i64,
    ) -> Result<Resource, RswsError> {
        // 验证价格
        if req.price < rust_decimal::Decimal::ZERO {
            return Err(RswsError::business(ErrorCode::INVALID_PARAMETER));
        }

        let resource = self
            .resource_repo
            .create(&req, owner_type, provider_id)
            .await?;

        info!(
            "Resource created: {} ({}:{})",
            resource.id, owner_type, provider_id
        );

        Ok(resource)
    }

    /// 更新资源
    pub async fn update(
        &self,
        resource_id: i64,
        req: UpdateResourceRequest,
        user_id: i64,
    ) -> Result<Resource, RswsError> {
        // 检查资源是否存在且属于该用户
        let existing = self
            .resource_repo
            .get_by_id(resource_id)
            .await?
            .ok_or_else(|| RswsError::business(ErrorCode::RESOURCE_NOT_FOUND))?;

        if existing.provider_id != Some(user_id) || existing.owner_type != OWNER_TYPE_USER {
            return Err(RswsError::business(ErrorCode::AUTH_PERMISSION_DENIED));
        }

        let updated = self.resource_repo.update(resource_id, &req).await?;

        info!("Resource updated: {} by user {}", resource_id, user_id);

        Ok(updated)
    }

    /// 删除资源
    pub async fn delete(&self, resource_id: i64, user_id: i64) -> Result<(), RswsError> {
        // 检查资源是否存在且属于该用户
        let existing = self
            .resource_repo
            .get_by_id(resource_id)
            .await?
            .ok_or_else(|| RswsError::business(ErrorCode::RESOURCE_NOT_FOUND))?;

        if existing.provider_id != Some(user_id) || existing.owner_type != OWNER_TYPE_USER {
            return Err(RswsError::business(ErrorCode::AUTH_PERMISSION_DENIED));
        }

        // 删除 OSS 中的文件
        self.delete_resource_files(&existing).await;

        // 删除数据库记录
        self.resource_repo.delete(resource_id).await?;

        info!("Resource deleted: {} by user {}", resource_id, user_id);

        Ok(())
    }

    /// 删除资源（管理员，跳过归属校验）
    pub async fn admin_delete(&self, resource_id: i64) -> Result<(), RswsError> {
        // 仅检查资源是否存在
        let existing = self
            .resource_repo
            .get_by_id(resource_id)
            .await?
            .ok_or_else(|| RswsError::business(ErrorCode::RESOURCE_NOT_FOUND))?;

        // 删除 OSS 中的文件
        self.delete_resource_files(&existing).await;

        // 删除数据库记录
        self.resource_repo.delete(resource_id).await?;

        info!("Resource deleted by admin: {}", resource_id);

        Ok(())
    }

    /// 删除资源关联的文件（从 OSS）
    async fn delete_resource_files(&self, resource: &Resource) {
        if let Some(ref storage_service) = self.get_storage_service().await {
            // 删除主文件
            if let Some(ref file_url) = resource.file_url {
                if let Some(key) = self.extract_key_from_url(file_url) {
                    match storage_service.delete(&key).await {
                        Ok(_) => info!("Deleted file from OSS: {}", key),
                        Err(e) => warn!("Failed to delete file from OSS: {} (key: {})", e, key),
                    }
                }
            }

            // 删除缩略图
            if let Some(ref thumbnail_url) = resource.thumbnail_url {
                if let Some(key) = self.extract_key_from_url(thumbnail_url) {
                    match storage_service.delete(&key).await {
                        Ok(_) => info!("Deleted thumbnail from OSS: {}", key),
                        Err(e) => {
                            warn!("Failed to delete thumbnail from OSS: {} (key: {})", e, key)
                        }
                    }
                }
            }
        }
    }

    /// 管理员更新资源（跳过归属校验）
    pub async fn admin_update(
        &self,
        resource_id: i64,
        req: UpdateResourceRequest,
    ) -> Result<Resource, RswsError> {
        // 仅检查资源是否存在
        let _existing = self
            .resource_repo
            .get_by_id(resource_id)
            .await?
            .ok_or_else(|| RswsError::business(ErrorCode::RESOURCE_NOT_FOUND))?;

        let updated = self.resource_repo.update(resource_id, &req).await?;

        info!("Resource updated by admin: {}", resource_id);

        Ok(updated)
    }
}
