//! 管理员模型

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 管理员
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Admin {
    pub id: i64,
    pub email: String,
    pub password_hash: String,
    pub username: String,
    pub nickname: String,
    pub avatar_url: Option<String>,
    pub is_active: bool,
    pub role: String,
    pub permissions: serde_json::Value,
    pub last_login_at: Option<DateTime<Utc>>,
    pub last_login_ip: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 管理员 API Key
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AdminApiKey {
    pub id: i64,
    pub admin_id: i64,
    pub name: String,
    pub api_key: String,
    pub permissions: serde_json::Value,
    pub rate_limit: Option<i32>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 创建管理员请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAdminRequest {
    pub email: String,
    pub password: String,
    pub username: String,
    pub role: String,
    pub permissions: Option<Vec<String>>,
}

/// 更新管理员请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAdminRequest {
    pub id: i64,
    pub email: Option<String>,
    pub password: Option<String>,
    pub username: Option<String>,
    pub avatar_url: Option<String>,
    pub is_active: Option<bool>,
    pub role: Option<String>,
    pub permissions: Option<Vec<String>>,
}

/// 管理员登录请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminLoginRequest {
    pub email: String,
    pub password: String,
}

/// 管理员登录响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminLoginResponse {
    pub admin: AdminInfo,
    /// 签名密钥（前端持有用于签名，不随请求传输）
    pub api_key: String,
    pub expires_at: DateTime<Utc>,
}

/// 管理员信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminInfo {
    pub id: i64,
    pub email: String,
    pub username: String,
    pub nickname: String,
    pub avatar_url: Option<String>,
    pub is_active: bool,
    pub role: String,
    pub permissions: Vec<String>,
}

/// 管理员操作日志
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AdminOperationLog {
    pub id: i64,
    pub admin_id: i64,
    pub operation_type: String,
    pub operation_target: Option<String>,
    pub target_id: Option<String>,
    pub operation_content: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// 创建 API Key 请求
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAdminApiKeyRequest {
    pub name: String,
    pub permissions: Vec<String>,
    pub rate_limit: Option<i32>,
    pub expires_in_days: Option<i32>,
}

/// API Key 响应
#[derive(Debug, Serialize, Deserialize)]
pub struct AdminApiKeyResponse {
    pub id: i64,
    pub name: String,
    pub api_key: String,
    pub permissions: Vec<String>,
    pub rate_limit: Option<i32>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

/// Dashboard 统计面板数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStats {
    // 用户统计
    pub total_users: i64,
    pub new_users_30d: i64,
    // 订单统计
    pub total_orders: i64,
    pub completed_orders: i64,
    // 收入统计（单位：分，前端除以100转元）
    pub total_revenue: i64,
    pub revenue_30d: i64,
    // 资源统计
    pub total_resources: i64,
    pub active_resources: i64,
    pub new_resources_30d: i64,
    // 过去30天订单趋势（按天分组）
    pub orders_trend: Vec<DailyOrderCount>,
}

/// 单日订单统计
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DailyOrderCount {
    pub date: String, // YYYY-MM-DD
    pub count: i64,
}

// ==================== 单元测试 ====================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_admin_request() {
        let req = CreateAdminRequest {
            email: "admin@example.com".to_string(),
            password: "Password123".to_string(),
            username: "admin".to_string(),
            role: "super_admin".to_string(),
            permissions: Some(vec!["all".to_string()]),
        };

        assert_eq!(req.email, "admin@example.com");
        assert_eq!(req.role, "super_admin");
    }

    #[test]
    fn test_admin_login_request() {
        let req = AdminLoginRequest {
            email: "admin@example.com".to_string(),
            password: "Password123".to_string(),
        };

        assert!(!req.email.is_empty());
        assert!(!req.password.is_empty());
    }
}
