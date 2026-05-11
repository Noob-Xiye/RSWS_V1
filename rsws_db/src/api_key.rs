//! API Key 仓储层
//!
//! 新设计（Cregis 方案）：
//! - api_key: 签名密钥，前端持有用于签名，不随请求传输
//! - 后端通过 user_id 查找 api_key 用于验签
//! - 不再需要 api_secret

use base64::{engine::general_purpose, Engine as _};
use chrono::Utc;
use rand::Rng;
use rsws_common::error::RswsError;
use rsws_model::api_key::{ApiKey, CreateApiKeyRequest};
use sqlx::PgPool;

/// API Key 仓储
pub struct ApiKeyRepository {
    pool: PgPool,
}

impl ApiKeyRepository {
    /// 创建 API Key 仓储实例
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 生成 API Key（作为签名密钥使用）
    ///
    /// 格式：ak_ + base64(random 32 bytes)
    /// 前端持有此值用于计算签名，但不随请求传输。
    fn generate_credentials() -> String {
        use rand::SeedableRng;
        let mut rng = rand::rngs::StdRng::from_os_rng();
        let api_key_bytes: [u8; 32] = rng.random();
        format!(
            "ak_{}",
            general_purpose::URL_SAFE_NO_PAD.encode(api_key_bytes)
        )
    }

    /// 创建 API Key
    pub async fn create(
        &self,
        user_id: i64,
        request: &CreateApiKeyRequest,
    ) -> Result<ApiKey, RswsError> {
        let api_key = Self::generate_credentials();

        // 计算过期时间
        let expires_at = request
            .expires_in_days
            .map(|days| Utc::now() + chrono::Duration::days(days as i64));

        let permissions_json = serde_json::to_value(&request.permissions)
            .map_err(|e| RswsError::internal(format!("Failed to serialize permissions: {}", e)))?;

        let api_key_record = sqlx::query_as::<_, ApiKey>(
            r#"
            INSERT INTO api_keys (user_id, api_key, name, permissions, rate_limit, expires_at, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, true, NOW(), NOW())
            RETURNING id, user_id, api_key, name, permissions, rate_limit, last_used_at, expires_at, is_active, created_at, updated_at
            "#
        )
        .bind(user_id)
        .bind(&api_key)
        .bind(&request.name)
        .bind(&permissions_json)
        .bind(request.rate_limit.unwrap_or(1000))
        .bind(expires_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to create API key: {}", e)))?;

        Ok(api_key_record)
    }

    /// 根据 API Key 获取记录（保留，用于旧 Header 兼容）
    pub async fn get_by_api_key(&self, api_key: &str) -> Result<Option<ApiKey>, RswsError> {
        let api_key_record = sqlx::query_as::<_, ApiKey>(
            "SELECT id, user_id, api_key, name, permissions, rate_limit, last_used_at, expires_at, is_active, created_at, updated_at FROM api_keys WHERE api_key = $1 AND is_active = true"
        )
        .bind(api_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to get API key: {}", e)))?;

        Ok(api_key_record)
    }

    /// 根据 user_id 获取活跃的 API Key（用于签名验证）
    ///
    /// Cregis 方案：前端传 user_id，后端用 user_id 查找 api_key 验签
    pub async fn get_active_key_by_user_id(
        &self,
        user_id: i64,
    ) -> Result<Option<ApiKey>, RswsError> {
        let api_key_record = sqlx::query_as::<_, ApiKey>(
            r#"
            SELECT id, user_id, api_key, name, permissions, rate_limit, last_used_at, expires_at, is_active, created_at, updated_at
            FROM api_keys 
            WHERE user_id = $1 AND is_active = true
            AND (expires_at IS NULL OR expires_at > NOW())
            ORDER BY created_at DESC
            LIMIT 1
            "#
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to get API key by user_id: {}", e)))?;

        Ok(api_key_record)
    }

    /// 验证 API Key（旧方式，保留用于降级兼容）
    /// 现在只验证 api_key 是否存在且活跃，不再验证 api_secret
    pub async fn validate(&self, api_key: &str) -> Result<Option<ApiKey>, RswsError> {
        let api_key_record = sqlx::query_as::<_, ApiKey>(
            r#"
            SELECT id, user_id, api_key, name, permissions, rate_limit, last_used_at, expires_at, is_active, created_at, updated_at
            FROM api_keys 
            WHERE api_key = $1 AND is_active = true
            AND (expires_at IS NULL OR expires_at > NOW())
            "#
        )
        .bind(api_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to validate API key: {}", e)))?;

        Ok(api_key_record)
    }

    /// 更新最后使用时间
    pub async fn update_last_used(&self, api_key_id: i64) -> Result<(), RswsError> {
        sqlx::query("UPDATE api_keys SET last_used_at = NOW() WHERE id = $1")
            .bind(api_key_id)
            .execute(&self.pool)
            .await
            .map_err(|e| RswsError::internal(format!("Failed to update last used: {}", e)))?;

        Ok(())
    }

    /// 获取用户的所有 API Key
    pub async fn get_user_api_keys(&self, user_id: i64) -> Result<Vec<ApiKey>, RswsError> {
        let api_keys = sqlx::query_as::<_, ApiKey>(
            "SELECT id, user_id, api_key, name, permissions, rate_limit, last_used_at, expires_at, is_active, created_at, updated_at FROM api_keys WHERE user_id = $1 ORDER BY created_at DESC"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to get user API keys: {}", e)))?;

        Ok(api_keys)
    }

    /// 删除 API Key
    pub async fn delete(&self, api_key_id: i64, user_id: i64) -> Result<bool, RswsError> {
        let result = sqlx::query("DELETE FROM api_keys WHERE id = $1 AND user_id = $2")
            .bind(api_key_id)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(|e| RswsError::internal(format!("Failed to delete API key: {}", e)))?;

        Ok(result.rows_affected() > 0)
    }

    /// 禁用指定 API Key
    pub async fn deactivate_by_id(&self, key_id: i64) -> Result<bool, RswsError> {
        let result =
            sqlx::query("UPDATE api_keys SET is_active = false, updated_at = NOW() WHERE id = $1")
                .bind(key_id)
                .execute(&self.pool)
                .await
                .map_err(|e| RswsError::internal(format!("Failed to deactivate API key: {}", e)))?;
        Ok(result.rows_affected() > 0)
    }

    /// 禁用用户所有 API Key（改密码/强制下线时使用）
    pub async fn deactivate_by_user(&self, user_id: i64) -> Result<u64, RswsError> {
        let result = sqlx::query("UPDATE api_keys SET is_active = false, updated_at = NOW() WHERE user_id = $1 AND is_active = true")
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(|e| RswsError::internal(format!("Failed to deactivate user API keys: {}", e)))?;
        Ok(result.rows_affected())
    }

    /// 获取用户所有活跃 API Key（用于 Redis 缓存清理）
    pub async fn get_active_keys_by_user(&self, user_id: i64) -> Result<Vec<ApiKey>, RswsError> {
        let keys = sqlx::query_as::<_, ApiKey>(
            "SELECT id, user_id, api_key, name, permissions, rate_limit, last_used_at, expires_at, is_active, created_at, updated_at FROM api_keys WHERE user_id = $1 AND is_active = true"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to get active API keys: {}", e)))?;
        Ok(keys)
    }
}

// ==================== 单元测试 ====================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_credentials() {
        let api_key = ApiKeyRepository::generate_credentials();

        assert!(api_key.starts_with("ak_"));
        assert!(api_key.len() > 10);
    }

    #[test]
    fn test_generate_credentials_unique() {
        let key1 = ApiKeyRepository::generate_credentials();
        let key2 = ApiKeyRepository::generate_credentials();

        assert_ne!(key1, key2);
    }
}
