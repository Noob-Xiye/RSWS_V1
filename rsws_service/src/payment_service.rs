//! 支付服务

use rsws_common::error::RswsError;
use rsws_db::PaymentRepository;
use std::sync::Arc;
use tracing::info;

/// 支付服务
pub struct PaymentService {
    payment_repo: Arc<PaymentRepository>,
}

impl PaymentService {
    /// 创建支付服务实例
    pub fn new(payment_repo: Arc<PaymentRepository>) -> Self {
        Self { payment_repo }
    }

    /// 创建支付交易
    pub async fn create(
        &self,
        order_id: i64,
        user_id: i64,
        amount: i64,
        currency: &str,
        payment_method: &str,
    ) -> Result<i64, RswsError> {
        let transaction = self.payment_repo
            .create(order_id, user_id, amount, currency, payment_method)
            .await?;

        info!("Payment transaction created: {}", transaction.id);

        Ok(transaction.id)
    }

    /// 更新交易状态
    pub async fn update_status(
        &self,
        transaction_id: i64,
        status: &str,
        provider_tx_id: Option<&str>,
    ) -> Result<(), RswsError> {
        self.payment_repo.update_status(transaction_id, status, provider_tx_id).await
    }
}
