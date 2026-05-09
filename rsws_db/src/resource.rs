//! Resource repository

use rsws_common::error::RswsError;
use rsws_model::resource::{CreateResourceRequest, Resource, UpdateResourceRequest};
use sqlx::PgPool;
use rsws_common::snowflake::next_id;

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

    /// 创建资源
    pub async fn create(
        &self,
        user_id: i64,
        req: &CreateResourceRequest,
    ) -> Result<Resource, RswsError> {
        // 生成雪花 ID
        let id = next_id();

        // 将 display_images 从 Vec<String> 转换为 PostgreSQL 数组格式
        let display_images_array: Option<Vec<String>> = req.display_images.clone();

        let resource = sqlx::query_as::<_, Resource>(
            "INSERT INTO resources (id, user_id, title, description, price, category_id, file_url, thumbnail_url, detail_description, specifications, usage_guide, precautions, display_images, provider_type, provider_id, commission_rate) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, 'user', $14, 0) RETURNING id, user_id, title, description, price, category_id, file_url, thumbnail_url, is_active, detail_description, specifications, usage_guide, precautions, display_images, provider_type, provider_id, commission_rate, created_at, updated_at"
        )
        .bind(id)
        .bind(user_id)
        .bind(&req.title)
        .bind(&req.description)
        .bind(req.price)
        .bind(req.category_id)
        .bind(&req.file_url)
        .bind(&req.thumbnail_url)
        .bind(&req.detail_description)
        .bind(&req.specifications)
        .bind(&req.usage_guide)
        .bind(&req.precautions)
        .bind(&display_images_array)
        .bind(user_id) // provider_id = user_id for user-created resources
        .fetch_one(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to create resource: {}", e)))?;

        Ok(resource)
    }

    /// 更新资源
    pub async fn update(
        &self,
        id: i64,
        req: &UpdateResourceRequest,
    ) -> Result<Resource, RswsError> {
        // 先获取当前资源
        let mut resource = self.get_by_id(id).await?
            .ok_or_else(|| RswsError::internal("Resource not found".to_string()))?;

        // 合并更新字段
        if let Some(title) = &req.title {
            resource.title = title.clone();
        }
        if let Some(description) = &req.description {
            resource.description = Some(description.clone());
        }
        if let Some(price) = req.price {
            resource.price = price;
        }
        if let Some(category_id) = req.category_id {
            resource.category_id = Some(category_id);
        }
        if let Some(file_url) = &req.file_url {
            resource.file_url = Some(file_url.clone());
        }
        if let Some(thumbnail_url) = &req.thumbnail_url {
            resource.thumbnail_url = Some(thumbnail_url.clone());
        }
        if let Some(is_active) = req.is_active {
            resource.is_active = is_active;
        }
        if let Some(detail_description) = &req.detail_description {
            resource.detail_description = Some(detail_description.clone());
        }
        if let Some(specifications) = &req.specifications {
            resource.specifications = Some(specifications.clone());
        }
        if let Some(usage_guide) = &req.usage_guide {
            resource.usage_guide = Some(usage_guide.clone());
        }
        if let Some(precautions) = &req.precautions {
            resource.precautions = Some(precautions.clone());
        }
        if let Some(display_images) = &req.display_images {
            resource.display_images = Some(serde_json::to_value(display_images).unwrap_or(serde_json::Value::Null));
        }

        // 更新数据库
        let updated = sqlx::query_as::<_, Resource>(
            "UPDATE resources SET title = $1, description = $2, price = $3, category_id = $4, file_url = $5, thumbnail_url = $6, is_active = $7, detail_description = $8, specifications = $9, usage_guide = $10, precautions = $11, display_images = $12, updated_at = NOW() WHERE id = $13 RETURNING id, user_id, title, description, price, category_id, file_url, thumbnail_url, is_active, detail_description, specifications, usage_guide, precautions, display_images, provider_type, provider_id, commission_rate, created_at, updated_at"
        )
        .bind(&resource.title)
        .bind(&resource.description)
        .bind(resource.price)
        .bind(resource.category_id)
        .bind(&resource.file_url)
        .bind(&resource.thumbnail_url)
        .bind(resource.is_active)
        .bind(&resource.detail_description)
        .bind(&resource.specifications)
        .bind(&resource.usage_guide)
        .bind(&resource.precautions)
        .bind(&resource.display_images)
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to update resource: {}", e)))?;

        Ok(updated)
    }

    /// 删除资源（软删除，设置 is_active = false）
    pub async fn delete(
        &self,
        id: i64,
    ) -> Result<(), RswsError> {
        sqlx::query("UPDATE resources SET is_active = false, updated_at = NOW() WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| RswsError::internal(format!("Failed to delete resource: {}", e)))?;

        Ok(())
    }

    /// 获取资源列表（支持关键词搜索）
    pub async fn get_list_with_search(
        &self,
        category_id: Option<i64>,
        search: Option<&str>,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<Resource>, i64), RswsError> {
        let offset = (page - 1) * page_size;

        // 构建 WHERE 条件
        let _base_where = "is_active = true";
        let (resources, total) = match (category_id, search) {
            (Some(cat_id), Some(kw)) => {
                let kw_pattern = format!("%{}%", kw);
                let resources = sqlx::query_as::<_, Resource>(
                    "SELECT id, user_id, title, description, price, category_id, file_url, thumbnail_url, is_active, detail_description, specifications, usage_guide, precautions, display_images, provider_type, provider_id, commission_rate, created_at, updated_at FROM resources WHERE category_id = $1 AND is_active = true AND (title ILIKE $2 OR description ILIKE $2) ORDER BY created_at DESC LIMIT $3 OFFSET $4",
                )
                .bind(cat_id)
                .bind(&kw_pattern)
                .bind(page_size)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
                .map_err(|e| RswsError::internal(format!("Failed to get resources: {}", e)))?;

                let total: (i64,) = sqlx::query_as(
                    "SELECT COUNT(*) FROM resources WHERE category_id = $1 AND is_active = true AND (title ILIKE $2 OR description ILIKE $2)",
                )
                .bind(cat_id)
                .bind(&kw_pattern)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| RswsError::internal(format!("Failed to count resources: {}", e)))?;

                (resources, total.0)
            }
            (Some(cat_id), None) => {
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
            }
            (None, Some(kw)) => {
                let kw_pattern = format!("%{}%", kw);
                let resources = sqlx::query_as::<_, Resource>(
                    "SELECT id, user_id, title, description, price, category_id, file_url, thumbnail_url, is_active, detail_description, specifications, usage_guide, precautions, display_images, provider_type, provider_id, commission_rate, created_at, updated_at FROM resources WHERE is_active = true AND (title ILIKE $1 OR description ILIKE $1) ORDER BY created_at DESC LIMIT $2 OFFSET $3",
                )
                .bind(&kw_pattern)
                .bind(page_size)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
                .map_err(|e| RswsError::internal(format!("Failed to get resources: {}", e)))?;

                let total: (i64,) = sqlx::query_as(
                    "SELECT COUNT(*) FROM resources WHERE is_active = true AND (title ILIKE $1 OR description ILIKE $1)",
                )
                .bind(&kw_pattern)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| RswsError::internal(format!("Failed to count resources: {}", e)))?;

                (resources, total.0)
            }
            (None, None) => {
                // 复用原有的无过滤查询
                self.get_list(category_id, page, page_size).await?
            }
        };

        Ok((resources, total))
    }

    /// 递增资源下载计数
    pub async fn increment_download_count(&self, resource_id: i64) -> Result<(), RswsError> {
        sqlx::query("UPDATE resources SET download_count = COALESCE(download_count, 0) + 1, updated_at = NOW() WHERE id = $1")
            .bind(resource_id)
            .execute(&self.pool)
            .await
            .map_err(|e| RswsError::internal(format!("Failed to increment download count: {}", e)))?;
        Ok(())
    }

    /// 获取基础统计（资源总数 + 已上线资源数 + 过去30天新增资源数）
    pub async fn get_basic_stats(&self) -> Result<(i64, i64, i64), RswsError> {
        let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM resources")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| RswsError::internal(format!("Failed to count resources: {}", e)))?;

        let active: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM resources WHERE is_active = true"
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to count active resources: {}", e)))?;

        let new_30d: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM resources WHERE created_at >= NOW() - INTERVAL '30 days'"
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to count recent resources: {}", e)))?;

        Ok((total.0, active.0, new_30d.0))
    }
}

// ==================== 单元测试 ====================

#[cfg(test)]
mod tests {
    

    #[test]
    fn test_resource_repository_new() {
        // 仅测试构造函数
    }

    // create, update, delete 方法需要数据库测试，这里省略
}
