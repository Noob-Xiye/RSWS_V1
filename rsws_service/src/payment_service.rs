//! 支付服务

use rust_decimal::Decimal;

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
        amount: Decimal,
        currency: &str,
        payment_method: &str,
    ) -> Result<i64, RswsError> {
        let transaction = self
            .payment_repo
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
        self.payment_repo
            .update_status(transaction_id, status, provider_tx_id)
            .await
    }

    /// 根据 PayPal order ID 查找交易
    ///
    /// provider_transaction_id 存储 PayPal order ID
    pub async fn get_by_paypal_order(
        &self,
        paypal_order_id: &str,
    ) -> Result<Option<rsws_model::payment::PaymentTransaction>, RswsError> {
        self.payment_repo.get_by_provider_tx(paypal_order_id).await
    }

    /// 根据订单 ID 查找交易
    pub async fn get_by_order(
        &self,
        order_id: i64,
    ) -> Result<Vec<rsws_model::payment::PaymentTransaction>, RswsError> {
        self.payment_repo.get_by_order_id(order_id).await
    }

    /// 获取交易信息
    pub async fn get_transaction(
        &self,
        transaction_id: i64,
    ) -> Result<Option<rsws_model::payment::PaymentTransaction>, RswsError> {
        self.payment_repo.get_by_id(transaction_id).await
    }

    /// 获取用户的交易记录
    pub async fn get_user_transactions(
        &self,
        user_id: i64,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<rsws_model::payment::PaymentTransaction>, i64), RswsError> {
        self.payment_repo
            .get_user_transactions(user_id, page, page_size)
            .await
    }
}
