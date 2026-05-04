//! API Key 模型

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// API Key
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ApiKey {
    pub id: i64,
    pub user_id: i64,
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

/// API Key 使用日志
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ApiKeyUsageLog {
    pub id: i64,
    pub api_key_id: i64,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub endpoint: Option<String>,
    pub method: Option<String>,
    pub status_code: Option<i32>,
    pub response_time_ms: Option<i32>,
    pub created_at: DateTime<Utc>,
}

/// 创建 API Key 请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateApiKeyRequest {
    pub name: String,
    pub permissions: Vec<String>,
    pub rate_limit: Option<i32>,
    pub expires_in_days: Option<i32>,
}

/// API Key 响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyResponse {
    pub id: i64,
    pub name: String,
    pub api_key: String,
    pub api_secret: Option<String>,
    pub permissions: Vec<String>,
    pub rate_limit: i32,
    pub last_used_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

/// API Key 会话（Redis 存储）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeySession {
    pub user_id: i64,
    pub api_key_id: i64,
    pub permissions: Vec<String>,
    pub rate_limit: i32,
    pub last_access: DateTime<Utc>,
}

/// 权限枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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

// ==================== 单元测试 ====================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_api_key_request() {
        let req = CreateApiKeyRequest {
            name: "test-key".to_string(),
            permissions: vec!["read".to_string(), "write".to_string()],
            rate_limit: Some(1000),
            expires_in_days: Some(30),
        };

        assert_eq!(req.name, "test-key");
        assert_eq!(req.permissions.len(), 2);
    }

    #[test]
    fn test_permission_serialize() {
        let perm = Permission::Read;
        let json = serde_json::to_string(&perm).unwrap();
        assert_eq!(json, "\"read\"");
    }
}
