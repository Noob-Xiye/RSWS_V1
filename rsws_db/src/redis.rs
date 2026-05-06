//! Redis 缓存服务

use deadpool_redis::{Config as RedisConfig, Pool, Runtime};
use redis::AsyncCommands;
use rsws_common::error::RswsError;
use serde::{de::DeserializeOwned, Serialize};
use tracing::{debug, error};

/// Redis 服务
#[derive(Clone)]
pub struct RedisService {
    pool: Pool,
}

impl RedisService {
    /// 创建 Redis 服务
    pub fn new(url: &str) -> Result<Self, RswsError> {
        let cfg = RedisConfig::from_url(url);
        let pool = cfg
            .create_pool(Some(Runtime::Tokio1))
            .map_err(|e| {
                error!("Failed to create Redis pool: {}", e);
                RswsError::internal("Failed to create Redis pool")
            })?;

        Ok(Self { pool })
    }

    /// 从连接池创建
    pub fn from_pool(pool: Pool) -> Self {
        Self { pool }
    }

    /// 获取连接
    async fn get_connection(&self) -> Result<deadpool_redis::Connection, RswsError> {
        self.pool.get().await.map_err(|e| {
            error!("Failed to get Redis connection: {}", e);
            RswsError::internal("Failed to get Redis connection")
        })
    }

    /// 设置键值（带过期时间，秒）
    pub async fn set_ex(&self, key: &str, value: &str, ttl_secs: u64) -> Result<(), RswsError> {
        let mut conn = self.get_connection().await?;
        conn.set_ex::<_, _, ()>(key, value, ttl_secs).await.map_err(|e| {
            error!("Failed to set Redis key: {}", e);
            RswsError::internal("Failed to set Redis key")
        })?;
        debug!("Redis SET {} (TTL: {}s)", key, ttl_secs);
        Ok(())
    }

    /// 获取键值
    pub async fn get(&self, key: &str) -> Result<Option<String>, RswsError> {
        let mut conn = self.get_connection().await?;
        let result: Option<String> = conn.get(key).await.map_err(|e| {
            error!("Failed to get Redis key: {}", e);
            RswsError::internal("Failed to get Redis key")
        })?;
        debug!("Redis GET {} -> {:?}", key, result.is_some());
        Ok(result)
    }

    /// 删除键
    pub async fn del(&self, key: &str) -> Result<(), RswsError> {
        let mut conn = self.get_connection().await?;
        conn.del::<_, ()>(key).await.map_err(|e| {
            error!("Failed to delete Redis key: {}", e);
            RswsError::internal("Failed to delete Redis key")
        })?;
        debug!("Redis DEL {}", key);
        Ok(())
    }

    /// 检查键是否存在
    pub async fn exists(&self, key: &str) -> Result<bool, RswsError> {
        let mut conn = self.get_connection().await?;
        let exists: bool = conn.exists(key).await.map_err(|e| {
            error!("Failed to check Redis key existence: {}", e);
            RswsError::internal("Failed to check Redis key existence")
        })?;
        Ok(exists)
    }

    /// 设置 JSON 对象
    pub async fn set_json<T: Serialize>(
        &self,
        key: &str,
        value: &T,
        ttl_secs: u64,
    ) -> Result<(), RswsError> {
        let json = serde_json::to_string(value).map_err(|e| {
            error!("Failed to serialize JSON: {}", e);
            RswsError::internal("Failed to serialize JSON")
        })?;
        self.set_ex(key, &json, ttl_secs).await
    }

    /// 获取 JSON 对象
    pub async fn get_json<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, RswsError> {
        let json = self.get(key).await?;
        match json {
            Some(s) => {
                let value: T = serde_json::from_str(&s).map_err(|e| {
                    error!("Failed to deserialize JSON: {}", e);
                    RswsError::internal("Failed to deserialize JSON")
                })?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    /// 原子递增
    /// 
    /// # Arguments
    /// * `key` - Redis key
    /// * `delta` - 增量值
    pub async fn incr(&self, key: &str, delta: i64) -> Result<i64, RswsError> {
        let mut conn = self.get_connection().await?;
        let result: i64 = conn.incr(key, delta).await.map_err(|e| {
            error!("Failed to increment Redis key: {}", e);
            RswsError::internal("Failed to increment Redis key")
        })?;
        Ok(result)
    }

    /// 设置键过期时间（秒）
    /// 
    /// # Returns
    /// * `true` - 设置成功
    /// * `false` - key 不存在
    pub async fn expire(&self, key: &str, ttl_secs: i64) -> Result<bool, RswsError> {
        let mut conn = self.get_connection().await?;
        let result: bool = conn.expire(key, ttl_secs).await.map_err(|e| {
            error!("Failed to set Redis key expiration: {}", e);
            RswsError::internal("Failed to set Redis key expiration")
        })?;
        Ok(result)
    }
}

// ==================== 验证码缓存 ====================

/// 验证码缓存键
pub struct VerificationCodeCache<'a> {
    pub email: &'a str,
    pub code_type: &'a str, // "register" | "login" | "reset_password"
}

impl<'a> VerificationCodeCache<'a> {
    /// 生成缓存键
    pub fn key(&self) -> String {
        format!("verify:{}:{}", self.code_type, self.email)
    }

    /// 生成尝试次数键
    pub fn attempts_key(&self) -> String {
        format!("verify:{}:{}:attempts", self.code_type, self.email)
    }
}

impl RedisService {
    /// 存储验证码（5分钟有效期）
    pub async fn set_verification_code(
        &self,
        email: &str,
        code_type: &str,
        code: &str,
    ) -> Result<(), RswsError> {
        let cache = VerificationCodeCache { email, code_type };
        // 验证码 5 分钟有效
        self.set_ex(&cache.key(), code, 300).await?;
        // 重置尝试次数
        self.set_ex(&cache.attempts_key(), "0", 300).await?;
        Ok(())
    }

    /// 验证验证码
    /// 返回 (是否正确, 剩余尝试次数)
    pub async fn verify_code(
        &self,
        email: &str,
        code_type: &str,
        code: &str,
    ) -> Result<(bool, i32), RswsError> {
        let cache = VerificationCodeCache { email, code_type };

        // 检查尝试次数
        let attempts_key = cache.attempts_key();
        let attempts: i64 = self
            .get(&attempts_key)
            .await?
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        if attempts >= 5 {
            // 超过最大尝试次数
            return Ok((false, 0));
        }

        // 增加尝试次数
        let new_attempts = self.incr(&attempts_key, 1).await?;

        // 获取存储的验证码
        let stored = self.get(&cache.key()).await?;

        match stored {
            Some(stored_code) if stored_code == code => {
                // 验证成功，删除验证码
                self.del(&cache.key()).await?;
                self.del(&attempts_key).await?;
                Ok((true, 5 - new_attempts as i32))
            }
            Some(_) => {
                // 验证失败
                Ok((false, 5 - new_attempts as i32))
            }
            None => {
                // 验证码不存在或已过期
                Ok((false, 0))
            }
        }
    }

    /// 检查验证码是否存在
    pub async fn has_verification_code(
        &self,
        email: &str,
        code_type: &str,
    ) -> Result<bool, RswsError> {
        let cache = VerificationCodeCache { email, code_type };
        self.exists(&cache.key()).await
    }
}

// ==================== 用户缓存 ====================

impl RedisService {
    /// 用户信息缓存键
    fn user_cache_key(user_id: i64) -> String {
        format!("user:{}", user_id)
    }

    /// 缓存用户信息（30分钟）
    pub async fn cache_user<T: Serialize>(&self, user_id: i64, user: &T) -> Result<(), RswsError> {
        let key = Self::user_cache_key(user_id);
        self.set_json(&key, user, 1800).await
    }

    /// 获取缓存的用户信息
    pub async fn get_cached_user<T: DeserializeOwned>(&self, user_id: i64) -> Result<Option<T>, RswsError> {
        let key = Self::user_cache_key(user_id);
        self.get_json(&key).await
    }

    /// 清除用户缓存
    pub async fn clear_user_cache(&self, user_id: i64) -> Result<(), RswsError> {
        let key = Self::user_cache_key(user_id);
        self.del(&key).await
    }
}

// ==================== 单元测试 ====================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verification_code_key() {
        let cache = VerificationCodeCache {
            email: "test@example.com",
            code_type: "login",
        };
        assert_eq!(cache.key(), "verify:login:test@example.com");
        assert_eq!(cache.attempts_key(), "verify:login:test@example.com:attempts");
    }

    #[test]
    fn test_user_cache_key() {
        assert_eq!(RedisService::user_cache_key(123), "user:123");
    }
}
