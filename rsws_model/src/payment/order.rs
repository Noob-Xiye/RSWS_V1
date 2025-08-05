use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, schemars::JsonSchema)]
pub struct Order {
    pub id: i64,
    pub user_id: i64,
    pub resource_id: i64,
    pub amount: Decimal,
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

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, schemars::JsonSchema)]
#[sqlx(type_name = "order_status", rename_all = "lowercase")]
pub enum OrderStatus {
    Pending,   // 待支付
    Paid,      // 已支付
    Completed, // 已完成
    Cancelled, // 已取消
    Refunded,  // 已退款
    Failed,    // 支付失败
}

// 创建订单请求
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateOrderRequest {
    pub resource_id: i64,
    pub payment_method: Option<String>,
}

// 订单响应
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct OrderResponse {
    pub id: i64,
    pub user_id: i64,
    pub resource_id: i64,
    pub resource_title: String,
    pub amount: Decimal,
    pub status: OrderStatus,
    pub payment_method: Option<String>,
    pub created_at: DateTime<Utc>,
    pub expired_at: Option<DateTime<Utc>>,
}

// 支付订单请求
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct PayOrderRequest {
    pub payment_method: String, // "paypal", "usdt", "alipay", "wechat"
    pub return_url: Option<String>,
    pub cancel_url: Option<String>,
}

// 支付订单响应
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct PayOrderResponse {
    pub success: bool,
    pub message: String,
    pub payment_url: Option<String>,
    pub payment_id: Option<String>,
    pub qr_code: Option<String>, // 用于二维码支付
}

// 订单列表查询请求
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct OrderListRequest {
    pub status: Option<OrderStatus>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}
