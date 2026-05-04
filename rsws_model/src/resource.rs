//! 资源模型

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 资源
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Resource {
    pub id: i64,
    pub user_id: i64,
    pub title: String,
    pub description: Option<String>,
    pub price: i64,
    pub category_id: Option<i64>,
    pub file_url: Option<String>,
    pub thumbnail_url: Option<String>,
    pub is_active: bool,
    pub detail_description: Option<String>,
    pub specifications: Option<serde_json::Value>,
    pub usage_guide: Option<String>,
    pub precautions: Option<String>,
    pub display_images: Option<serde_json::Value>,
    pub provider_type: String,
    pub provider_id: Option<i64>,
    pub commission_rate: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 创建资源请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateResourceRequest {
    pub title: String,
    pub description: Option<String>,
    pub price: i64,
    pub category_id: Option<i64>,
    pub file_url: Option<String>,
    pub thumbnail_url: Option<String>,
    pub detail_description: Option<String>,
    pub specifications: Option<serde_json::Value>,
    pub usage_guide: Option<String>,
    pub precautions: Option<String>,
    pub display_images: Option<Vec<String>>,
}

/// 更新资源请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateResourceRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub price: Option<i64>,
    pub category_id: Option<i64>,
    pub file_url: Option<String>,
    pub thumbnail_url: Option<String>,
    pub is_active: Option<bool>,
    pub detail_description: Option<String>,
    pub specifications: Option<serde_json::Value>,
    pub usage_guide: Option<String>,
    pub precautions: Option<String>,
    pub display_images: Option<Vec<String>>,
}

/// 资源列表响应
#[derive(Debug, Clone, Serialize, Deserialize)]
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
            price: 1000,
            category_id: None,
            file_url: None,
            thumbnail_url: None,
            detail_description: None,
            specifications: None,
            usage_guide: None,
            precautions: None,
            display_images: None,
        };

        assert_eq!(req.title, "Test Resource");
        assert_eq!(req.price, 1000);
    }
}
