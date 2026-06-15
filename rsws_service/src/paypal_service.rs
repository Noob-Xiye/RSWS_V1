//! PayPal 服务
//!
//! 配置从 paypal_configs 数据库表读取，不再依赖 config.toml

use crate::config_service::PayPalDbConfig;
use reqwest::Client;
use rsws_common::error::RswsError;
use rsws_common::snowflake;
use serde_json::Value;
use tracing::{info, warn};

/// PayPal 服务
pub struct PayPalService {
    client: Client,
    config: Option<PayPalDbConfig>,
}

impl PayPalService {
    /// 创建 PayPal 服务实例（可无配置，将使用 mock 模式）
    pub fn new(config: Option<PayPalDbConfig>) -> Self {
        Self {
            client: Client::builder()
                .use_rustls_tls()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .expect("valid reqwest client"),
            config,
        }
    }

    /// 是否已配置
    pub fn is_configured(&self) -> bool {
        self.config
            .as_ref()
            .is_some_and(|c| !c.client_id.is_empty())
    }

    /// 获取 base URL（sandbox 或 live）
    fn api_base_url(&self) -> &str {
        match &self.config {
            Some(c) if !c.sandbox => "https://api-m.paypal.com",
            _ => "https://api-m.sandbox.paypal.com",
        }
    }

    /// 获取 Access Token
    pub async fn get_access_token(&self) -> Result<String, RswsError> {
        let config = self
            .config
            .as_ref()
            .ok_or_else(|| RswsError::internal("PayPal not configured"))?;

        let auth_url = format!("{}/v1/oauth2/token", self.api_base_url());

        let params = [("grant_type", "client_credentials")];

        let resp = self
            .client
            .post(&auth_url)
            .form(&params)
            .basic_auth(&config.client_id, Some(&config.client_secret))
            .send()
            .await
            .map_err(|e| RswsError::internal(format!("Failed to get access token: {}", e)))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            warn!("PayPal auth failed: {} - {}", status, body);
            return Err(RswsError::internal("Failed to authenticate with PayPal"));
        }

        let json: Value = resp
            .json()
            .await
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
        let config = match &self.config {
            Some(c) if !c.client_id.is_empty() => c,
            _ => {
                // Mock 模式
                info!("Creating mock PayPal order for order_id: {}", order_id);
                return Ok(serde_json::json!({
                    "id": format!("PAYPAL-{}", snowflake::next_id()),
                    "status": "CREATED",
                    "links": [
                        {
                            "rel": "approve",
                            "href": format!("http://localhost:3000/payment/success?token={}", snowflake::next_id())
                        }
                    ]
                }));
            }
        };

        let token = self.get_access_token().await?;
        let order_url = format!("{}/v2/checkout/orders", self.api_base_url());

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
                "return_url": config.return_url,
                "cancel_url": config.cancel_url,
                "brand_name": config.brand_name
            }
        });

        let resp = self
            .client
            .post(&order_url)
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

        let json: Value = resp
            .json()
            .await
            .map_err(|e| RswsError::internal(format!("Failed to parse PayPal response: {}", e)))?;

        info!("PayPal order created: {:?}", json["id"]);
        Ok(json)
    }

    /// 捕获支付
    pub async fn capture_order(&self, paypal_order_id: &str) -> Result<Value, RswsError> {
        let config = match &self.config {
            Some(c) if !c.client_id.is_empty() => c,
            _ => {
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
        };

        let _ = config; // 使用 config 确认已配置
        let token = self.get_access_token().await?;
        let capture_url = format!(
            "{}/v2/checkout/orders/{}/capture",
            self.api_base_url(),
            paypal_order_id
        );

        let resp = self
            .client
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

        let json: Value = resp
            .json()
            .await
            .map_err(|e| RswsError::internal(format!("Failed to parse PayPal response: {}", e)))?;

        info!("PayPal order captured: {:?}", json["id"]);
        Ok(json)
    }

    /// 验证 Webhook 签名
    pub async fn verify_webhook(
        &self,
        headers: &[(String, String)],
        body: &[u8],
    ) -> Result<bool, RswsError> {
        let config = match &self.config {
            Some(c) => c,
            None => {
                warn!("PayPal not configured — skipping webhook verification");
                return Ok(true);
            }
        };

        let webhook_id = config.webhook_id.as_deref().unwrap_or("");

        // Dev 模式：无 webhook_id 时跳过验证
        if webhook_id.is_empty() {
            warn!("PayPal webhook_id not configured — skipping signature verification (DEV MODE)");
            return Ok(true);
        }

        let get_header = |key: &str| -> String {
            headers
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case(key))
                .map(|(_, v)| v.clone())
                .unwrap_or_default()
        };

        let transmission_id = get_header("PAYPAL-TRANSMISSION-ID");
        let transmission_time = get_header("PAYPAL-TRANSMISSION-TIME");
        let transmission_sig = get_header("PAYPAL-TRANSMISSION-SIG");
        let cert_url = get_header("PAYPAL-CERT-URL");

        if transmission_id.is_empty() || transmission_sig.is_empty() || cert_url.is_empty() {
            warn!("PayPal webhook headers incomplete");
            return Ok(false);
        }

        let event_json: Value = serde_json::from_slice(body)
            .map_err(|e| RswsError::internal(format!("Failed to parse webhook body: {}", e)))?;
        let event_type = event_json["event_type"].as_str().unwrap_or("UNKNOWN");

        let verify_url = format!(
            "{}/v1/notifications/verify-webhook-signature",
            self.api_base_url()
        );

        let verify_body = serde_json::json!({
            "auth_algo": "SHA256withRSA",
            "cert_url": cert_url,
            "transmission_id": transmission_id,
            "transmission_sig": transmission_sig,
            "transmission_time": transmission_time,
            "webhook_id": webhook_id,
            "webhook_event": event_json
        });

        let token = match self.get_access_token().await {
            Ok(t) => t,
            Err(e) => {
                warn!("PayPal verify_webhook: failed to get access token: {}", e);
                return Ok(false);
            }
        };

        let resp = self
            .client
            .post(&verify_url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(&verify_body)
            .send()
            .await
            .map_err(|e| RswsError::internal(format!("PayPal verify request failed: {}", e)))?;

        let resp_json: Value = resp
            .json()
            .await
            .unwrap_or_else(|_| serde_json::json!({ "verification_status": "FAILED" }));

        let status = resp_json["verification_status"]
            .as_str()
            .unwrap_or("FAILED");

        if status == "SUCCESS" {
            info!("PayPal webhook signature verified: event={}", event_type);
            Ok(true)
        } else {
            warn!(
                "PayPal webhook verification FAILED: {} | resp={}",
                event_type, resp_json
            );
            Ok(false)
        }
    }
}
