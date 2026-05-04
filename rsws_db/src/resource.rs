//! 资源仓储层

use rsws_common::error::RswsError;
use rsws_model::resource::Resource;
use sqlx::PgPool;

/// 资源仓储
pub struct ResourceRepository {
    pool: PgPool,
}

impl ResourceRepository {
    /// 创建资源仓储实例
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 根据 ID 获取资源
    pub async fn get_by_id(&self, id: i64) -> Result<Option<Resource>, RswsError> {
        let resource = sqlx::query_as::<_, Resource>(
            "SELECT id, user_id, title, description, price, category_id, file_url, thumbnail_url, is_active, detail_description, specifications, usage_guide, precautions, display_images, provider_type, provider_id, commission_rate, created_at, updated_at FROM resources WHERE id = $1 AND is_active = true",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to get resource: {}", e)))?;

        Ok(resource)
    }

    /// 获取资源列表
    pub async fn get_list(
        &self,
        category_id: Option<i64>,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<Resource>, i64), RswsError> {
        let offset = (page - 1) * page_size;

        let (resources, total) = if let Some(cat_id) = category_id {
            let resources = sqlx::query_as::<_, Resource>(
                "SELECT id, user_id, title, description, price, category_id, file_url, thumbnail_url, is_active, detail_description, specifications, usage_guide, precautions, display_images, provider_type, provider_id, commission_rate, created_at, updated_at FROM resources WHERE category_id = $1 AND is_active = true ORDER BY created_at DESC LIMIT $2 OFFSET $3",
            )
            .bind(cat_id)
            .bind(page_size)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| RswsError::internal(format!("Failed to get resources: {}", e)))?;

            let total: (i64,) = sqlx::query_as(
                "SELECT COUNT(*) FROM resources WHERE category_id = $1 AND is_active = true",
            )
            .bind(cat_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| RswsError::internal(format!("Failed to count resources: {}", e)))?;

            (resources, total.0)
        } else {
            let resources = sqlx::query_as::<_, Resource>(
                "SELECT id, user_id, title, description, price, category_id, file_url, thumbnail_url, is_active, detail_description, specifications, usage_guide, precautions, display_images, provider_type, provider_id, commission_rate, created_at, updated_at FROM resources WHERE is_active = true ORDER BY created_at DESC LIMIT $1 OFFSET $2",
            )
            .bind(page_size)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| RswsError::internal(format!("Failed to get resources: {}", e)))?;

            let total: (i64,) = sqlx::query_as(
                "SELECT COUNT(*) FROM resources WHERE is_active = true",
            )
            .fetch_one(&self.pool)
            .await
            .map_err(|e| RswsError::internal(format!("Failed to count resources: {}", e)))?;

            (resources, total.0)
        };

        Ok((resources, total))
    }

    /// 获取用户上传的资源
    pub async fn get_user_resources(
        &self,
        user_id: i64,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<Resource>, i64), RswsError> {
        let offset = (page - 1) * page_size;

        let resources = sqlx::query_as::<_, Resource>(
            "SELECT id, user_id, title, description, price, category_id, file_url, thumbnail_url, is_active, detail_description, specifications, usage_guide, precautions, display_images, provider_type, provider_id, commission_rate, created_at, updated_at FROM resources WHERE user_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(user_id)
        .bind(page_size)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to get user resources: {}", e)))?;

        let total: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM resources WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to count user resources: {}", e)))?;

        Ok((resources, total.0))
    }
}

// ==================== 单元测试 ====================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_repository_new() {
        // 仅测试构造函数
    }
}
