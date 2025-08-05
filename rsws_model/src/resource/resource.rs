use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize)]
pub struct UploadResourceRequest {
    pub title: String,
    pub description: String,
    pub detail_description: Option<String>, // 详细描述，类似淘宝的商品详情
    pub specifications: Option<serde_json::Value>, // 规格参数，JSON格式
    pub usage_guide: Option<String>,        // 使用指南
    pub precautions: Option<String>,        // 注意事项
    pub display_images: Option<Vec<String>>, // 展示图片URL列表
    pub file_name: String,
    pub file_size: usize,
    pub content_type: Option<String>,
    pub price: f64,
    pub category_id: Option<i64>,
    pub file: salvo::http::form::FilePart,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Resource {
    pub id: i64,
    pub user_id: i64,
    pub title: String,
    pub description: String,
    pub detail_description: Option<String>,
    pub specifications: Option<serde_json::Value>,
    pub usage_guide: Option<String>,
    pub precautions: Option<String>,
    pub display_images: Option<Vec<String>>,
    pub file_name: String,
    pub storage_filename: String,
    pub file_size: i64,
    pub content_type: Option<String>,
    pub price: f64,
    pub category_id: Option<i64>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceResponse {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub detail_description: Option<String>,
    pub specifications: Option<serde_json::Value>,
    pub usage_guide: Option<String>,
    pub precautions: Option<String>,
    pub display_images: Option<Vec<String>>,
    pub file_name: String,
    pub file_size: i64,
    pub content_type: Option<String>,
    pub price: f64,
    pub status: String,
    pub created_at: DateTime<Utc>,
}
