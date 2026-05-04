//! 佣金服务

use rsws_common::error::RswsError;
use sqlx::PgPool;
use tracing::info;

/// 佣金服务
pub struct CommissionService {
    pool: PgPool,
}

impl CommissionService {
    /// 创建佣金服务实例
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 计算佣金
    pub async fn calculate(&self, order_id: i64, amount: i64, rate: i64) -> Result<i64, RswsError> {
        let commission = amount * rate / 10000; // rate 是万分比
        info!("Calculated commission for order {}: {}", order_id, commission);
        Ok(commission)
    }

    /// 结算佣金
    pub async fn settle(&self, order_id: i64) -> Result<(), RswsError> {
        info!("Settling commission for order: {}", order_id);

        sqlx::query(
            "UPDATE commission_records SET status = 'settled', settled_at = NOW() WHERE order_id = $1 AND status = 'pending'",
        )
        .bind(order_id)
        .execute(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to settle commission: {}", e)))?;

        Ok(())
    }
}
