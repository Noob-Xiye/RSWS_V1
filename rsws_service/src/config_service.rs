//! 配置服务
//!
//! 提供系统配置的读取和写入

use rsws_common::error::RswsError;
use rsws_db::RedisService;
use sqlx::PgPool;

/// 配置服务
pub struct ConfigService {
    pool: PgPool,
    redis: RedisService,
}

impl ConfigService {
    /// 创建配置服务实例
    pub fn new(pool: PgPool, redis: RedisService) -> Self {
        Self { pool, redis }
    }

    /// 获取数据库连接池（供 middleware 直接查询）
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// 获取 Redis 客户端（供 rate_limit 使用）
    pub fn redis_client(&self) -> &RedisService {
        &self.redis
    }

    /// 获取配置值
    pub async fn get(&self, key: &str) -> Result<Option<String>, RswsError> {
        let result: Option<(String,)> = sqlx::query_as(
            "SELECT config_value FROM system_configs WHERE config_key = $1",
        )
        .bind(key)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to get config: {}", e)))?;

        Ok(result.map(|r| r.0))
    }

    /// 设置配置
    pub async fn set(&self, key: &str, value: &str) -> Result<(), RswsError> {
        sqlx::query(
            r#"
            INSERT INTO system_configs (config_key, config_value, config_type, is_encrypted, created_at, updated_at)
            VALUES ($1, $2, 'string', false, NOW(), NOW())
            ON CONFLICT (config_key) DO UPDATE SET config_value = $2, updated_at = NOW()
            "#,
        )
        .bind(key)
        .bind(value)
        .execute(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to set config: {}", e)))?;

        Ok(())
    }

    /// 获取整型配置值
    pub async fn get_int(&self, key: &str) -> Result<Option<i64>, RswsError> {
        match self.get(key).await? {
            Some(v) => Ok(Some(v.parse().map_err(|_| {
                RswsError::internal(format!("Config '{}' is not a valid integer", key))
            })?)),
            None => Ok(None),
        }
    }

    /// 获取布尔配置值
    pub async fn get_bool(&self, key: &str) -> Result<Option<bool>, RswsError> {
        match self.get(key).await? {
            Some(v) => Ok(Some(v.parse().unwrap_or(false))),
            None => Ok(None),
        }
    }
}
