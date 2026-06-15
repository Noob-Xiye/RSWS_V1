//! API Key 模型
//!
//! 新设计：api_key 即为签名密钥（Cregis 方案）
//! - api_key: 随机文本，用于计算 MD5 签名，不随请求传输
//! - 请求中传 user_id（公开标识）+ timestamp + nonce + sign

use chrono::{DateTime, Utc};
use salvo_oapi::ToSchema;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// API Key
///
/// api_key 既是记录标识也是签名密钥。
/// 前端持有 api_key 用于签名，后端通过 user_id 查库获取 api_key 验签。
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct ApiKey {
    pub id: i64,
    pub user_id: i64,
    /// 签名密钥（随机文本）。前端持有此值计算签名，但不随请求传输。
    pub api_key: String,
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
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
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
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateApiKeyRequest {
    pub name: String,
    pub permissions: Vec<String>,
    pub rate_limit: Option<i32>,
    pub expires_in_days: Option<i32>,
}

/// API Key 响应（登录/创建时返回给前端）
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ApiKeyResponse {
    pub id: i64,
    pub name: String,
    /// 签名密钥。前端保存此值用于后续 API 调用签名。
    pub api_key: String,
    pub permissions: Vec<String>,
    pub rate_limit: i32,
    pub last_used_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

/// API Key 会话（Redis 存储）
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ApiKeySession {
    pub user_id: i64,
    pub api_key_id: i64,
    pub permissions: Vec<String>,
    pub rate_limit: i32,
    pub last_access: DateTime<Utc>,
}

/// 权限枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
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
