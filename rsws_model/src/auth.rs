use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::collections::HashMap;
use std::net::IpAddr;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserSession {
    pub id: i64,
    pub user_id: i64,
    pub session_token: String,
    pub api_key: String,
    pub api_secret: String,
    pub device_info: Option<serde_json::Value>,
    pub ip_address: Option<IpAddr>,
    pub user_agent: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    pub device_info: Option<DeviceInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub device_type: String, // mobile, desktop, tablet
    pub os: String,
    pub browser: Option<String>,
    pub app_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub session_token: String,
    pub api_key: String,
    pub api_secret: String,
    pub expires_at: DateTime<Utc>,
    pub user_info: UserInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: i64,
    pub username: String,
    pub email: Option<String>,
    pub nickname: Option<String>,
    pub avatar: Option<String>,
    pub permissions: Vec<String>,
}

// API请求签名结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiRequest {
    pub api_key: String,
    pub timestamp: i64,
    pub nonce: String,
    pub signature: String,
    pub data: Option<serde_json::Value>,
}

// 签名验证结果
#[derive(Debug, Clone)]
pub struct SignatureValidation {
    pub is_valid: bool,
    pub user_session: Option<UserSession>,
    pub error_message: Option<String>,
}
