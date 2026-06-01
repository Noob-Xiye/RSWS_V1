//! 用户 API Key 仓储层

use base64::{engine::general_purpose, Engine as _};
use chrono::{Duration, Utc};
use rand::SeedableRng;
use rsws_common::error::RswsError;
use rsws_model::api_key::{ApiKey, CreateApiKeyRequest};
use sqlx::PgPool;

/// 用户 API Key 仓储
pub struct UserApiKeyRepository {
    pool: PgPool,
}

impl UserApiKeyRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 创建用户 API Key
    pub async fn create(
        &self,
        user_id: i64,
        req: &CreateApiKeyRequest,
    ) -> Result<ApiKey, RswsError> {
        use rand::Rng;

        let mut rng = rand::rngs::StdRng::from_os_rng();
        let key_bytes: [u8; 32] = rng.random();
        let api_key = format!(
            "usr_ak_{}",
            general_purpose::URL_SAFE_NO_PAD.encode(key_bytes)
        );

        let permissions_json =
            serde_json::to_value(&req.permissions)
                .map_err(|e| RswsError::internal(format!("Failed to serialize permissions: {}", e)))?;

        let rate_limit = req.rate_limit.unwrap_or(100);
        let expires_at = req.expires_in_days.map(|d| Utc::now() + Duration::days(d as i64));

        let record = sqlx::query_as::<_, ApiKey>(
            r#"
            INSERT INTO user_api_keys (user_id, api_key, name, permissions, rate_limit, expires_at, is_active)
            VALUES ($1, $2, $3, $4, $5, $6, true)
            RETURNING id, user_id, api_key, name, permissions, rate_limit, last_used_at, expires_at, is_active, created_at, updated_at
            "#,
        )
        .bind(user_id)
        .bind(&api_key)
        .bind(&req.name)
        .bind(&permissions_json)
        .bind(rate_limit)
        .bind(expires_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to create user API key: {}", e)))?;

        Ok(record)
    }

    /// 获取用户的所有 API Key
    pub async fn get_by_user(&self, user_id: i64) -> Result<Vec<ApiKey>, RswsError> {
        sqlx::query_as::<_, ApiKey>(
            r#"SELECT id, user_id, api_key, name, permissions, rate_limit, last_used_at, expires_at, is_active, created_at, updated_at
               FROM user_api_keys WHERE user_id = $1 ORDER BY created_at DESC"#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to get user API keys: {}", e)))
    }

    /// 根据 api_key 获取记录（用于签名验证）
    pub async fn get_by_key(&self, api_key: &str) -> Result<Option<ApiKey>, RswsError> {
        sqlx::query_as::<_, ApiKey>(
            r#"SELECT id, user_id, api_key, name, permissions, rate_limit, last_used_at, expires_at, is_active, created_at, updated_at
               FROM user_api_keys WHERE api_key = $1 AND is_active = true"#,
        )
        .bind(api_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to get user API key: {}", e)))
    }

    /// 更新最后使用时间
    pub async fn update_last_used(&self, key_id: i64) -> Result<(), RswsError> {
        sqlx::query("UPDATE user_api_keys SET last_used_at = NOW() WHERE id = $1")
            .bind(key_id)
            .execute(&self.pool)
            .await
            .map_err(|e| RswsError::internal(format!("Failed to update last_used: {}", e)))?;
        Ok(())
    }

    /// 删除 API Key
    pub async fn delete(&self, key_id: i64, user_id: i64) -> Result<bool, RswsError> {
        let result = sqlx::query("DELETE FROM user_api_keys WHERE id = $1 AND user_id = $2")
            .bind(key_id)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(|e| RswsError::internal(format!("Failed to delete user API key: {}", e)))?;
        Ok(result.rows_affected() > 0)
    }

    /// 切换启用/停用状态
    pub async fn toggle_active(&self, key_id: i64, user_id: i64) -> Result<bool, RswsError> {
        let result = sqlx::query_as::<_, (bool,)>(
            r#"UPDATE user_api_keys SET is_active = NOT is_active, updated_at = NOW()
               WHERE id = $1 AND user_id = $2 RETURNING is_active"#,
        )
        .bind(key_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to toggle API key status: {}", e)))?;
        match result {
            Some((is_active,)) => Ok(is_active),
            None => Ok(false),
        }
    }

    /// 删除用户的所有 API Key（管理员用）
    pub async fn delete_by_user(&self, user_id: i64) -> Result<u64, RswsError> {
        let result = sqlx::query("DELETE FROM user_api_keys WHERE user_id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(|e| RswsError::internal(format!("Failed to delete user API keys: {}", e)))?;
        Ok(result.rows_affected())
    }
}
