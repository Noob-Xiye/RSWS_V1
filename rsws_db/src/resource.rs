use rsws_common::error::ServiceError;
use rsws_model::resource::Resource;
use sqlx::PgPool;

pub struct ResourceRepository {
    pool: PgPool,
}

impl ResourceRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // 根据ID获取资源
    pub async fn get_by_id(&self, id: i64) -> Result<Option<Resource>, ServiceError> {
        let result = sqlx::query_as!(
            Resource,
            "SELECT id, user_id, title, description, price, category_id, file_url, thumbnail_url, is_active, detail_description, specifications, usage_guide, precautions, display_images, provider_type, provider_id, commission_rate, created_at, updated_at FROM resources WHERE id = $1 AND is_active = true",
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        Ok(result)
    }

    // 获取资源列表
    pub async fn get_list(
        &self,
        category_id: Option<i64>,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<Resource>, i64), ServiceError> {
        let offset = (page - 1) * page_size;

        let (resources, total) = if let Some(category_id) = category_id {
            let resources = sqlx::query_as!(
                Resource,
                "SELECT id, title, description, price, category_id, file_url, thumbnail_url, is_active, created_at, updated_at FROM resources WHERE category_id = $1 AND is_active = true ORDER BY created_at DESC LIMIT $2 OFFSET $3",
                category_id,
                page_size,
                offset
            )
            .fetch_all(&self.pool)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

            let total: i64 = sqlx::query_scalar!(
                "SELECT COUNT(*) FROM resources WHERE category_id = $1 AND is_active = true",
                category_id
            )
            .fetch_one(&self.pool)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

            (resources, total)
        } else {
            let resources = sqlx::query_as!(
                Resource,
                "SELECT id, title, description, price, category_id, file_url, thumbnail_url, is_active, created_at, updated_at FROM resources WHERE is_active = true ORDER BY created_at DESC LIMIT $1 OFFSET $2",
                page_size,
                offset
            )
            .fetch_all(&self.pool)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

            let total: i64 =
                sqlx::query_scalar!("SELECT COUNT(*) FROM resources WHERE is_active = true")
                    .fetch_one(&self.pool)
                    .await
                    .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

            (resources, total)
        };

        Ok((resources, total))
    }
}
