//! 资源服务

use rsws_common::error::RswsError;
use rsws_common::error_code::ErrorCode;
use rsws_db::ResourceRepository;
use rsws_model::resource::{CreateResourceRequest, Resource, UpdateResourceRequest};
use std::sync::Arc;
use tracing::info;

/// 资源服务
pub struct ResourceService {
    resource_repo: Arc<ResourceRepository>,
}

impl ResourceService {
    /// 创建资源服务实例
    pub fn new(resource_repo: Arc<ResourceRepository>) -> Self {
        Self { resource_repo }
    }

    /// 获取资源
    pub async fn get(&self, resource_id: i64) -> Result<Option<Resource>, RswsError> {
        self.resource_repo.get_by_id(resource_id).await
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
        user_id: i64,
        owner_type: &str,
        provider_id: i64,
    ) -> Result<Resource, RswsError> {
        // 验证价格
        if req.price < 0 {
            return Err(RswsError::business(ErrorCode::INVALID_PARAMETER));
        }

        let resource = self.resource_repo.create(user_id, &req, owner_type, provider_id).await?;

        info!("Resource created: {} by user {}", resource.id, user_id);

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

        if existing.user_id != user_id {
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

        if existing.user_id != user_id {
            return Err(RswsError::business(ErrorCode::AUTH_PERMISSION_DENIED));
        }

        self.resource_repo.delete(resource_id).await?;

        info!("Resource deleted: {} by user {}", resource_id, user_id);

        Ok(())
    }

    /// 删除资源（管理员，跳过归属校验）
    pub async fn admin_delete(&self, resource_id: i64) -> Result<(), RswsError> {
        // 仅检查资源是否存在
        let _existing = self
            .resource_repo
            .get_by_id(resource_id)
            .await?
            .ok_or_else(|| RswsError::business(ErrorCode::RESOURCE_NOT_FOUND))?;

        self.resource_repo.delete(resource_id).await?;

        info!("Resource deleted by admin: {}", resource_id);

        Ok(())
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
