use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PaymentLog {
    pub id: i64,
    pub transaction_id: Option<String>,
    pub order_id: Option<i64>,
    pub user_id: i64,
    pub payment_method: String,
    pub amount: Decimal,
    pub currency: String,
    pub status: String,
    pub provider_response: Option<serde_json::Value>,
    pub gateway_transaction_id: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePaymentLogRequest {
    pub transaction_id: Option<String>,
    pub order_id: Option<i64>,
    pub user_id: i64,
    pub payment_method: String,
    pub amount: Decimal,
    pub currency: String,
    pub status: String,
    pub provider_response: Option<serde_json::Value>,
    pub gateway_transaction_id: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}
