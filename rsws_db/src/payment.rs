//! 支付交易仓储层

use chrono::Utc;
use rsws_common::error::RswsError;
use rsws_common::snowflake;
use rsws_model::payment::PaymentTransaction;
use sqlx::PgPool;

/// 支付交易仓储
pub struct PaymentRepository {
    pool: PgPool,
}

impl PaymentRepository {
    /// 创建支付交易仓储实例
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 创建支付交易
    pub async fn create(
        &self,
        order_id: i64,
        user_id: i64,
        amount: i64,
        currency: &str,
        payment_method: &str,
    ) -> Result<PaymentTransaction, RswsError> {
        let transaction_id = snowflake::next_id();

        let transaction = sqlx::query_as::<_, PaymentTransaction>(
            r#"
            INSERT INTO payment_transactions (id, order_id, user_id, amount, currency, payment_method, status, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, 'pending', NOW(), NOW())
            RETURNING id, order_id, user_id, amount, currency, payment_method, provider_transaction_id, status, created_at, updated_at, completed_at
            "#,
        )
        .bind(transaction_id)
        .bind(order_id)
        .bind(user_id)
        .bind(amount)
        .bind(currency)
        .bind(payment_method)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to create transaction: {}", e)))?;

        Ok(transaction)
    }

    /// 根据 ID 获取交易
    pub async fn get_by_id(&self, id: i64) -> Result<Option<PaymentTransaction>, RswsError> {
        let transaction = sqlx::query_as::<_, PaymentTransaction>(
            "SELECT id, order_id, user_id, amount, currency, payment_method, provider_transaction_id, status, created_at, updated_at, completed_at FROM payment_transactions WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to get transaction: {}", e)))?;

        Ok(transaction)
    }

    /// 根据订单 ID 获取交易
    pub async fn get_by_order_id(&self, order_id: i64) -> Result<Vec<PaymentTransaction>, RswsError> {
        let transactions = sqlx::query_as::<_, PaymentTransaction>(
            "SELECT id, order_id, user_id, amount, currency, payment_method, provider_transaction_id, status, created_at, updated_at, completed_at FROM payment_transactions WHERE order_id = $1 ORDER BY created_at DESC",
        )
        .bind(order_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to get transactions: {}", e)))?;

        Ok(transactions)
    }

    /// 更新交易状态
    pub async fn update_status(
        &self,
        transaction_id: i64,
        status: &str,
        provider_transaction_id: Option<&str>,
    ) -> Result<(), RswsError> {
        let completed_at = if status == "completed" {
            Some(Utc::now())
        } else {
            None
        };

        sqlx::query(
            r#"
            UPDATE payment_transactions 
            SET status = $1::transaction_status, 
                provider_transaction_id = COALESCE($2, provider_transaction_id),
                completed_at = $3,
                updated_at = NOW() 
            WHERE id = $4
            "#,
        )
        .bind(status)
        .bind(provider_transaction_id)
        .bind(completed_at)
        .bind(transaction_id)
        .execute(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to update transaction: {}", e)))?;

        Ok(())
    }

    /// 根据 PayPal order ID（provider_transaction_id）查找交易
    pub async fn get_by_provider_tx(
        &self,
        provider_tx_id: &str,
    ) -> Result<Option<PaymentTransaction>, RswsError> {
        let tx = sqlx::query_as::<_, PaymentTransaction>(
            "SELECT id, order_id, user_id, amount, currency, payment_method, provider_transaction_id, status, created_at, updated_at, completed_at FROM payment_transactions WHERE provider_transaction_id = $1",
        )
        .bind(provider_tx_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to get transaction by provider tx: {}", e)))?;

        Ok(tx)
    }

    /// 获取所有待处理交易（webhook 查找用，生产环境建议加索引）
    pub async fn get_all_pending(&self) -> Result<Vec<PaymentTransaction>, RswsError> {
        let txs = sqlx::query_as::<_, PaymentTransaction>(
            "SELECT id, order_id, user_id, amount, currency, payment_method, provider_transaction_id, status, created_at, updated_at, completed_at FROM payment_transactions WHERE status = 'pending' ORDER BY created_at DESC LIMIT 1000",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to get pending transactions: {}", e)))?;
        Ok(txs)
    }

    /// 获取用户交易记录
    pub async fn get_user_transactions(
        &self,
        user_id: i64,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<PaymentTransaction>, i64), RswsError> {
        let offset = (page - 1) * page_size;

        // 获取交易列表
        let transactions = sqlx::query_as::<_, PaymentTransaction>(
            r#"
            SELECT id, order_id, user_id, amount, currency, payment_method, provider_transaction_id, status, created_at, updated_at, completed_at
            FROM payment_transactions 
            WHERE user_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(user_id)
        .bind(page_size)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to get user transactions: {}", e)))?;

        // 获取总数
        let total: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM payment_transactions WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to count transactions: {}", e)))?;

        Ok((transactions, total.0))
    }
}

// ==================== 单元测试 ====================

#[cfg(test)]
mod tests {
    

    #[test]
    fn test_payment_repository_new() {
        // 仅测试构造函数
    }
}
