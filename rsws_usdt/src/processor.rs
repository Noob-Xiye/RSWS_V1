//! 交易处理器

use crate::{UsdtError, matcher::{PendingOrder, OrderMatcher, MatchStrategy}};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::{info, warn};

/// USDT 交易记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsdtTransaction {
    /// 交易 ID (雪花 ID)
    pub id: i64,

    /// 交易 Hash
    pub tx_hash: String,

    /// 网络类型
    pub network: String,

    /// 发送地址
    pub from_address: String,

    /// 接收地址
    pub to_address: String,

    /// 金额
    pub amount: Decimal,

    /// 区块号
    pub block_number: i64,

    /// 确认数
    pub confirmations: i32,

    /// 状态: pending, confirmed, processed
    pub status: String,

    /// 关联订单 ID
    pub order_id: Option<i64>,

    /// 处理时间
    pub processed_at: Option<DateTime<Utc>>,

    /// 创建时间
    pub created_at: DateTime<Utc>,
}

/// 交易处理器
pub struct TransactionProcessor {
    db_pool: PgPool,
    matcher: OrderMatcher,
}

impl TransactionProcessor {
    /// 创建新处理器
    pub fn new(db_pool: PgPool, match_strategy: MatchStrategy) -> Self {
        Self {
            db_pool,
            matcher: OrderMatcher::new(match_strategy),
        }
    }

    /// 处理交易
    ///
    /// 1. 检查交易是否已处理 (幂等)
    /// 2. 匹配订单
    /// 3. 确认订单
    /// 4. 记录交易
    pub async fn process_transaction(
        &self,
        tx_hash: &str,
        network: &str,
        from_address: &str,
        to_address: &str,
        amount: Decimal,
        block_number: i64,
        confirmations: i32,
    ) -> Result<ProcessResult, UsdtError> {
        // 1. 检查是否已处理
        if self.is_transaction_processed(tx_hash).await? {
            info!(tx_hash = %tx_hash, "Transaction already processed, skipping");
            return Ok(ProcessResult::AlreadyProcessed);
        }

        // 2. 获取待匹配订单
        let pending_orders = self.get_pending_orders(to_address).await?;

        if pending_orders.is_empty() {
            info!(to_address = %to_address, "No pending orders for address");
            // 记录未匹配交易
            self.record_transaction(
                tx_hash,
                network,
                from_address,
                to_address,
                amount,
                block_number,
                confirmations,
                None,
                "unmatched",
            ).await?;
            return Ok(ProcessResult::NoMatchingOrder);
        }

        // 3. 匹配订单
        let match_result = self.matcher.match_order(amount, to_address, &pending_orders);

        if !match_result.matched {
            warn!(
                tx_hash = %tx_hash,
                amount = %amount,
                "Transaction amount does not match any order"
            );
            self.record_transaction(
                tx_hash,
                network,
                from_address,
                to_address,
                amount,
                block_number,
                confirmations,
                None,
                "unmatched",
            ).await?;
            return Ok(ProcessResult::NoMatchingOrder);
        }

        let order_id = match_result.order_id.unwrap();

        // 4. 确认订单
        self.confirm_order(order_id, tx_hash).await?;

        // 5. 记录交易
        self.record_transaction(
            tx_hash,
            network,
            from_address,
            to_address,
            amount,
            block_number,
            confirmations,
            Some(order_id),
            "processed",
        ).await?;

        info!(
            tx_hash = %tx_hash,
            order_id = order_id,
            amount = %amount,
            "Transaction processed successfully"
        );

        Ok(ProcessResult::Success { order_id })
    }

    /// 检查交易是否已处理
    async fn is_transaction_processed(&self, tx_hash: &str) -> Result<bool, UsdtError> {
        let result = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM usdt_transactions WHERE tx_hash = $1"
        )
        .bind(tx_hash)
        .fetch_one(&self.db_pool)
        .await?;

        Ok(result > 0)
    }

    /// 获取待匹配订单
    async fn get_pending_orders(&self, wallet_address: &str) -> Result<Vec<PendingOrder>, UsdtError> {
        let rows = sqlx::query_as::<_, (i64, i64, Decimal, String, String, DateTime<Utc>, Option<DateTime<Utc>>)>(
            r#"
            SELECT
                o.id,
                o.user_id,
                o.amount,
                uw.address as wallet_address,
                uw.network,
                o.created_at,
                o.expired_at
            FROM orders o
            JOIN usdt_wallets uw ON o.payment_method = CONCAT('usdt_', uw.network)
            WHERE o.status = 'pending'
                AND uw.address = $1
                AND uw.is_active = true
                AND (o.expired_at IS NULL OR o.expired_at > NOW())
            "#
        )
        .bind(wallet_address)
        .fetch_all(&self.db_pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|(order_id, user_id, amount, wallet_address, network, created_at, expires_at)| {
                PendingOrder {
                    order_id,
                    user_id,
                    amount,
                    wallet_address,
                    network,
                    created_at,
                    expires_at,
                }
            })
            .collect())
    }

    /// 确认订单
    async fn confirm_order(&self, order_id: i64, tx_hash: &str) -> Result<(), UsdtError> {
        let mut tx = self.db_pool.begin().await?;

        // 更新订单状态
        sqlx::query(
            r#"
            UPDATE orders
            SET status = 'completed',
                transaction_id = $2,
                completed_at = NOW(),
                updated_at = NOW()
            WHERE id = $1 AND status = 'pending'
            "#
        )
        .bind(order_id)
        .bind(tx_hash)
        .execute(&mut *tx)
        .await?;

        // TODO: 处理佣金结算
        // TODO: 开放资源下载权限

        tx.commit().await?;

        Ok(())
    }

    /// 记录交易
    async fn record_transaction(
        &self,
        tx_hash: &str,
        network: &str,
        from_address: &str,
        to_address: &str,
        amount: Decimal,
        block_number: i64,
        confirmations: i32,
        order_id: Option<i64>,
        status: &str,
    ) -> Result<(), UsdtError> {
        // 生成雪花 ID
        let id = rsws_common::snowflake::next_id();

        sqlx::query(
            r#"
            INSERT INTO usdt_transactions (
                id, tx_hash, network, from_address, to_address,
                amount, block_number, confirmations, status, order_id, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, NOW())
            ON CONFLICT (tx_hash) DO NOTHING
            "#
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
        .await?;

        Ok(())
    }
}

/// 处理结果
#[derive(Debug, Clone)]
pub enum ProcessResult {
    /// 处理成功
    Success { order_id: i64 },

    /// 已处理 (幂等)
    AlreadyProcessed,

    /// 无匹配订单
    NoMatchingOrder,

    /// 确认数不足
    InsufficientConfirmations,
}
