use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, schemars::JsonSchema)]
pub struct PaymentTransaction {
    pub id: i64,
    pub order_id: i64,
    pub user_id: i64,
    pub payment_method: String,
    pub payment_provider: String, // "paypal", "blockchain", "alipay", "wechat"
    pub external_transaction_id: String,
    pub amount: Decimal,
    pub currency: String,
    pub status: TransactionStatus,
    pub gateway_response: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, schemars::JsonSchema)]
#[sqlx(type_name = "transaction_status", rename_all = "lowercase")]
pub enum TransactionStatus {
    Pending,    // 待处理
    Processing, // 处理中
    Completed,  // 已完成
    Failed,     // 失败
    Cancelled,  // 已取消
    Refunded,   // 已退款
}

// 支付方法配置
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct PaymentMethod {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub enabled: bool,
    pub min_amount: Option<Decimal>,
    pub max_amount: Option<Decimal>,
    pub fee_rate: Option<Decimal>,
    pub description: Option<String>,
}

// 支付验证请求
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct VerifyPaymentRequest {
    pub payment_id: String,
    pub transaction_id: Option<String>,
}

// 支付验证响应
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct VerifyPaymentResponse {
    pub success: bool,
    pub status: TransactionStatus,
    pub message: String,
    pub order_id: Option<i64>,
}
