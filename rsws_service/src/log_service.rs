//! 日志服务

use rsws_common::error::RswsError;
use serde_json::Value;
use sqlx::PgPool;

/// 日志服务
pub struct LogService {
    pool: PgPool,
}

impl LogService {
    /// 创建日志服务实例
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 记录系统日志
    pub async fn log_system(&self, level: &str, message: &str, data: Option<Value>) -> Result<(), RswsError> {
        sqlx::query(
            "INSERT INTO system_logs (level, message, data, created_at) VALUES ($1, $2, $3, NOW())",
        )
        .bind(level)
        .bind(message)
        .bind(data)
        .execute(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to log: {}", e)))?;

        Ok(())
    }

    /// 记录错误日志
    pub async fn log_error(&self, error: &str, stack_trace: Option<&str>) -> Result<(), RswsError> {
        sqlx::query(
            "INSERT INTO error_logs (error_message, stack_trace, created_at) VALUES ($1, $2, NOW())",
        )
        .bind(error)
        .bind(stack_trace)
        .execute(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to log error: {}", e)))?;

        Ok(())
    }
}
