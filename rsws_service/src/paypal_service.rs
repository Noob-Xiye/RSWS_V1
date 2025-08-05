use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use rsws_common::error::ServiceError;
use rsws_model::payment::*;
use rust_decimal::Decimal;
use std::collections::HashMap;

pub struct PayPalService {
    client: Client,
    client_id: String,
    client_secret: String,
    base_url: String,
    access_token: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct PayPalAccessToken {
    access_token: String,
    token_type: String,
    expires_in: u64,
}

#[derive(Serialize)]
struct CreateOrderRequest {
    intent: String,
    purchase_units: Vec<PurchaseUnit>,
    application_context: ApplicationContext,
}

#[derive(Serialize)]
struct PurchaseUnit {
    amount: Amount,
    description: String,
}

#[derive(Serialize)]
struct Amount {
    currency_code: String,
    value: String,
}

#[derive(Serialize)]
struct ApplicationContext {
    return_url: String,
    cancel_url: String,
    brand_name: String,
    user_action: String,
}

#[derive(Deserialize)]
struct PayPalOrderResponse {
    id: String,
    status: String,
    links: Vec<PayPalLink>,
}

#[derive(Deserialize)]
struct PayPalLink {
    href: String,
    rel: String,
    method: String,
}

impl PayPalService {
    pub fn new(client_id: String, client_secret: String, sandbox: bool) -> Self {
        let base_url = if sandbox {
            "https://api-m.sandbox.paypal.com".to_string()
        } else {
            "https://api-m.paypal.com".to_string()
        };
        
        Self {
            client: Client::new(),
            client_id,
            client_secret,
            base_url,
            access_token: None,
        }
    }
    
    async fn get_access_token(&mut self) -> Result<String, ServiceError> {
        let auth = base64::encode(format!("{}:{}", self.client_id, self.client_secret));
        
        let response = self.client
            .post(&format!("{}/v1/oauth2/token", self.base_url))
            .header("Authorization", format!("Basic {}", auth))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body("grant_type=client_credentials")
            .send()
            .await
            .map_err(|e| ServiceError::ExternalServiceError(format!("PayPal API error: {}", e)))?;
            
        let token_response: PayPalAccessToken = response
            .json()
            .await
            .map_err(|e| ServiceError::ExternalServiceError(format!("PayPal token parse error: {}", e)))?;
            
        self.access_token = Some(token_response.access_token.clone());
        Ok(token_response.access_token)
    }
}

#[async_trait]
impl PaymentProvider for PayPalService {
    async fn create_payment(
        &self,
        order_id: i64,
        amount: Decimal,
        currency: &str,
        return_url: Option<&str>,
        cancel_url: Option<&str>,
    ) -> Result<PaymentResult, ServiceError> {
        let mut service = self.clone(); // 需要实现Clone
        let token = service.get_access_token().await?;
        
        let create_order = CreateOrderRequest {
            intent: "CAPTURE".to_string(),
            purchase_units: vec![PurchaseUnit {
                amount: Amount {
                    currency_code: currency.to_string(),
                    value: amount.to_string(),
                },
                description: format!("Order #{}", order_id),
            }],
            application_context: ApplicationContext {
                return_url: return_url.unwrap_or("https://yoursite.com/success").to_string(),
                cancel_url: cancel_url.unwrap_or("https://yoursite.com/cancel").to_string(),
                brand_name: "RSWS".to_string(),
                user_action: "PAY_NOW".to_string(),
            },
        };
        
        let response = service.client
            .post(&format!("{}/v2/checkout/orders", service.base_url))
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(&create_order)
            .send()
            .await
            .map_err(|e| ServiceError::ExternalServiceError(format!("PayPal create order error: {}", e)))?;
            
        let order_response: PayPalOrderResponse = response
            .json()
            .await
            .map_err(|e| ServiceError::ExternalServiceError(format!("PayPal response parse error: {}", e)))?;
            
        let payment_url = order_response.links
            .iter()
            .find(|link| link.rel == "approve")
            .map(|link| link.href.clone());
            
        Ok(PaymentResult {
            payment_id: order_response.id,
            payment_url,
            qr_code: None, // PayPal通常不使用二维码
            status: TransactionStatus::Pending,
        })
    }
    
    async fn verify_payment(&self, payment_id: &str) -> Result<PaymentVerification, ServiceError> {
        let mut service = self.clone();
        let token = service.get_access_token().await?;
        
        let response = service.client
            .get(&format!("{}/v2/checkout/orders/{}", service.base_url, payment_id))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .map_err(|e| ServiceError::ExternalServiceError(format!("PayPal verify error: {}", e)))?;
            
        let order: PayPalOrderResponse = response
            .json()
            .await
            .map_err(|e| ServiceError::ExternalServiceError(format!("PayPal verify parse error: {}", e)))?;
            
        let status = match order.status.as_str() {
            "COMPLETED" => TransactionStatus::Completed,
            "APPROVED" => TransactionStatus::Processing,
            "CREATED" | "SAVED" => TransactionStatus::Pending,
            _ => TransactionStatus::Failed,
        };
        
        Ok(PaymentVerification {
            payment_id: order.id,
            status,
            amount: Decimal::new(0, 0), // 需要从订单详情获取
            currency: "USD".to_string(),
            external_transaction_id: Some(payment_id.to_string()),
        })
    }
    
    async fn refund_payment(
        &self,
        payment_id: &str,
        amount: Decimal,
    ) -> Result<RefundResult, ServiceError> {
        // 实现退款逻辑
        todo!("Implement PayPal refund")
    }
}