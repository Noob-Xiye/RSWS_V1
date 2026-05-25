//! Resource repository

use rsws_common::error::RswsError;
use rsws_common::snowflake::next_id;
use rsws_model::resource::{CreateResourceRequest, Resource, UpdateResourceRequest};
use sqlx::PgPool;

/// 璧勬簮浠撳偍
pub struct ResourceRepository {
    pool: PgPool,
}

impl ResourceRepository {
    /// 鍒涘缓璧勬簮浠撳偍瀹炰緥
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 鏍规嵁 ID 鑾峰彇璧勬簮
    pub async fn get_by_id(&self, id: i64) -> Result<Option<Resource>, RswsError> {
        let resource = sqlx::query_as::<_, Resource>(
            "SELECT * FROM resources WHERE id = $1 AND is_active = true",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to get resource: {}", e)))?;

        Ok(resource)
    }

    /// 鑾峰彇璧勬簮鍒楄〃
    pub async fn get_list(
        &self,
        category_id: Option<i64>,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<Resource>, i64), RswsError> {
        let offset = (page - 1) * page_size;

        let (resources, total) = if let Some(cat_id) = category_id {
            let resources = sqlx::query_as::<_, Resource>(
                "SELECT * FROM resources WHERE category_id = $1 AND is_active = true ORDER BY created_at DESC LIMIT $2 OFFSET $3",
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
                "SELECT * FROM resources WHERE is_active = true ORDER BY created_at DESC LIMIT $1 OFFSET $2",
            )
            .bind(page_size)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| RswsError::internal(format!("Failed to get resources: {}", e)))?;

            let total: (i64,) =
                sqlx::query_as("SELECT COUNT(*) FROM resources WHERE is_active = true")
                    .fetch_one(&self.pool)
                    .await
                    .map_err(|e| {
                        RswsError::internal(format!("Failed to count resources: {}", e))
                    })?;

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
            "SELECT * FROM resources WHERE owner_type = 'user' AND provider_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(user_id)
        .bind(page_size)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to get user resources: {}", e)))?;

        let total: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM resources WHERE owner_type = 'user' AND provider_id = $1",
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to count user resources: {}", e)))?;

        Ok((resources, total.0))
    }

    /// 鍒涘缓璧勬簮
    pub async fn create(
        &self,
        req: &CreateResourceRequest,
        owner_type: &str,
        provider_id: i64,
    ) -> Result<Resource, RswsError> {
        // 鐢熸垚闆姳 ID
        let id = next_id();

        // 灏?display_images 浠?Vec<String> 杞崲涓?PostgreSQL 鏁扮粍鏍煎紡
        let display_images_array: Option<Vec<String>> = req.display_images.clone();

        // supported_os: Vec<String> → serde_json::Value for JSONB column
        let supported_os_json: Option<serde_json::Value> = req.supported_os.as_ref().map(|v| {
            serde_json::Value::Array(
                v.iter()
                    .map(|s| serde_json::Value::String(s.clone()))
                    .collect(),
            )
        });

        let resource = sqlx::query_as::<_, Resource>(
            "INSERT INTO resources (id, title, description, price, category_id, file_url, thumbnail_url, detail_description, specifications, usage_guide, precautions, display_images, supported_os, provider_type, provider_id, commission_rate) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, 0) RETURNING *"
        )
        .bind(id)
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
        .bind(&supported_os_json)
        .bind(owner_type)
        .bind(provider_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to create resource: {}", e)))?;

        Ok(resource)
    }

    /// 鏇存柊璧勬簮
    pub async fn update(
        &self,
        id: i64,
        req: &UpdateResourceRequest,
    ) -> Result<Resource, RswsError> {
        // 鍏堣幏鍙栧綋鍓嶈祫婧?
        let mut resource = self
            .get_by_id(id)
            .await?
            .ok_or_else(|| RswsError::internal("Resource not found".to_string()))?;

        // 鍚堝苟鏇存柊瀛楁
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
            resource.display_images =
                Some(serde_json::to_value(display_images).unwrap_or(serde_json::Value::Null));
        }

        // 鏇存柊鏁版嵁搴?
        let updated = sqlx::query_as::<_, Resource>(
            "UPDATE resources SET title = $1, description = $2, price = $3, category_id = $4, file_url = $5, thumbnail_url = $6, is_active = $7, detail_description = $8, specifications = $9, usage_guide = $10, precautions = $11, display_images = $12, supported_os = $13, updated_at = NOW() WHERE id = $14 RETURNING *"
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
        .bind(&resource.supported_os)
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to update resource: {}", e)))?;

        Ok(updated)
    }

    /// 鍒犻櫎璧勬簮锛堣蒋鍒犻櫎锛岃缃?is_active = false锛?
    pub async fn delete(&self, id: i64) -> Result<(), RswsError> {
        sqlx::query("UPDATE resources SET is_active = false, updated_at = NOW() WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| RswsError::internal(format!("Failed to delete resource: {}", e)))?;

        Ok(())
    }

    /// 鑾峰彇璧勬簮鍒楄〃锛堟敮鎸佸叧閿瘝鎼滅储锛?
    pub async fn get_list_with_search(
        &self,
        category_id: Option<i64>,
        search: Option<&str>,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<Resource>, i64), RswsError> {
        let offset = (page - 1) * page_size;

        // 鏋勫缓 WHERE 鏉′欢
        let _base_where = "is_active = true";
        let (resources, total) = match (category_id, search) {
            (Some(cat_id), Some(kw)) => {
                let kw_pattern = format!("%{}%", kw);
                let resources = sqlx::query_as::<_, Resource>(
                    "SELECT * FROM resources WHERE category_id = $1 AND is_active = true AND (title ILIKE $2 OR description ILIKE $2) ORDER BY created_at DESC LIMIT $3 OFFSET $4",
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
                    "SELECT * FROM resources WHERE category_id = $1 AND is_active = true ORDER BY created_at DESC LIMIT $2 OFFSET $3",
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
                    "SELECT * FROM resources WHERE is_active = true AND (title ILIKE $1 OR description ILIKE $1) ORDER BY created_at DESC LIMIT $2 OFFSET $3",
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
                // 澶嶇敤鍘熸湁鐨勬棤杩囨护鏌ヨ
                self.get_list(category_id, page, page_size).await?
            }
        };

        Ok((resources, total))
    }

    /// 閫掑璧勬簮涓嬭浇璁℃暟
    pub async fn increment_download_count(&self, resource_id: i64) -> Result<(), RswsError> {
        sqlx::query("UPDATE resources SET download_count = COALESCE(download_count, 0) + 1, updated_at = NOW() WHERE id = $1")
            .bind(resource_id)
            .execute(&self.pool)
            .await
            .map_err(|e| RswsError::internal(format!("Failed to increment download count: {}", e)))?;
        Ok(())
    }

    /// 鑾峰彇鍩虹缁熻锛堣祫婧愭€绘暟 + 宸蹭笂绾胯祫婧愭暟 + 杩囧幓30澶╂柊澧炶祫婧愭暟锛?
    pub async fn get_basic_stats(&self) -> Result<(i64, i64, i64), RswsError> {
        let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM resources")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| RswsError::internal(format!("Failed to count resources: {}", e)))?;

        let active: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM resources WHERE is_active = true")
                .fetch_one(&self.pool)
                .await
                .map_err(|e| {
                    RswsError::internal(format!("Failed to count active resources: {}", e))
                })?;

        let new_30d: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM resources WHERE created_at >= NOW() - INTERVAL '30 days'",
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to count recent resources: {}", e)))?;

        Ok((total.0, active.0, new_30d.0))
    }
}

// ==================== 鍗曞厓娴嬭瘯 ====================

#[cfg(test)]
mod tests {

    #[test]
    fn test_resource_repository_new() {
        // 浠呮祴璇曟瀯閫犲嚱鏁?
    }

    // create, update, delete 鏂规硶闇€瑕佹暟鎹簱娴嬭瘯锛岃繖閲岀渷鐣?
}
