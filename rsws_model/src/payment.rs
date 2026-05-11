//! 支付模型

use chrono::{DateTime, Utc};
use salvo_oapi::ToSchema;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// ==================== 订单 ====================

/// 订单状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "order_status", rename_all = "lowercase")]
#[derive(Default)]
pub enum OrderStatus {
    #[default]
    Pending,
    Paid,
    Completed,
    Cancelled,
    Refunded,
}

/// 订单
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Order {
    pub id: i64,
    pub user_id: i64,
    pub resource_id: i64,
    pub amount: i64,
    pub status: String,
    pub payment_method: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub expired_at: Option<DateTime<Utc>>,
}

/// 订单详情（包含资源信息）
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct OrderDetail {
    pub id: i64,
    pub user_id: i64,
    pub resource_id: i64,
    pub amount: i64,
    pub status: String,
    pub payment_method: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub expired_at: Option<DateTime<Utc>>,
    /// 资源标题
    pub resource_title: Option<String>,
}

/// 创建订单请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOrderRequest {
    pub resource_id: i64,
    pub payment_method: String,
}

// ==================== 支付交易 ====================

/// 交易状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "transaction_status", rename_all = "lowercase")]
#[derive(Default)]
pub enum TransactionStatus {
    #[default]
    Pending,
    Completed,
    Failed,
    Cancelled,
    Refunded,
}

/// 支付交易
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PaymentTransaction {
    pub id: i64,
    pub order_id: i64,
    pub user_id: i64,
    pub amount: i64,
    pub currency: String,
    pub payment_method: String,
    pub provider_transaction_id: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

// ==================== 用户支付配置 ====================

/// 用户支付配置
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserPaymentConfig {
    pub id: i64,
    pub user_id: i64,
    pub payment_method: String,
    pub account_address: String,
    pub account_name: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 创建用户支付配置请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserPaymentConfigRequest {
    pub payment_method: String,
    pub account_address: String,
    pub account_name: Option<String>,
}

// ==================== 资源支付配置 ====================

/// 资源支付配置
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ResourcePaymentConfig {
    pub id: i64,
    pub resource_id: i64,
    pub user_payment_config_id: i64,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

// ==================== 支付方式 ====================

/// 支付方式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentMethod {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    pub enabled: bool,
    pub min_amount: Option<i64>,
    pub max_amount: Option<i64>,
    pub fee_rate: Option<i64>,
    pub description: Option<String>,
}

// ==================== 响应 ====================

/// 支付订单响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayOrderResponse {
    pub success: bool,
    pub message: String,
    pub payment_url: Option<String>,
    pub payment_id: Option<String>,
    pub qr_code: Option<String>,
}

/// 验证支付响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyPaymentResponse {
    pub success: bool,
    pub status: String,
    pub message: String,
    pub order_id: Option<i64>,
}

// ==================== PayPal 配置 ====================

/// PayPal 配置
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
    pub min_amount: rust_decimal::Decimal,
    pub max_amount: rust_decimal::Decimal,
    pub fee_rate: rust_decimal::Decimal,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// PayPal 配置更新请求（管理员用）
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdatePayPalConfigRequest {
    pub client_id: Option<String>,
    pub client_secret_encrypted: Option<String>,
    pub sandbox: Option<bool>,
    pub webhook_id: Option<String>,
    pub webhook_secret_encrypted: Option<String>,
    pub base_url: Option<String>,
    pub return_url: Option<String>,
    pub cancel_url: Option<String>,
    pub brand_name: Option<String>,
    pub min_amount: Option<rust_decimal::Decimal>,
    pub max_amount: Option<rust_decimal::Decimal>,
    pub fee_rate: Option<rust_decimal::Decimal>,
    pub is_active: Option<bool>,
}

// ==================== 单元测试 ====================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order_status_default() {
        let status = OrderStatus::default();
        assert_eq!(status, OrderStatus::Pending);
    }

    #[test]
    fn test_transaction_status_default() {
        let status = TransactionStatus::default();
        assert_eq!(status, TransactionStatus::Pending);
    }

    #[test]
    fn test_create_order_request() {
        let req = CreateOrderRequest {
            resource_id: 1,
            payment_method: "paypal".to_string(),
        };

        assert_eq!(req.resource_id, 1);
        assert_eq!(req.payment_method, "paypal");
    }
}
