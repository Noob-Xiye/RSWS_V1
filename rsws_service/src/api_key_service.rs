//! API Key 服务
//!
//! 新设计（Cregis 方案）：
//! - api_key 作为签名密钥，前端持有用于签名
//! - 签名验证：通过 user_id 查找 api_key，重算签名对比
//! - 不再需要 api_secret

use rsws_common::error::RswsError;
use rsws_db::{ApiKeyRepository, RedisService};
use rsws_model::api_key::{ApiKey, ApiKeyResponse, CreateApiKeyRequest};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Redis 中缓存的 API Key 会话信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedApiKey {
    pub user_id: i64,
    pub api_key_id: i64,
    /// 签名密钥，用于签名验证
    pub api_key: String,
    pub permissions: Vec<String>,
    pub rate_limit: i32,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// API Key 服务
pub struct ApiKeyService {
    repository: Arc<ApiKeyRepository>,
    redis: Option<RedisService>,
}

impl ApiKeyService {
    /// 创建 API Key 服务实例（无 Redis）
    pub fn new(repository: Arc<ApiKeyRepository>) -> Self {
        Self {
            repository,
            redis: None,
        }
    }

    /// 创建 API Key 服务实例（带 Redis 缓存）
    pub fn with_redis(repository: Arc<ApiKeyRepository>, redis: RedisService) -> Self {
        Self {
            repository,
            redis: Some(redis),
        }
    }

    /// Redis key 格式（按 user_id 存储，因为按 user_id 查找）
    fn redis_key(user_id: i64) -> String {
        format!("apikey:user:{}", user_id)
    }

    /// 默认会话 TTL（秒）= 7 天
    const DEFAULT_SESSION_TTL: u64 = 7 * 24 * 3600;

    /// 获取会话 TTL
    async fn session_ttl(&self) -> u64 {
        if let Some(ref redis) = self.redis {
            redis
                .get("config:api_key.session_expire_days")
                .await
                .ok()
                .flatten()
                .and_then(|v| v.parse::<u64>().ok())
                .map(|days| days * 24 * 3600)
                .unwrap_or(Self::DEFAULT_SESSION_TTL)
        } else {
            Self::DEFAULT_SESSION_TTL
        }
    }

    /// 验证 API Key（按 user_id 验证，用于签名认证）
    pub async fn validate_by_user_id(&self, user_id: i64) -> Result<Option<ApiKey>, RswsError> {
        // 1) 先查 Redis
        if let Some(ref redis) = self.redis {
            if let Some(cached) = redis
                .get_json::<CachedApiKey>(&Self::redis_key(user_id))
                .await?
            {
                // 检查是否过期
                if let Some(expires) = cached.expires_at {
                    if expires < chrono::Utc::now() {
                        let _ = redis.del(&Self::redis_key(user_id)).await;
                        return Ok(None);
                    }
                }
                // 从缓存重建 ApiKey
                return Ok(Some(ApiKey {
                    id: cached.api_key_id,
                    user_id: cached.user_id,
                    api_key: cached.api_key,
                    name: String::new(),
                    permissions: serde_json::to_value(&cached.permissions).unwrap_or_default(),
                    rate_limit: cached.rate_limit,
                    last_used_at: None,
                    expires_at: cached.expires_at,
                    is_active: true,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                }));
            }
        }

        // 2) Redis miss → 查 DB
        let result = self.repository.get_active_key_by_user_id(user_id).await?;

        // 3) DB 命中 → 写入 Redis
        if let Some(ref key_record) = result {
            if let Some(ref redis) = self.redis {
                let permissions: Vec<String> =
                    serde_json::from_value(key_record.permissions.clone()).unwrap_or_default();
                let cached = CachedApiKey {
                    user_id: key_record.user_id,
                    api_key_id: key_record.id,
                    api_key: key_record.api_key.clone(),
                    permissions,
                    rate_limit: key_record.rate_limit,
                    expires_at: key_record.expires_at,
                };
                let ttl = self.session_ttl().await;
                let _ = redis
                    .set_json(&Self::redis_key(user_id), &cached, ttl)
                    .await;
            }
        }

        Ok(result)
    }

    /// 创建 API Key（Cregis 方案：只生成 api_key，返回给前端）
    pub async fn create(
        &self,
        user_id: i64,
        request: CreateApiKeyRequest,
    ) -> Result<ApiKeyResponse, RswsError> {
        let api_key_record = self.repository.create(user_id, &request).await?;

        // 创建后写入 Redis 缓存
        if let Some(ref redis) = self.redis {
            let permissions: Vec<String> =
                serde_json::from_value(api_key_record.permissions.clone()).unwrap_or_default();
            let cached = CachedApiKey {
                user_id: api_key_record.user_id,
                api_key_id: api_key_record.id,
                api_key: api_key_record.api_key.clone(),
                permissions,
                rate_limit: api_key_record.rate_limit,
                expires_at: api_key_record.expires_at,
            };
            let ttl = self.session_ttl().await;
            let _ = redis
                .set_json(&Self::redis_key(user_id), &cached, ttl)
                .await;
        }

        Ok(ApiKeyResponse {
            id: api_key_record.id,
            name: api_key_record.name,
            api_key: api_key_record.api_key,
            permissions: request.permissions,
            rate_limit: api_key_record.rate_limit,
            last_used_at: api_key_record.last_used_at,
            expires_at: api_key_record.expires_at,
            is_active: api_key_record.is_active,
            created_at: api_key_record.created_at,
        })
    }

    /// 获取用户的 API Keys
    pub async fn get_user_keys(&self, user_id: i64) -> Result<Vec<ApiKey>, RswsError> {
        self.repository.get_user_api_keys(user_id).await
    }

    /// 删除 API Key
    pub async fn delete(&self, api_key_id: i64, user_id: i64) -> Result<bool, RswsError> {
        let deleted = self.repository.delete(api_key_id, user_id).await?;

        if deleted {
            if let Some(ref redis) = self.redis {
                let _ = redis.del(&Self::redis_key(user_id)).await;
            }
        }

        Ok(deleted)
    }

    /// 更新最后使用时间
    pub async fn update_last_used(&self, api_key_id: i64) -> Result<(), RswsError> {
        self.repository.update_last_used(api_key_id).await
    }

    /// 禁用 API Key（后台强制下线）
    pub async fn deactivate_key(&self, key_id: i64) -> Result<bool, RswsError> {
        self.repository.deactivate_by_id(key_id).await
    }

    /// 密码变更后清除用户所有 API Key 缓存（强制重新登录）
    pub async fn on_password_change(&self, user_id: i64) -> Result<(), RswsError> {
        // 1) DB: 禁用用户所有 API Key
        self.repository.deactivate_by_user(user_id).await?;

        // 2) Redis: 清除该用户 API Key 缓存
        if let Some(ref redis) = self.redis {
            let _ = redis.del(&Self::redis_key(user_id)).await;
        }

        Ok(())
    }

    /// 验证签名认证（Cregis 方案：通过 user_id 查找 api_key 验签）
    ///
    /// 流程：
    /// 1. 用 user_id 查 DB/Redis 获取 api_key
    /// 2. 用同样算法重算签名（api_key 拼在参数前面 → MD5）
    /// 3. 对比签名，一致则通过
    pub async fn validate_signature_by_user_id(
        &self,
        user_id: i64,
        params: &HashMap<String, String>,
        sign: &str,
    ) -> Result<Option<ApiKey>, RswsError> {
        // 1) 获取 api_key（通过 validate_by_user_id 走缓存+DB）
        let api_key_record = self.validate_by_user_id(user_id).await?;
        let api_key_record = match api_key_record {
            Some(r) => r,
            None => return Ok(None),
        };

        let api_key = &api_key_record.api_key;

        // 2) 重算签名（Cregis: api_key 拼在排序参数前面）
        let computed_sign = rsws_common::signature::compute_cregis_signature(params, api_key);

        // 3) 对比签名
        if computed_sign == sign {
            Ok(Some(api_key_record))
        } else {
            tracing::warn!(
                "Signature mismatch for user_id: {}. Expected: {}, Got: {}",
                user_id,
                computed_sign,
                sign
            );
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rsws_common::signature::compute_cregis_signature as compute_signature;

    #[test]
    fn test_compute_signature_basic() {
        let mut params = HashMap::new();
        params.insert("user_id".to_string(), "12345".to_string());
        params.insert("timestamp".to_string(), "1715400000000".to_string());
        params.insert("nonce".to_string(), "abc123".to_string());
        params.insert("sign".to_string(), "should_be_ignored".to_string());

        let api_key = "ak_test_api_key_12345";
        let signature = compute_signature(&params, api_key);

        assert!(!signature.is_empty());
        assert_eq!(signature.len(), 32);
        assert!(signature.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_signature_excludes_sign_param() {
        let mut params_with_sign = HashMap::new();
        params_with_sign.insert("user_id".to_string(), "test".to_string());
        params_with_sign.insert("sign".to_string(), "value1".to_string());

        let mut params_without_sign = HashMap::new();
        params_without_sign.insert("user_id".to_string(), "test".to_string());

        let api_key = "secret";
        let sig_with_sign = rsws_common::signature::compute_cregis_signature(&params_with_sign, api_key);
        let sig_without_sign = rsws_common::signature::compute_cregis_signature(&params_without_sign, api_key);

        // 两种情况的签名应该相同
        assert_eq!(sig_with_sign, sig_without_sign);
    }

    #[test]
    fn test_signature_key_ordering() {
        let mut params1 = HashMap::new();
        params1.insert("user_id".to_string(), "key".to_string());
        params1.insert("timestamp".to_string(), "123".to_string());

        let mut params2 = HashMap::new();
        params2.insert("timestamp".to_string(), "123".to_string());
        params2.insert("user_id".to_string(), "key".to_string());

        let api_key = "secret";
        let sig1 = rsws_common::signature::compute_cregis_signature(&params1, api_key);
        let sig2 = rsws_common::signature::compute_cregis_signature(&params2, api_key);

        // 签名应该相同（因为会排序）
        assert_eq!(sig1, sig2);
    }

    #[test]
    fn test_signature_different_inputs_different_output() {
        let mut params1 = HashMap::new();
        params1.insert("user_id".to_string(), "key1".to_string());

        let mut params2 = HashMap::new();
        params2.insert("user_id".to_string(), "key2".to_string());

        let sig1 = compute_signature(&params1, "secret");
        let sig2 = compute_signature(&params2, "secret");

        assert_ne!(sig1, sig2);
    }

    #[test]
    fn test_signature_different_keys_different_output() {
        let mut params = HashMap::new();
        params.insert("user_id".to_string(), "key".to_string());

        let sig1 = compute_signature(&params, "secret1");
        let sig2 = compute_signature(&params, "secret2");

        assert_ne!(sig1, sig2);
    }

    #[test]
    fn test_signature_matches_cregis_algorithm() {
        // Cregis 算法：
        // 1. 排除 sign，按 key ASCII 排序
        // 2. 拼接 key + value
        // 3. api_key 拼在最前面
        // 4. MD5 小写 hex

        let mut params = HashMap::new();
        params.insert("user_id".to_string(), "12345".to_string());
        params.insert("timestamp".to_string(), "1234567890".to_string());
        params.insert("nonce".to_string(), "nonce123".to_string());

        let api_key = "my_secret_api_key";

        // ASCII 排序后: nonce, timestamp, user_id
        // 拼接: api_key + "nonce" + "nonce123" + "timestamp" + "1234567890" + "user_id" + "12345"
        let expected_concat = format!(
            "{}{}{}{}{}{}{}",
            api_key, "nonce", "nonce123", "timestamp", "1234567890", "user_id", "12345"
        );

        let expected_md5 = format!("{:x}", md5::compute(expected_concat.as_bytes()));
        let actual_sig = compute_signature(&params, api_key);

        assert_eq!(actual_sig, expected_md5);
    }

    #[test]
    fn test_signature_matches_frontend() {
        // 前端 TypeScript 逻辑:
        // const keys = Object.keys(params).sort()
        // const paramStr = keys.map(k => k + params[k]).join('')
        // const signStr = apiKey + paramStr
        // CryptoJS.MD5(signStr).toString()

        let mut params = HashMap::new();
        params.insert("api_key".to_string(), "test".to_string());
        params.insert("timestamp".to_string(), "1234567890".to_string());
        params.insert("nonce".to_string(), "nonce123".to_string());

        let api_key = "my_secret_key";

        // 按 ASCII 排序: api_key, nonce, timestamp
        // 拼接: api_key + "api_key" + "test" + "nonce" + "nonce123" + "timestamp" + "1234567890"
        let expected_concat = format!(
            "{}{}{}{}{}{}{}",
            api_key, "api_key", "test", "nonce", "nonce123", "timestamp", "1234567890"
        );

        let expected_md5 = format!("{:x}", md5::compute(expected_concat.as_bytes()));
        let actual_sig = compute_signature(&params, api_key);

        assert_eq!(actual_sig, expected_md5);
    }
}
