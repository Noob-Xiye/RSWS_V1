use sqlx::PgPool;
use rsws_model::payment::{PaymentTransaction, TransactionStatus};
use rsws_common::error::ServiceError;
use chrono::{DateTime, Utc};

pub struct PaymentRepository {
    pool: PgPool,
}

impl PaymentRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // 创建支付交易
    pub async fn create_transaction(
        &self,
        transaction: &PaymentTransaction,
    ) -> Result<PaymentTransaction, ServiceError> {
        let result = sqlx::query_as!(
            PaymentTransaction,
            r#"
            INSERT INTO payment_transactions (id, order_id, user_id, amount, currency, payment_method, provider_transaction_id, status, created_at, updated_at, completed_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8::transaction_status, $9, $10, $11)
            RETURNING id, order_id, user_id, amount, currency, payment_method, provider_transaction_id, status as "status: TransactionStatus", created_at, updated_at, completed_at
            "#,
            transaction.id,
            transaction.order_id,
            transaction.user_id,
            transaction.amount,
            transaction.currency,
            transaction.payment_method,
            transaction.provider_transaction_id,
            transaction.status as TransactionStatus,
            transaction.created_at,
            transaction.updated_at,
            transaction.completed_at
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        Ok(result)
    }

    // 根据ID获取交易
    pub async fn get_by_id(&self, id: &str) -> Result<Option<PaymentTransaction>, ServiceError> {
        let result = sqlx::query_as!(
            PaymentTransaction,
            r#"
            SELECT id, order_id, user_id, amount, currency, payment_method, provider_transaction_id, status as "status: TransactionStatus", created_at, updated_at, completed_at
            FROM payment_transactions 
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        Ok(result)
    }

    // 根据订单ID获取交易
    pub async fn get_by_order_id(
        &self,
        order_id: i64,
    ) -> Result<Vec<PaymentTransaction>, ServiceError> {
        let result = sqlx::query_as!(
            PaymentTransaction,
            r#"
            SELECT id, order_id, user_id, amount, currency, payment_method, provider_transaction_id, status as "status: TransactionStatus", created_at, updated_at, completed_at
            FROM payment_transactions 
            WHERE order_id = $1
            ORDER BY created_at DESC
            "#,
            order_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        Ok(result)
    }

    // 更新交易状态
    pub async fn update_transaction_status(
        &self,
        transaction_id: &str,
        status: TransactionStatus,
        provider_transaction_id: Option<String>,
    ) -> Result<(), ServiceError> {
        let completed_at = if status == TransactionStatus::Completed {
            Some(Utc::now())
        } else {
            None
        };

        sqlx::query!(
            r#"
            UPDATE payment_transactions 
            SET status = $1::transaction_status, 
                provider_transaction_id = COALESCE($2, provider_transaction_id),
                completed_at = $3,
                updated_at = NOW() 
            WHERE id = $4
            "#,
            status as TransactionStatus,
            provider_transaction_id,
            completed_at,
            transaction_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    // 获取用户交易记录
    pub async fn get_user_transactions(
        &self,
        user_id: i64,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<PaymentTransaction>, i64), ServiceError> {
        let offset = (page - 1) * page_size;

        // 获取交易列表
        let transactions = sqlx::query_as!(
            PaymentTransaction,
            r#"
            SELECT id, order_id, user_id, amount, currency, payment_method, provider_transaction_id, status as "status: TransactionStatus", created_at, updated_at, completed_at
            FROM payment_transactions 
            WHERE user_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            user_id,
            page_size,
            offset
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        // 获取总数
        let total: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM payment_transactions WHERE user_id = $1",
            user_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        Ok((transactions, total))
    }
}