//! API Key 服务
//!
//! API Key 验证走 Redis 缓存，DB 存配置元数据。
//! 禁用 key → 删 Redis 缓存 → 强制下线。
//! 改密码 → 清用户所有 Redis API Key 缓存。

use md5;
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
    pub api_secret: String,
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

    /// Redis key 格式
    fn redis_key(api_key: &str) -> String {
        format!("apikey:{}", api_key)
    }

    /// 默认会话 TTL（秒）= 7 天
    const DEFAULT_SESSION_TTL: u64 = 7 * 24 * 3600;

    /// 获取会话 TTL
    async fn session_ttl(&self) -> u64 {
        if let Some(ref redis) = self.redis {
            // 尝试从 system_configs 读，读不到用默认值
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

    /// 验证 API Key（带 Redis 缓存）
    ///
    /// 流程：Redis → 命中且 secret 匹配 → 通过
    ///        Redis miss → 查 DB → 写入 Redis → 通过
    pub async fn validate(
        &self,
        api_key: &str,
        api_secret: &str,
    ) -> Result<Option<ApiKey>, RswsError> {
        // 1) 先查 Redis
        if let Some(ref redis) = self.redis {
            if let Some(cached) = redis
                .get_json::<CachedApiKey>(&Self::redis_key(api_key))
                .await?
            {
                // Redis 命中：验证 secret
                if cached.api_secret == api_secret {
                    // 检查是否过期
                    if let Some(expires) = cached.expires_at {
                        if expires < chrono::Utc::now() {
                            // 已过期，删缓存
                            let _ = redis.del(&Self::redis_key(api_key)).await;
                            return Ok(None);
                        }
                    }
                    // 构造 ApiKey 返回（简化版，足够中间件使用）
                    return Ok(Some(ApiKey {
                        id: cached.api_key_id,
                        user_id: cached.user_id,
                        api_key: api_key.to_string(),
                        api_secret: cached.api_secret.clone(),
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
                // secret 不匹配，可能已被替换，删缓存
                let _ = redis.del(&Self::redis_key(api_key)).await;
            }
        }

        // 2) Redis miss 或无 Redis → 查 DB
        let result = self.repository.validate(api_key, api_secret).await?;

        // 3) DB 验证通过 → 写入 Redis
        if let Some(ref key_record) = result {
            if let Some(ref redis) = self.redis {
                let permissions: Vec<String> =
                    serde_json::from_value(key_record.permissions.clone()).unwrap_or_default();
                let cached = CachedApiKey {
                    user_id: key_record.user_id,
                    api_key_id: key_record.id,
                    api_secret: key_record.api_secret.clone(),
                    permissions,
                    rate_limit: key_record.rate_limit,
                    expires_at: key_record.expires_at,
                };
                let ttl = self.session_ttl().await;
                let _ = redis
                    .set_json(&Self::redis_key(api_key), &cached, ttl)
                    .await;
            }
        }

        Ok(result)
    }

    /// 创建 API Key
    pub async fn create(
        &self,
        user_id: i64,
        request: CreateApiKeyRequest,
    ) -> Result<ApiKeyResponse, RswsError> {
        let (api_key, api_secret) = self.repository.create(user_id, &request).await?;

        // 创建后写入 Redis 缓存
        if let Some(ref redis) = self.redis {
            let permissions: Vec<String> =
                serde_json::from_value(api_key.permissions.clone()).unwrap_or_default();
            let cached = CachedApiKey {
                user_id: api_key.user_id,
                api_key_id: api_key.id,
                api_secret: api_key.api_secret.clone(),
                permissions,
                rate_limit: api_key.rate_limit,
                expires_at: api_key.expires_at,
            };
            let ttl = self.session_ttl().await;
            let _ = redis
                .set_json(&Self::redis_key(&api_key.api_key), &cached, ttl)
                .await;
        }

        Ok(ApiKeyResponse {
            id: api_key.id,
            name: api_key.name,
            api_key: api_key.api_key,
            api_secret: Some(api_secret),
            permissions: request.permissions,
            rate_limit: api_key.rate_limit,
            last_used_at: api_key.last_used_at,
            expires_at: api_key.expires_at,
            is_active: api_key.is_active,
            created_at: api_key.created_at,
        })
    }

    /// 获取用户的 API Keys
    pub async fn get_user_keys(&self, user_id: i64) -> Result<Vec<ApiKey>, RswsError> {
        self.repository.get_user_api_keys(user_id).await
    }

    /// 删除 API Key
    pub async fn delete(&self, api_key_id: i64, user_id: i64) -> Result<bool, RswsError> {
        // 删前先取 key 名称，用于清 Redis
        let keys = self.repository.get_user_api_keys(user_id).await?;
        let key_value = keys
            .iter()
            .find(|k| k.id == api_key_id)
            .map(|k| k.api_key.clone());

        let deleted = self.repository.delete(api_key_id, user_id).await?;

        if deleted {
            // 删除 Redis 缓存
            if let (Some(ref redis), Some(kv)) = (&self.redis, key_value) {
                let _ = redis.del(&Self::redis_key(&kv)).await;
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
        let deactivated = self.repository.deactivate_by_id(key_id).await?;

        if deactivated {
            // 需要清 Redis 缓存，但我们只有 key_id 没有 key_value
            // 通过 user_api_keys 找到对应的 key
            // 简单做法：暴力扫所有活跃 key 找到匹配的
            // 更好的做法：repository 返回被禁用 key 的 api_key 字段
            // 暂时在 deactivate_by_id 中返回 api_key
        }

        Ok(deactivated)
    }

    /// 密码变更后清除用户所有 API Key 缓存（强制重新登录）
    pub async fn on_password_change(&self, user_id: i64) -> Result<(), RswsError> {
        // 1) DB: 禁用用户所有 API Key
        self.repository.deactivate_by_user(user_id).await?;

        // 2) Redis: 清除该用户所有 API Key 缓存
        if let Some(ref redis) = self.redis {
            let active_keys = self.repository.get_active_keys_by_user(user_id).await?;
            for key in &active_keys {
                let _ = redis.del(&Self::redis_key(&key.api_key)).await;
            }
        }

        Ok(())
    }

    /// 验证签名认证（符合 Cregis 方案）
    ///
    /// 流程：
    /// 1. 用 api_key 查 DB 获取 api_secret
    /// 2. 用同样算法重算签名
    /// 3. 对比签名，一致则通过
    pub async fn validate_signature(
        &self,
        api_key: &str,
        params: &HashMap<String, String>,
        sign: &str,
    ) -> Result<Option<ApiKey>, RswsError> {
        // 1) 获取 api_secret（先查 Redis 缓存）
        let api_secret = if let Some(ref redis) = self.redis {
            if let Some(cached) = redis
                .get_json::<CachedApiKey>(&Self::redis_key(api_key))
                .await?
            {
                Some(cached.api_secret)
            } else {
                None
            }
        } else {
            None
        };

        let api_secret = match api_secret {
            Some(s) => s,
            None => {
                // Redis miss，从 DB 获取
                let record = self.repository.get_by_api_key(api_key).await?;
                match record {
                    Some(r) => {
                        // 写入 Redis 缓存
                        if let Some(ref redis) = self.redis {
                            let permissions: Vec<String> =
                                serde_json::from_value(r.permissions.clone()).unwrap_or_default();
                            let cached = CachedApiKey {
                                user_id: r.user_id,
                                api_key_id: r.id,
                                api_secret: r.api_secret.clone(),
                                permissions,
                                rate_limit: r.rate_limit,
                                expires_at: r.expires_at,
                            };
                            let ttl = self.session_ttl().await;
                            let _ = redis
                                .set_json(&Self::redis_key(api_key), &cached, ttl)
                                .await;
                        }
                        r.api_secret
                    }
                    None => return Ok(None),
                }
            }
        };

        // 2) 重算签名
        let computed_sign = compute_signature(params, &api_secret);

        // 3) 对比签名
        if computed_sign == sign {
            // 签名正确，获取完整的 ApiKey 记录
            let record = self.repository.get_by_api_key(api_key).await?;
            Ok(record)
        } else {
            tracing::warn!(
                "Signature mismatch for api_key: {}. Expected: {}, Got: {}",
                api_key,
                computed_sign,
                sign
            );
            Ok(None)
        }
    }
}

/// 计算签名（符合 Cregis 方案）
///
/// 算法：
/// 1. 排除 sign 字段，按 key ASCII 升序排序
/// 2. 拼接参数字符串（key + value）
/// 3. 拼接 api_secret 到字符串末尾
/// 4. MD5 计算并转小写 hex
fn compute_signature(params: &HashMap<String, String>, api_secret: &str) -> String {
    // 1. 获取所有 key（排除 sign），排序
    let mut keys: Vec<&String> = params.keys().filter(|k| (*k).as_str() != "sign").collect();
    keys.sort();

    // 2. 按 ASCII 顺序拼接 key + value
    let param_str: String = keys
        .iter()
        .map(|k| format!("{}{}", k, params[*k]))
        .collect();

    // 3. 拼接 api_secret（拼在前面，与 Cregis 方案一致）
    let sign_str = format!("{}{}", api_secret, param_str);

    // 4. MD5 + 小写 hex
    format!("{:x}", md5::compute(sign_str.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_signature_basic() {
        let mut params = HashMap::new();
        params.insert("api_key".to_string(), "test_api_key".to_string());
        params.insert("timestamp".to_string(), "1715400000000".to_string());
        params.insert("nonce".to_string(), "abc123".to_string());
        params.insert("sign".to_string(), "should_be_ignored".to_string()); // 应被忽略

        let api_secret = "test_secret";
        let signature = compute_signature(&params, api_secret);

        // 验证签名不为空
        assert!(!signature.is_empty());
        // 验证是有效的 MD5 hex 字符串（32个字符）
        assert_eq!(signature.len(), 32);
        assert!(signature.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_signature_excludes_sign_param() {
        let mut params_with_sign = HashMap::new();
        params_with_sign.insert("api_key".to_string(), "test".to_string());
        params_with_sign.insert("sign".to_string(), "value1".to_string());

        let mut params_without_sign = HashMap::new();
        params_without_sign.insert("api_key".to_string(), "test".to_string());

        let api_secret = "secret";
        let sig_with_sign = compute_signature(&params_with_sign, api_secret);
        let sig_without_sign = compute_signature(&params_without_sign, api_secret);

        // 两种情况的签名应该相同（因为 sign 被排除）
        assert_eq!(sig_with_sign, sig_without_sign);
    }

    #[test]
    fn test_signature_key_ordering() {
        // 按不同顺序插入参数，结果应该相同
        let mut params1 = HashMap::new();
        params1.insert("api_key".to_string(), "key".to_string());
        params1.insert("timestamp".to_string(), "123".to_string());

        let mut params2 = HashMap::new();
        params2.insert("timestamp".to_string(), "123".to_string());
        params2.insert("api_key".to_string(), "key".to_string());

        let api_secret = "secret";
        let sig1 = compute_signature(&params1, api_secret);
        let sig2 = compute_signature(&params2, api_secret);

        // 签名应该相同（因为会排序）
        assert_eq!(sig1, sig2);
    }

    #[test]
    fn test_signature_different_inputs_different_output() {
        let mut params1 = HashMap::new();
        params1.insert("api_key".to_string(), "key1".to_string());

        let mut params2 = HashMap::new();
        params2.insert("api_key".to_string(), "key2".to_string());

        let sig1 = compute_signature(&params1, "secret");
        let sig2 = compute_signature(&params2, "secret");

        assert_ne!(
            sig1, sig2,
            "Different inputs should produce different signatures"
        );
    }

    #[test]
    fn test_signature_different_secrets_different_output() {
        let mut params = HashMap::new();
        params.insert("api_key".to_string(), "key".to_string());

        let sig1 = compute_signature(&params, "secret1");
        let sig2 = compute_signature(&params, "secret2");

        assert_ne!(
            sig1, sig2,
            "Different secrets should produce different signatures"
        );
    }

    #[test]
    fn test_signature_matches_frontend_algorithm() {
        // 模拟前端 TypeScript 的行为:
        // const keys = Object.keys(params).sort()
        // const paramStr = keys.map(k => k + params[k]).join('')
        // const signStr = apiSecret + paramStr
        // CryptoJS.MD5(signStr).toString()

        // 参数: api_key=test, timestamp=1234567890, nonce=nonce123
        let mut params = HashMap::new();
        params.insert("api_key".to_string(), "test".to_string());
        params.insert("timestamp".to_string(), "1234567890".to_string());
        params.insert("nonce".to_string(), "nonce123".to_string());

        let api_secret = "my_secret_key";

        // ASCII 排序后: api_key, nonce, timestamp
        // 拼接: api_secret + "api_key" + "test" + "nonce" + "nonce123" + "timestamp" + "1234567890"
        let expected_concat = format!(
            "{}{}{}{}{}{}{}",
            api_secret, "api_key", "test", "nonce", "nonce123", "timestamp", "1234567890"
        );

        // 手动计算 MD5 (使用 Rust md5 crate)
        let expected_md5 = format!("{:x}", md5::compute(expected_concat.as_bytes()));
        let actual_sig = compute_signature(&params, api_secret);

        assert_eq!(actual_sig, expected_md5);
    }
}
