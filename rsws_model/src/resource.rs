use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Resource {
    pub id: i64,
    pub user_id: i64,
    pub title: String,
    pub description: Option<String>,
    pub price: Decimal,
    pub category_id: Option<i64>,
    pub file_url: Option<String>,
    pub thumbnail_url: Option<String>,
    pub is_active: bool,
    pub detail_description: Option<String>,
    pub specifications: Option<serde_json::Value>,
    pub usage_guide: Option<String>,
    pub precautions: Option<String>,
    pub display_images: Option<Vec<String>>,
    // 新增字段
    pub provider_type: String, // "admin" 或 "user"
    pub provider_id: Option<i64>,
    pub commission_rate: Decimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// 删除重复的结构体定义，统一使用 payment.rs 中的定义
// CreateUserPaymentConfigRequest 和 ResourcePaymentConfig 保持不变

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserPaymentConfigRequest {
    pub payment_method: String,
    pub account_address: String,
    pub account_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ResourcePaymentConfig {
    pub id: i64,
    pub resource_id: i64,
    pub user_payment_config_id: i64,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}
