//! 配置服务

use rsws_common::error::RswsError;
use sqlx::PgPool;

/// 配置服务
pub struct ConfigService {
    pool: PgPool,
}

impl ConfigService {
    /// 创建配置服务实例
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 获取配置
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
}
