//! 佣金服务

use rsws_common::error::RswsError;
use sqlx::PgPool;
use tracing::info;

pub struct CommissionService {
    pool: PgPool,
}

impl CommissionService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 计算佣金金额。amount 是以分为单位的订单金额，rate 是万分比。
    pub fn calculate(&self, order_id: i64, amount: i64, rate: i64) -> i64 {
        let commission = amount * rate / 10000;
        info!(
            "Commission calc: order={} amount={} rate={} => {}",
            order_id, amount, rate, commission
        );
        commission
    }

    /// 结算并记录佣金。
    /// 从 orders 表取 referrer_id，从 resources 表取 commission_rate，
    /// 计算后写入 commission_records。若无推荐人或佣金率为 0，跳过。
    pub async fn settle_commission(&self, order_id: i64) -> Result<(), RswsError> {
        let row: Option<(i64, i64, Option<i64>)> = sqlx::query_as(
            r#"
            SELECT o.amount, r.commission_rate, o.referrer_id
            FROM orders o
            JOIN resources r ON r.id = o.resource_id
            WHERE o.id = $1
            "#,
        )
        .bind(order_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Query commission data: {}", e)))?;

        let (order_amount, commission_rate, referrer_id) = match row {
            Some(r) => r,
            None => {
                info!("Order {} not found for commission", order_id);
                return Ok(());
            }
        };

        let referrer_id = match referrer_id {
            Some(id) if id > 0 => id,
            _ => {
                info!("Order {} has no referrer, skipping", order_id);
                return Ok(());
            }
        };

        if commission_rate <= 0 {
            info!(
                "Order {} commission_rate={}, skipping",
                order_id, commission_rate
            );
            return Ok(());
        }

        let commission_amount = self.calculate(order_id, order_amount, commission_rate);
        if commission_amount <= 0 {
            info!("Commission amount=0 for order {}, skipping", order_id);
            return Ok(());
        }

        sqlx::query(
            r#"
            INSERT INTO commission_records
                (order_id, user_id, referrer_id, order_amount, commission_amount, commission_rate, status)
            VALUES ($1, 0, $2, $3, $4, $5, 'pending')
            ON CONFLICT (order_id) DO NOTHING
            "#,
        )
            .bind(order_id)
            .bind(referrer_id)
            .bind(order_amount)
            .bind(commission_amount)
            .bind(commission_rate)
            .execute(&self.pool)
            .await
            .map_err(|e| RswsError::internal(format!("Insert commission: {}", e)))?;

        info!(
            "Commission settled: order={} referrer={} amount={} rate={}",
            order_id, referrer_id, commission_amount, commission_rate
        );
        Ok(())
    }

    /// 将 pending 佣金记录标记为 settled
    pub async fn settle(&self, order_id: i64) -> Result<(), RswsError> {
        sqlx::query(
            "UPDATE commission_records SET status = 'settled', settled_at = NOW() WHERE order_id = $1 AND status = 'pending'",
        )
            .bind(order_id)
            .execute(&self.pool)
            .await
            .map_err(|e| RswsError::internal(format!("Settle commission: {}", e)))?;

        info!("Commission settled for order: {}", order_id);
        Ok(())
    }
}
