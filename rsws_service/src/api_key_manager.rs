//! 统一 API Key 管理器（Pure Redis）
//!
//! Admin 和 User 的 API Key 使用相同的 Cregis 签名方案，
//! 仅通过 Redis key 前缀区分。

use rsws_common::error::RswsError;
use rsws_common::utils::generate_api_key;
use rsws_db::RedisService;
use rsws_model::api_key::{ApiKey, ApiKeyResponse, CreateApiKeyRequest};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Redis 缓存的 API Key 会话
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedApiKey {
    pub owner_id: i64,      // admin_id 或 user_id
    pub key_id: i64,
    pub api_key: String,
    pub role: String,        // "admin" 或 "user"
    pub permissions: Vec<String>,
    pub rate_limit: Option<i32>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// 统一 API Key 管理器
pub struct ApiKeyManager {
    redis: Arc<RedisService>,
    key_prefix: String,     // "admin_apikey" 或 "user_apikey"
}

impl ApiKeyManager {
    pub fn new(redis: Arc<RedisService>, key_prefix: &str) -> Self {
        Self {
            redis,
            key_prefix: key_prefix.to_string(),
        }
    }

    /// 创建 Admin API Key 管理器
    pub fn for_admin(redis: Arc<RedisService>) -> Self {
        Self::new(redis, "admin_apikey")
    }

    /// 创建 User API Key 管理器
    pub fn for_user(redis: Arc<RedisService>) -> Self {
        Self::new(redis, "user_apikey")
    }

    fn redis_key(&self, owner_id: i64) -> String {
        format!("{}:{}", self.key_prefix, owner_id)
    }

    const DEFAULT_SESSION_TTL: u64 = 7 * 24 * 3600;

    async fn session_ttl(&self) -> u64 {
        self.redis
            .get("config:api_key.session_expire_days")
            .await
            .ok()
            .flatten()
            .and_then(|v| v.parse::<u64>().ok())
            .map(|days| days * 24 * 3600)
            .unwrap_or(Self::DEFAULT_SESSION_TTL)
    }

    /// 创建 API Key
    pub async fn create(
        &self,
        owner_id: i64,
        request: CreateApiKeyRequest,
    ) -> Result<ApiKeyResponse, RswsError> {
        let api_key = generate_api_key();
        let expires_at = request
            .expires_in_days
            .map(|days| chrono::Utc::now() + chrono::Duration::days(days as i64));

        let key_id = chrono::Utc::now().timestamp_millis();
        let cached = CachedApiKey {
            owner_id,
            key_id,
            api_key: api_key.clone(),
            role: String::new(),
            permissions: request.permissions.clone(),
            rate_limit: request.rate_limit,
            expires_at,
        };

        let ttl = self.session_ttl().await;
        self.redis
            .set_json(&self.redis_key(owner_id), &cached, ttl)
            .await?;

        Ok(ApiKeyResponse {
            id: key_id,
            name: request.name,
            api_key,
            permissions: request.permissions,
            rate_limit: cached.rate_limit.unwrap_or(1000),
            last_used_at: None,
            expires_at,
            is_active: true,
            created_at: chrono::Utc::now(),
        })
    }

    /// 获取 API Key（从 Redis）
    pub async fn get(&self, owner_id: i64) -> Result<Option<ApiKey>, RswsError> {
        let cached = self.redis
            .get_json::<CachedApiKey>(&self.redis_key(owner_id))
            .await?;

        match cached {
            Some(cached) => {
                // 检查过期
                if let Some(expires) = cached.expires_at {
                    if expires < chrono::Utc::now() {
                        let _ = self.redis.del(&self.redis_key(owner_id)).await;
                        return Ok(None);
                    }
                }
                Ok(Some(ApiKey {
                    id: cached.key_id,
                    user_id: cached.owner_id,
                    api_key: cached.api_key,
                    name: String::new(),
                    permissions: serde_json::to_value(&cached.permissions).unwrap_or_default(),
                    rate_limit: cached.rate_limit.unwrap_or(1000),
                    last_used_at: None,
                    expires_at: cached.expires_at,
                    is_active: true,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                }))
            }
            None => Ok(None),
        }
    }

    /// 删除 API Key（从 Redis）
    pub async fn delete(&self, owner_id: i64) -> Result<bool, RswsError> {
        self.redis.del(&self.redis_key(owner_id)).await?;
        Ok(true)
    }

    /// 使失效（等同于 delete）
    pub async fn invalidate(&self, owner_id: i64) -> Result<(), RswsError> {
        let _ = self.redis.del(&self.redis_key(owner_id)).await;
        Ok(())
    }

    /// 密码变更时使失效
    pub async fn on_password_change(&self, owner_id: i64) -> Result<(), RswsError> {
        self.invalidate(owner_id).await
    }

    /// 切换状态（Redis 方案：不活跃 = 删除）
    pub async fn toggle_status(&self, owner_id: i64, is_active: bool) -> Result<(), RswsError> {
        if !is_active {
            self.invalidate(owner_id).await?;
        }
        Ok(())
    }

    /// 验证签名（Cregis 方案）
    ///
    /// 返回 Ok(Some(ApiKey)) 验签通过，Ok(None) 验签失败
    pub async fn validate_signature(
        &self,
        owner_id: i64,
        params: &HashMap<String, String>,
        sign: &str,
    ) -> Result<Option<ApiKey>, RswsError> {
        let api_key_record = self.get(owner_id).await?;
        match api_key_record {
            Some(record) => {
                let computed_sign = rsws_common::signature::compute_cregis_signature(params, &record.api_key);
                if computed_sign == sign {
                    Ok(Some(record))
                } else {
                    tracing::warn!(
                        "Signature mismatch for owner_id: {}. Expected: {}, Got: {}",
                        owner_id, computed_sign, sign
                    );
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use rsws_common::utils::generate_api_key;

    #[test]
    fn test_generate_api_key() {
        let api_key = generate_api_key();
        assert!(api_key.starts_with("ak_"));
        assert!(api_key.len() > 10);
    }

    #[test]
    fn test_generate_api_key_unique() {
        let key1 = generate_api_key();
        let key2 = generate_api_key();
        assert_ne!(key1, key2);
    }
}
