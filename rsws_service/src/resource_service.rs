//! 资源服务

use rsws_common::error::RswsError;
use rsws_db::ResourceRepository;
use rsws_model::resource::Resource;
use std::sync::Arc;

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
        self.resource_repo.get_list(category_id, page, page_size).await
    }
}
