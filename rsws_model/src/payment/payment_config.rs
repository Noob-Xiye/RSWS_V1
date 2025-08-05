use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// PayPal配置
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PayPalConfig {
    pub id: i32,
    pub client_id: String,
    pub client_secret_encrypted: String,
    pub sandbox: bool,
    pub webhook_id: Option<String>,
    pub webhook_secret_encrypted: Option<String>,
    pub base_url: String,
    pub return_url: String,
    pub cancel_url: String,
    pub brand_name: String,
    pub min_amount: Decimal,
    pub max_amount: Decimal,
    pub fee_rate: Decimal,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// 区块链网络配置
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BlockchainConfig {
    pub id: i32,
    pub network: String,      // "tron", "ethereum"
    pub network_name: String, // "TRON", "Ethereum"
    pub api_url: String,
    pub api_key_encrypted: Option<String>,
    pub usdt_contract: String,
    pub wallet_addresses: Vec<String>,
    pub min_confirmations: i32,
    pub min_amount: Decimal,
    pub max_amount: Decimal,
    pub fee_rate: Decimal,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// 支付方式配置
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PaymentMethodConfig {
    pub id: i32,
    pub method_id: String, // "paypal", "usdt_tron", "usdt_eth"
    pub method_name: String,
    pub icon_url: String,
    pub description: Option<String>,
    pub sort_order: i32,
    pub is_active: bool,
    pub config_json: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// 支付配置请求
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct UpdatePayPalConfigRequest {
    pub client_id: String,
    pub client_secret: String,
    pub sandbox: bool,
    pub webhook_id: Option<String>,
    pub webhook_secret: Option<String>,
    pub return_url: String,
    pub cancel_url: String,
    pub brand_name: String,
    pub min_amount: Decimal,
    pub max_amount: Decimal,
    pub fee_rate: Decimal,
    pub is_active: bool,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct UpdateBlockchainConfigRequest {
    pub network: String,
    pub network_name: String,
    pub api_url: String,
    pub api_key: Option<String>,
    pub usdt_contract: String,
    pub wallet_addresses: Vec<String>,
    pub min_confirmations: i32,
    pub min_amount: Decimal,
    pub max_amount: Decimal,
    pub fee_rate: Decimal,
    pub is_active: bool,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct UpdatePaymentMethodRequest {
    pub method_name: String,
    pub icon_url: String,
    pub description: Option<String>,
    pub sort_order: i32,
    pub is_active: bool,
}
