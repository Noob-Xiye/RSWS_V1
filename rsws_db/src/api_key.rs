//! API Key 仓储层

use chrono::Utc;
use rand::Rng;
use base64::{Engine as _, engine::general_purpose};
use rsws_model::api_key::{ApiKey, CreateApiKeyRequest};
use rsws_common::error::RswsError;
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

    /// 生成 API Key 和 Secret
    fn generate_credentials() -> (String, String) {
        let mut rng = rand::rng();

        // 生成 32 字节的随机数据作为 API Key
        let api_key_bytes: [u8; 32] = rng.random();
        let api_key = format!("ak_{}", general_purpose::URL_SAFE_NO_PAD.encode(&api_key_bytes));

        // 生成 64 字节的随机数据作为 API Secret
        let secret_bytes: [u8; 64] = rng.random();
        let api_secret = format!("sk_{}", general_purpose::URL_SAFE_NO_PAD.encode(&secret_bytes));

        (api_key, api_secret)
    }

    /// 创建 API Key
    pub async fn create(
        &self,
        user_id: i64,
        request: &CreateApiKeyRequest,
    ) -> Result<(ApiKey, String), RswsError> {
        let (api_key, api_secret) = Self::generate_credentials();

        // 计算过期时间
        let expires_at = request.expires_in_days.map(|days| {
            Utc::now() + chrono::Duration::days(days as i64)
        });

        let permissions_json = serde_json::to_value(&request.permissions)
            .map_err(|e| RswsError::internal(format!("Failed to serialize permissions: {}", e)))?;

        let api_key_record = sqlx::query_as::<_, ApiKey>(
            r#"
            INSERT INTO api_keys (user_id, api_key, api_secret, name, permissions, rate_limit, expires_at, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, true, NOW(), NOW())
            RETURNING id, user_id, api_key, api_secret, name, permissions, rate_limit, last_used_at, expires_at, is_active, created_at, updated_at
            "#
        )
        .bind(user_id)
        .bind(&api_key)
        .bind(&api_secret)
        .bind(&request.name)
        .bind(&permissions_json)
        .bind(request.rate_limit.unwrap_or(1000))
        .bind(expires_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to create API key: {}", e)))?;

        Ok((api_key_record, api_secret))
    }

    /// 根据 API Key 获取记录
    pub async fn get_by_api_key(&self, api_key: &str) -> Result<Option<ApiKey>, RswsError> {
        let api_key_record = sqlx::query_as::<_, ApiKey>(
            "SELECT id, user_id, api_key, api_secret, name, permissions, rate_limit, last_used_at, expires_at, is_active, created_at, updated_at FROM api_keys WHERE api_key = $1 AND is_active = true"
        )
        .bind(api_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to get API key: {}", e)))?;

        Ok(api_key_record)
    }

    /// 验证 API Key 和 Secret
    pub async fn validate(&self, api_key: &str, api_secret: &str) -> Result<Option<ApiKey>, RswsError> {
        let api_key_record = sqlx::query_as::<_, ApiKey>(
            r#"
            SELECT id, user_id, api_key, api_secret, name, permissions, rate_limit, last_used_at, expires_at, is_active, created_at, updated_at
            FROM api_keys 
            WHERE api_key = $1 AND api_secret = $2 AND is_active = true
            AND (expires_at IS NULL OR expires_at > NOW())
            "#
        )
        .bind(api_key)
        .bind(api_secret)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to validate credentials: {}", e)))?;

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
            "SELECT id, user_id, api_key, api_secret, name, permissions, rate_limit, last_used_at, expires_at, is_active, created_at, updated_at FROM api_keys WHERE user_id = $1 ORDER BY created_at DESC"
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
}

// ==================== 单元测试 ====================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_credentials() {
        let (api_key, api_secret) = ApiKeyRepository::generate_credentials();

        assert!(api_key.starts_with("ak_"));
        assert!(api_secret.starts_with("sk_"));
        assert!(api_key.len() > 10);
        assert!(api_secret.len() > 10);
    }

    #[test]
    fn test_generate_credentials_unique() {
        let (key1, secret1) = ApiKeyRepository::generate_credentials();
        let (key2, secret2) = ApiKeyRepository::generate_credentials();

        assert_ne!(key1, key2);
        assert_ne!(secret1, secret2);
    }
}
