//! 用户模型

use chrono::{DateTime, Utc};
use salvo_oapi::ToSchema;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 用户
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct User {
    pub id: i64,
    pub email: String,
    pub password_hash: String,
    pub username: String, // 登录用用户名（唯一）
    pub nickname: String, // 显示名称（可修改）
    pub avatar_url: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 邮箱验证码
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct EmailVerificationCode {
    pub id: i64,
    pub email: String,
    pub code: String,
    pub code_type: String,
    pub expires_at: DateTime<Utc>,
    pub is_used: bool,
    pub attempts: i32,
    pub created_at: DateTime<Utc>,
}

/// 修改密码请求
#[derive(Debug, Deserialize, ToSchema)]
pub struct ChangePasswordRequest {
    pub old_password: String,
    pub new_password: String,
}

/// 更新资料请求
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateProfileRequest {
    pub nickname: Option<String>,
    pub avatar_url: Option<String>,
}

/// 注册请求
#[derive(Debug, Deserialize, ToSchema)]
pub struct RegisterRequest {
    pub username: String, // 登录用用户名
    pub nickname: String, // 显示名称
    pub email: String,
    pub password: String,
}

/// 发送验证码请求
#[derive(Debug, Deserialize, ToSchema)]
pub struct SendVerificationCodeRequest {
    pub email: String,
    pub code_type: String,
}

/// 验证码请求
#[derive(Debug, Deserialize, ToSchema)]
pub struct VerifyCodeRequest {
    pub email: String,
    pub code: String,
    pub nickname: String,
    pub password: String,
}

/// 注册响应
#[derive(Debug, Serialize, ToSchema)]
pub struct RegisterResponse {
    pub success: bool,
    pub message: String,
    pub user_id: Option<i64>,
}

/// 发送验证码响应
#[derive(Debug, Serialize, ToSchema)]
pub struct SendCodeResponse {
    pub success: bool,
    pub message: String,
    pub expires_in: i64,
}

/// 登录请求（两种方式）
#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginRequest {
    /// 登录方式: "password" | "code"
    pub login_type: String,
    /// 用户名（password 方式必填）
    pub username: Option<String>,
    /// 密码（password 方式必填）
    pub password: Option<String>,
    /// 邮箱（code 方式必填）
    pub email: Option<String>,
    /// 验证码（code 方式必填）
    pub verification_code: Option<String>,
    pub device_info: Option<serde_json::Value>,
    pub user_agent: Option<String>,
}

/// 登录响应
#[derive(Debug, Serialize, ToSchema)]
pub struct LoginResponse {
    pub success: bool,
    pub message: String,
    pub user_info: Option<UserInfo>,
    pub session_data: Option<SessionData>,
}

/// 用户信息
#[derive(Debug, Serialize, ToSchema)]
pub struct UserInfo {
    pub id: i64,
    pub email: String,
    pub username: String, // 登录用用户名
    pub nickname: String, // 显示名称
    pub avatar_url: Option<String>,
    pub is_active: bool,
}

/// 会话数据
#[derive(Debug, Serialize, ToSchema)]
pub struct SessionData {
    pub api_key: String,
    pub api_secret: String,
    pub expires_at: DateTime<Utc>,
}

/// 管理员视图用户信息（分页列表用）
#[derive(Debug, Serialize, ToSchema, FromRow)]
pub struct AdminUserView {
    pub id: i64,
    pub email: String,
    pub username: String,
    pub avatar_url: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

impl From<User> for AdminUserView {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            email: user.email,
            username: user.username,
            avatar_url: user.avatar_url,
            is_active: user.is_active,
            created_at: user.created_at,
        }
    }
}

// ==================== 单元测试 ====================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_request() {
        let req = RegisterRequest {
            username: "testuser".to_string(),
            nickname: "Test User".to_string(),
            email: "test@example.com".to_string(),
            password: "Password123".to_string(),
        };

        assert_eq!(req.username, "testuser");
        assert_eq!(req.nickname, "Test User");
        assert!(!req.email.is_empty());
    }

    #[test]
    fn test_login_request() {
        // 密码登录
        let req = LoginRequest {
            login_type: "password".to_string(),
            username: Some("testuser".to_string()),
            password: Some("Password123".to_string()),
            email: None,
            verification_code: None,
            device_info: None,
            user_agent: Some("Mozilla/5.0".to_string()),
        };

        assert_eq!(req.login_type, "password");
        assert!(req.username.is_some());

        // 验证码登录
        let req2 = LoginRequest {
            login_type: "code".to_string(),
            username: None,
            password: None,
            email: Some("test@example.com".to_string()),
            verification_code: Some("123456".to_string()),
            device_info: None,
            user_agent: Some("Mozilla/5.0".to_string()),
        };

        assert_eq!(req2.login_type, "code");
        assert!(req2.email.is_some());
    }

    #[test]
    fn test_user_info() {
        let info = UserInfo {
            id: 1,
            email: "test@example.com".to_string(),
            username: "testuser".to_string(),
            nickname: "Test User".to_string(),
            avatar_url: None,
            is_active: true,
        };

        assert_eq!(info.id, 1);
        assert_eq!(info.username, "testuser");
        assert_eq!(info.nickname, "Test User");
        assert!(info.is_active);
    }
}
