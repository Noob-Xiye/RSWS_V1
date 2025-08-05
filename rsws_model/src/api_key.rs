use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::net::IpAddr;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ApiKey {
    pub id: i32,
    pub user_id: i32,
    pub api_key: String,
    pub api_secret: String,
    pub name: String,
    pub permissions: serde_json::Value,
    pub rate_limit: i32,
    pub last_used_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ApiKeyUsageLog {
    pub id: i32,
    pub api_key_id: i32,
    pub ip_address: Option<IpAddr>,
    pub user_agent: Option<String>,
    pub endpoint: Option<String>,
    pub method: Option<String>,
    pub status_code: Option<i32>,
    pub response_time_ms: Option<i32>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateApiKeyRequest {
    pub name: String,
    pub permissions: Vec<String>,
    pub rate_limit: Option<i32>,
    pub expires_in_days: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyResponse {
    pub id: i32,
    pub name: String,
    pub api_key: String,
    pub api_secret: Option<String>, // 只在创建时返回
    pub permissions: Vec<String>,
    pub rate_limit: i32,
    pub last_used_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

// Redis中存储的会话信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeySession {
    pub user_id: i32,
    pub api_key_id: i32,
    pub permissions: Vec<String>,
    pub rate_limit: i32,
    pub last_access: DateTime<Utc>,
}

// 权限枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Permission {
    #[serde(rename = "read")]
    Read,
    #[serde(rename = "write")]
    Write,
    #[serde(rename = "delete")]
    Delete,
    #[serde(rename = "admin")]
    Admin,
    #[serde(rename = "config")]
    Config,
    #[serde(rename = "user_management")]
    UserManagement,
}
