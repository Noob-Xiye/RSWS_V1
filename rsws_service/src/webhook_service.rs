//! Webhook 服务

use rsws_common::error::RswsError;
use serde_json::Value;
use sqlx::PgPool;
use tracing::info;

/// Webhook 服务
pub struct WebhookService {
    pool: PgPool,
}

impl WebhookService {
    /// 创建 Webhook 服务实例
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 处理 PayPal Webhook
    pub async fn handle_paypal(&self, payload: Value) -> Result<(), RswsError> {
        info!("Handling PayPal webhook: {:?}", payload);

        // TODO: 实现 PayPal webhook 处理逻辑

        Ok(())
    }

    /// 处理 USDT Webhook
    pub async fn handle_usdt(&self, payload: Value) -> Result<(), RswsError> {
        info!("Handling USDT webhook: {:?}", payload);

        // TODO: 实现 USDT webhook 处理逻辑

        Ok(())
    }
}
