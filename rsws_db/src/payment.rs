//! 支付交易仓储层

use chrono::Utc;
use rsws_common::error::RswsError;
use rsws_common::snowflake;
use rsws_model::payment::PaymentTransaction;
use rust_decimal::Decimal;
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
        amount: Decimal,
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
    pub async fn get_by_order_id(
        &self,
        order_id: i64,
    ) -> Result<Vec<PaymentTransaction>, RswsError> {
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
        let total: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM payment_transactions WHERE user_id = $1")
                .bind(user_id)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| RswsError::internal(format!("Failed to count transactions: {}", e)))?;

        Ok((transactions, total.0))
    }
}

// ==================== PayPal 配置仓储 ====================

use rsws_model::payment::PayPalConfig;

/// PayPal 配置仓储
pub struct PayPalConfigRepository {
    pool: PgPool,
}

impl PayPalConfigRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 获取所有 PayPal 配置
    pub async fn list_all(&self) -> Result<Vec<PayPalConfig>, RswsError> {
        let configs = sqlx::query_as::<_, PayPalConfig>(
            "SELECT id, client_id, client_secret_encrypted, sandbox, webhook_id, webhook_secret_encrypted, base_url, return_url, cancel_url, brand_name, min_amount, max_amount, fee_rate, is_active, created_at, updated_at FROM paypal_configs ORDER BY id",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to list paypal configs: {}", e)))?;

        Ok(configs)
    }

    /// 根据 ID 获取 PayPal 配置
    pub async fn get_by_id(&self, id: i32) -> Result<Option<PayPalConfig>, RswsError> {
        let config = sqlx::query_as::<_, PayPalConfig>(
            "SELECT id, client_id, client_secret_encrypted, sandbox, webhook_id, webhook_secret_encrypted, base_url, return_url, cancel_url, brand_name, min_amount, max_amount, fee_rate, is_active, created_at, updated_at FROM paypal_configs WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to get paypal config: {}", e)))?;

        Ok(config)
    }

    /// 获取当前激活的 PayPal 配置
    pub async fn get_active(&self) -> Result<Option<PayPalConfig>, RswsError> {
        let config = sqlx::query_as::<_, PayPalConfig>(
            "SELECT id, client_id, client_secret_encrypted, sandbox, webhook_id, webhook_secret_encrypted, base_url, return_url, cancel_url, brand_name, min_amount, max_amount, fee_rate, is_active, created_at, updated_at FROM paypal_configs WHERE is_active = true LIMIT 1",
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to get active paypal config: {}", e)))?;

        Ok(config)
    }

    /// 更新 PayPal 配置
    pub async fn update(
        &self,
        id: i32,
        req: &rsws_model::payment::UpdatePayPalConfigRequest,
    ) -> Result<PayPalConfig, RswsError> {
        // 先获取现有配置
        let existing = self
            .get_by_id(id)
            .await?
            .ok_or_else(|| RswsError::not_found("PayPal config not found"))?;

        let updated = sqlx::query_as::<_, PayPalConfig>(
            r#"
            UPDATE paypal_configs SET
                client_id = COALESCE($1, client_id),
                client_secret_encrypted = COALESCE($2, client_secret_encrypted),
                sandbox = COALESCE($3, sandbox),
                webhook_id = CASE WHEN $4::boolean THEN $5 ELSE webhook_id END,
                webhook_secret_encrypted = CASE WHEN $6::boolean THEN $7 ELSE webhook_secret_encrypted END,
                base_url = COALESCE($8, base_url),
                return_url = COALESCE($9, return_url),
                cancel_url = COALESCE($10, cancel_url),
                brand_name = COALESCE($11, brand_name),
                min_amount = COALESCE($12, min_amount),
                max_amount = COALESCE($13, max_amount),
                fee_rate = COALESCE($14, fee_rate),
                is_active = COALESCE($15, is_active),
                updated_at = NOW()
            WHERE id = $16
            RETURNING id, client_id, client_secret_encrypted, sandbox, webhook_id, webhook_secret_encrypted, base_url, return_url, cancel_url, brand_name, min_amount, max_amount, fee_rate, is_active, created_at, updated_at
            "#,
        )
        .bind(req.client_id.as_ref().unwrap_or(&existing.client_id))
        .bind(req.client_secret_encrypted.as_ref().unwrap_or(&existing.client_secret_encrypted))
        .bind(req.sandbox)
        .bind(req.webhook_id.is_some())
        .bind(req.webhook_id.as_deref())
        .bind(req.webhook_secret_encrypted.is_some())
        .bind(req.webhook_secret_encrypted.as_deref())
        .bind(req.base_url.as_ref().unwrap_or(&existing.base_url))
        .bind(req.return_url.as_ref().unwrap_or(&existing.return_url))
        .bind(req.cancel_url.as_ref().unwrap_or(&existing.cancel_url))
        .bind(req.brand_name.as_ref().unwrap_or(&existing.brand_name))
        .bind(req.min_amount)
        .bind(req.max_amount)
        .bind(req.fee_rate)
        .bind(req.is_active)
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to update paypal config: {}", e)))?;

        Ok(updated)
    }

    /// 设置激活状态
    pub async fn set_active(&self, id: i32, is_active: bool) -> Result<(), RswsError> {
        sqlx::query("UPDATE paypal_configs SET is_active = $1, updated_at = NOW() WHERE id = $2")
            .bind(is_active)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| {
                RswsError::internal(format!("Failed to set paypal config active: {}", e))
            })?;

        Ok(())
    }

    /// 创建 PayPal 配置
    pub async fn create(
        &self,
        req: &rsws_model::payment::CreatePayPalConfigRequest,
    ) -> Result<PayPalConfig, RswsError> {
        let config = sqlx::query_as::<_, PayPalConfig>(
            r#"INSERT INTO paypal_configs (
                client_id, client_secret_encrypted, sandbox,
                base_url, return_url, cancel_url, brand_name,
                min_amount, max_amount, fee_rate, is_active
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING *"#,
        )
        .bind(&req.client_id)
        .bind(&req.client_secret_encrypted)
        .bind(req.sandbox)
        .bind(&req.base_url)
        .bind(&req.return_url)
        .bind(&req.cancel_url)
        .bind(&req.brand_name)
        .bind(req.min_amount)
        .bind(req.max_amount)
        .bind(req.fee_rate)
        .bind(req.is_active)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to create paypal config: {}", e)))?;

        Ok(config)
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
