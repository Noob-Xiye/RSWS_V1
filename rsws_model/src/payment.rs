use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

// 统一的用户支付配置结构体
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserPaymentConfig {
    pub id: i64,
    pub user_id: i64,
    pub payment_method: String,  // paypal, usdt_tron, usdt_eth
    pub account_address: String, // PayPal邮箱或USDT地址
    pub account_name: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserPaymentConfigRequest {
    pub payment_method: String,
    pub account_address: String,
    pub account_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPaymentConfigResponse {
    pub id: i64,
    pub payment_method: String,
    pub account_address: String,
    pub account_name: Option<String>,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPaymentConfigStatsResponse {
    pub paypal_count: i64,
    pub usdt_tron_count: i64,
    pub usdt_eth_count: i64,
    pub total_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetDefaultPaymentConfigRequest {
    pub config_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ResourcePaymentConfig {
    pub id: i64,
    pub resource_id: i64,
    pub user_payment_config_id: i64,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

// 支付相关的其他结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentMethod {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    pub enabled: bool,
    pub min_amount: Option<rust_decimal::Decimal>,
    pub max_amount: Option<rust_decimal::Decimal>,
    pub fee_rate: Option<rust_decimal::Decimal>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayOrderResponse {
    pub success: bool,
    pub message: String,
    pub payment_url: Option<String>,
    pub payment_id: Option<String>,
    pub qr_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyPaymentResponse {
    pub success: bool,
    pub status: TransactionStatus,
    pub message: String,
    pub order_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransactionStatus {
    Pending,
    Completed,
    Failed,
    Cancelled,
    Refunded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentTransaction {
    pub id: i64,
    pub order_id: i64,
    pub user_id: i64,
    pub payment_method: String,
    pub payment_provider: String,
    pub external_transaction_id: String,
    pub amount: rust_decimal::Decimal,
    pub currency: String,
    pub status: TransactionStatus,
    pub gateway_response: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OrderStatus {
    Pending,
    Completed,
    Cancelled,
    Refunded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: i64,
    pub user_id: i64,
    pub resource_id: i64,
    pub amount: rust_decimal::Decimal,
    pub status: OrderStatus,
    pub payment_method: Option<String>,
    pub payment_id: Option<String>,
    pub transaction_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub expired_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
}
