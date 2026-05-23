//! 资源模型

use chrono::{DateTime, Utc};
use salvo_oapi::ToSchema;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 资源归属类型常量
pub const OWNER_TYPE_USER: &str = "user";
pub const OWNER_TYPE_PLATFORM: &str = "platform";

/// 资源
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Resource {
    pub id: i64,
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
    pub display_images: Option<serde_json::Value>,
    pub owner_type: String,
    pub provider_id: Option<i64>,
    pub supported_os: Option<serde_json::Value>,
    pub commission_rate: i64,
    pub download_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 创建资源请求
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateResourceRequest {
    pub title: String,
    pub description: Option<String>,
    pub price: Decimal,
    pub category_id: Option<i64>,
    pub file_url: Option<String>,
    pub thumbnail_url: Option<String>,
    pub detail_description: Option<String>,
    pub specifications: Option<serde_json::Value>,
    pub usage_guide: Option<String>,
    pub precautions: Option<String>,
    pub display_images: Option<Vec<String>>,
    pub supported_os: Option<Vec<String>>,
}

/// 更新资源请求
#[derive(Debug, Clone, Serialize, Deserialize, Default, ToSchema)]
pub struct UpdateResourceRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub price: Option<Decimal>,
    pub category_id: Option<i64>,
    pub file_url: Option<String>,
    pub thumbnail_url: Option<String>,
    pub is_active: Option<bool>,
    pub detail_description: Option<String>,
    pub specifications: Option<serde_json::Value>,
    pub usage_guide: Option<String>,
    pub precautions: Option<String>,
    pub display_images: Option<Vec<String>>,
    pub supported_os: Option<Vec<String>>,
}

/// 资源列表响应
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ResourceListResponse {
    pub items: Vec<Resource>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

// ==================== 单元测试 ====================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_resource_request() {
        let req = CreateResourceRequest {
            title: "Test Resource".to_string(),
            description: Some("A test resource".to_string()),
            price: Decimal::new(1000, 0),
            category_id: None,
            file_url: None,
            thumbnail_url: None,
            detail_description: None,
            specifications: None,
            usage_guide: None,
            precautions: None,
            display_images: None,
            supported_os: None,
        };

        assert_eq!(req.title, "Test Resource");
        assert_eq!(req.price, Decimal::new(1000, 0));
    }
}
