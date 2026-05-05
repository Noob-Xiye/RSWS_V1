//! Webhook 服务

use rsws_common::error::RswsError;
use serde_json::Value;
use tracing::info;

/// Webhook 服务
pub struct WebhookService;

impl WebhookService {
    /// 创建 Webhook 服务实例
    pub fn new() -> Self {
        Self
    }

    /// 处理 PayPal Webhook
    pub async fn handle_paypal(&self, payload: Value) -> Result<String, RswsError> {
        info!("Handling PayPal webhook: {:?}", payload);

        let event_type = payload.get("event_type")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        match event_type {
            "CHECKOUT.ORDER.APPROVED" => {
                info!("PayPal order approved");
                Ok("ORDER_APPROVED".to_string())
            }
            "PAYMENT.CAPTURE.COMPLETED" => {
                info!("PayPal payment completed");
                Ok("PAYMENT_COMPLETED".to_string())
            }
            "PAYMENT.CAPTURE.DENIED" => {
                info!("PayPal payment denied");
                Ok("PAYMENT_DENIED".to_string())
            }
            "PAYMENT.CAPTURE.REFUNDED" => {
                info!("PayPal payment refunded");
                Ok("PAYMENT_REFUNDED".to_string())
            }
            _ => {
                info!("Unhandled PayPal event: {}", event_type);
                Ok("UNHANDLED".to_string())
            }
        }
    }

    /// 处理 USDT Webhook
    pub async fn handle_usdt(&self, payload: Value) -> Result<String, RswsError> {
        info!("Handling USDT webhook: {:?}", payload);

        let tx_hash = payload.get("tx_hash")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let status = payload.get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("pending");

        if tx_hash.is_empty() {
            return Err(RswsError::business(
                rsws_common::error_code::ErrorCode::USDT_TRANSACTION_NOT_FOUND
            ));
        }

        info!("USDT transaction {} status: {}", tx_hash, status);
        Ok(status.to_uppercase())
    }

    /// 验证 PayPal Webhook 签名
    pub async fn verify_paypal_signature(
        &self,
        _headers: &[(String, String)],
        _body: &[u8],
    ) -> Result<bool, RswsError> {
        // TODO: 实现真实的 PayPal Webhook 签名验证
        // https://developer.paypal.com/docs/api-basics/notifications/webhooks/verify-webhook-signatures/
        
        Ok(true)
    }
}

impl Default for WebhookService {
    fn default() -> Self {
        Self::new()
    }
}