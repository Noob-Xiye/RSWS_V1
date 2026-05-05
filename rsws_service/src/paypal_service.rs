//! PayPal 服务

use rsws_common::error::RswsError;
use rsws_common::config::PayPalConfig;
use rsws_common::snowflake;
use reqwest::Client;
use serde_json::Value;
use tracing::{info, warn};

/// PayPal 服务
pub struct PayPalService {
    client: Client,
    config: PayPalConfig,
}

impl PayPalService {
    /// 创建 PayPal 服务实例
    pub fn new(config: PayPalConfig) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }

    /// 获取 Access Token
    pub async fn get_access_token(&self) -> Result<String, RswsError> {
        let auth_url = if self.config.mode == "live" {
            "https://api-m.paypal.com/v1/oauth2/token"
        } else {
            "https://api-m.sandbox.paypal.com/v1/oauth2/token"
        };

        let params = [
            ("grant_type", "client_credentials"),
        ];

        let resp = self.client
            .post(auth_url)
            .form(&params)
            .basic_auth(&self.config.client_id, Some(&self.config.client_secret))
            .send()
            .await
            .map_err(|e| RswsError::internal(format!("Failed to get access token: {}", e)))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            warn!("PayPal auth failed: {} - {}", status, body);
            return Err(RswsError::internal("Failed to authenticate with PayPal"));
        }

        let json: Value = resp.json().await
            .map_err(|e| RswsError::internal(format!("Failed to parse token response: {}", e)))?;

        json["access_token"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| RswsError::internal("Invalid token response"))
    }

    /// 创建支付订单
    pub async fn create_order(
        &self,
        amount: f64,
        currency: &str,
        description: &str,
        order_id: i64,
    ) -> Result<Value, RswsError> {
        if self.config.client_id.is_empty() {
            // Mock 模式
            info!("Creating mock PayPal order for order_id: {}", order_id);
            return Ok(serde_json::json!({
                "id": format!("PAYPAL-{}", snowflake::next_id()),
                "status": "CREATED",
                "links": [
                    {
                        "rel": "approve",
                        "href": format!("{}?token={}", self.config.return_url, snowflake::next_id())
                    }
                ]
            }));
        }

        let token = self.get_access_token().await?;
        let order_url = if self.config.mode == "live" {
            "https://api-m.paypal.com/v2/checkout/orders"
        } else {
            "https://api-m.sandbox.paypal.com/v2/checkout/orders"
        };

        let order_request = serde_json::json!({
            "intent": "CAPTURE",
            "purchase_units": [{
                "reference_id": order_id.to_string(),
                "description": description,
                "amount": {
                    "currency_code": currency,
                    "value": format!("{:.2}", amount)
                }
            }],
            "application_context": {
                "return_url": self.config.return_url,
                "cancel_url": self.config.cancel_url
            }
        });

        let resp = self.client
            .post(order_url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(&order_request)
            .send()
            .await
            .map_err(|e| RswsError::internal(format!("Failed to create PayPal order: {}", e)))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            warn!("PayPal create order failed: {} - {}", status, body);
            return Err(RswsError::internal("Failed to create PayPal order"));
        }

        let json: Value = resp.json().await
            .map_err(|e| RswsError::internal(format!("Failed to parse PayPal response: {}", e)))?;

        info!("PayPal order created: {:?}", json["id"]);
        Ok(json)
    }

    /// 捕获支付
    pub async fn capture_order(&self, paypal_order_id: &str) -> Result<Value, RswsError> {
        if self.config.client_id.is_empty() {
            // Mock 模式
            info!("Capturing mock PayPal order: {}", paypal_order_id);
            return Ok(serde_json::json!({
                "id": paypal_order_id,
                "status": "COMPLETED",
                "purchase_units": [{
                    "payments": {
                        "captures": [{
                            "id": format!("CAPTURE-{}", snowflake::next_id()),
                            "status": "COMPLETED"
                        }]
                    }
                }]
            }));
        }

        let token = self.get_access_token().await?;
        let capture_url = if self.config.mode == "live" {
            format!("https://api-m.paypal.com/v2/checkout/orders/{}/capture", paypal_order_id)
        } else {
            format!("https://api-m.sandbox.paypal.com/v2/checkout/orders/{}/capture", paypal_order_id)
        };

        let resp = self.client
            .post(&capture_url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|e| RswsError::internal(format!("Failed to capture PayPal order: {}", e)))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            warn!("PayPal capture failed: {} - {}", status, body);
            return Err(RswsError::internal("Failed to capture PayPal payment"));
        }

        let json: Value = resp.json().await
            .map_err(|e| RswsError::internal(format!("Failed to parse PayPal response: {}", e)))?;

        info!("PayPal order captured: {:?}", json["id"]);
        Ok(json)
    }

    /// 验证 Webhook 签名（简化版）
    pub async fn verify_webhook(
        &self,
        _headers: &[(String, String)],
        _body: &[u8],
    ) -> Result<bool, RswsError> {
        // TODO: 实现真实的 PayPal Webhook 签名验证
        // https://developer.paypal.com/docs/api-basics/notifications/webhooks/verify-webhook-signatures/
        
        Ok(true)
    }
}