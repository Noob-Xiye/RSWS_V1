//! 交易处理器

use crate::{UsdtError, matcher::PendingOrder};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::info;

/// USDT 交易记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsdtTransaction {
    pub id: i64,
    pub tx_hash: String,
    pub network: String,
    pub from_address: String,
    pub to_address: String,
    pub amount: Decimal,
    pub block_number: i64,
    pub confirmations: i32,
    pub status: String,
    pub order_id: Option<i64>,
    pub processed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// 交易处理器
pub struct TransactionProcessor {
    db_pool: PgPool,
}

impl TransactionProcessor {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    /// 处理单笔交易
    pub async fn process_transaction(&self, tx: UsdtTransaction) -> Result<bool, UsdtError> {
        // 检查是否已处理
        if self.is_transaction_processed(&tx.tx_hash).await? {
            return Ok(false);
        }

        // 查询该地址的待支付订单
        let pending_orders = self.get_pending_orders(&tx.to_address).await?;

        for order in pending_orders {
            let tolerance = order.amount * Decimal::try_from(0.01).map_err(|_| UsdtError::InvalidAmount)?;

            if (tx.amount - order.amount).abs() <= tolerance {
                info!(
                    "Matched order {}: tx_amount={}, order_amount={}",
                    order.order_id, tx.amount, order.amount
                );

                self.confirm_order(order.order_id, &tx.tx_hash).await?;

                self.record_transaction(
                    tx.tx_hash.clone(),
                    tx.network.clone(),
                    tx.from_address.clone(),
                    tx.to_address.clone(),
                    tx.amount,
                    tx.block_number,
                    tx.confirmations,
                    Some(order.order_id),
                    "processed",
                ).await?;

                return Ok(true);
            }
        }

        // 未匹配，记录为未匹配交易
        self.record_transaction(
            tx.tx_hash,
            tx.network,
            tx.from_address,
            tx.to_address,
            tx.amount,
            tx.block_number,
            tx.confirmations,
            None,
            "unmatched",
        ).await?;

        Ok(false)
    }

    async fn is_transaction_processed(&self, tx_hash: &str) -> Result<bool, UsdtError> {
        let row: (bool,) = sqlx::query_as(
            "SELECT EXISTS(SELECT 1 FROM usdt_transactions WHERE tx_hash = $1)"
        )
        .bind(tx_hash)
        .fetch_one(&self.db_pool)
        .await
        .map_err(|e| UsdtError::DatabaseError(e.to_string()))?;

        Ok(row.0)
    }

    async fn get_pending_orders(&self, wallet_address: &str) -> Result<Vec<PendingOrder>, UsdtError> {
        let rows: Vec<(i64, i64, Decimal, DateTime<Utc>, Option<DateTime<Utc>>)> = sqlx::query_as(
            r#"
            SELECT o.id, o.user_id, o.amount, o.created_at, o.expired_at
            FROM orders o
            JOIN resources r ON r.id = o.resource_id
            JOIN usdt_wallets w ON w.id = r.wallet_id
            WHERE w.address = $1
              AND o.status = 'pending'
              AND (o.expired_at IS NULL OR o.expired_at > NOW())
            ORDER BY o.created_at ASC
            "#,
        )
        .bind(wallet_address)
        .fetch_all(&self.db_pool)
        .await
        .map_err(|e| UsdtError::DatabaseError(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|(order_id, user_id, amount, created_at, expires_at)| PendingOrder {
                order_id,
                user_id,
                amount,
                wallet_address: wallet_address.to_string(),
                network: "tron".to_string(),
                created_at,
                expires_at,
            })
            .collect())
    }

    /// 确认订单 — 在数据库事务中执行
    ///
    /// 包含两个业务操作：
    /// 1. **佣金结算**：订单完成后，根据资源的 `commission_rate` 计算佣金并记录到 `commission_records`
    /// 2. **资源下载权限**：`status = 'completed'` 即代表用户有下载权限（下载时通过 orders 表验证）
    async fn confirm_order(&self, order_id: i64, tx_hash: &str) -> Result<(), UsdtError> {
        let mut db_tx = self.db_pool.begin().await.map_err(|e| UsdtError::DatabaseError(e.to_string()))?;

        // ① 更新订单状态为已完成
        let affected = sqlx::query(
            r#"
            UPDATE orders
            SET status = 'completed',
                transaction_id = $2,
                updated_at = NOW()
            WHERE id = $1 AND status = 'pending'
            "#,
        )
        .bind(order_id)
        .bind(tx_hash)
        .execute(&mut *db_tx)
        .await
        .map_err(|e| UsdtError::DatabaseError(e.to_string()))?;

        if affected.rows_affected() == 0 {
            db_tx.commit().await.map_err(|e| UsdtError::DatabaseError(e.to_string()))?;
            return Ok(());
        }

        // ② 佣金结算
        let order_info: Option<(i64, i64, Decimal)> = sqlx::query_as(
            "SELECT user_id, resource_id, amount FROM orders WHERE id = $1"
        )
        .bind(order_id)
        .fetch_optional(&mut *db_tx)
        .await
        .map_err(|e| UsdtError::DatabaseError(e.to_string()))?;

        if let Some((_buyer_id, resource_id, order_amount)) = order_info {
            let resource: Option<(Decimal, Option<i64>)> = sqlx::query_as(
                "SELECT commission_rate, provider_id FROM resources WHERE id = $1"
            )
            .bind(resource_id)
            .fetch_optional(&mut *db_tx)
            .await
            .map_err(|e| UsdtError::DatabaseError(e.to_string()))?;

            if let Some((commission_rate, provider_id)) = resource {
                let commission_amount = order_amount * commission_rate;

                let default_rule_id: (i64,) = sqlx::query_as(
                    "SELECT COALESCE(MIN(id), 0) FROM commission_rules WHERE is_active = true"
                )
                .fetch_one(&mut *db_tx)
                .await
                .map_err(|e| UsdtError::DatabaseError(e.to_string()))?;

                if commission_amount > Decimal::ZERO {
                    let commission_record_id = rsws_common::snowflake::next_id();
                    let _ = sqlx::query(
                        r#"
                        INSERT INTO commission_records
                            (id, order_id, user_id, referrer_id, rule_id,
                             order_amount, commission_amount, commission_rate, status, created_at)
                        VALUES ($1, $2, $3, NULL, $4, $5, $6, $7, 'pending', NOW())
                        ON CONFLICT DO NOTHING
                        "#,
                    )
                    .bind(commission_record_id)
                    .bind(order_id)
                    .bind(provider_id.unwrap_or(0))
                    .bind(default_rule_id.0)
                    .bind(order_amount)
                    .bind(commission_amount)
                    .bind(commission_rate)
                    .execute(&mut *db_tx)
                    .await
                    .map_err(|e| UsdtError::DatabaseError(e.to_string()))?;

                    info!(
                        "Commission settled: order_id={}, amount={} (rate={})",
                        order_id, commission_amount, commission_rate
                    );
                }
            }
        }

        // ③ 资源下载权限
        // orders 表已有 UNIQUE(user_id, resource_id) 约束
        // status = 'completed' 即代表该用户已购买此资源
        // 下载接口通过 check_user_purchased(order_id, user_id) 验证权限
        info!("Order {} confirmed: download access granted", order_id);

        db_tx.commit().await.map_err(|e| UsdtError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn record_transaction(
        &self,
        tx_hash: String,
        network: String,
        from_address: String,
        to_address: String,
        amount: Decimal,
        block_number: i64,
        confirmations: i32,
        order_id: Option<i64>,
        status: &str,
    ) -> Result<(), UsdtError> {
        let id = rsws_common::snowflake::next_id();

        let _ = sqlx::query(
            r#"
            INSERT INTO usdt_transactions (
                id, tx_hash, network, from_address, to_address,
                amount, block_number, confirmations, status, order_id, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, NOW())
            ON CONFLICT (tx_hash) DO NOTHING
            "#,
        )
        .bind(id)
        .bind(tx_hash)
        .bind(network)
        .bind(from_address)
        .bind(to_address)
        .bind(amount)
        .bind(block_number)
        .bind(confirmations)
        .bind(status)
        .bind(order_id)
        .execute(&self.db_pool)
        .await
        .map_err(|e| UsdtError::DatabaseError(e.to_string()))?;

        Ok(())
    }
}
