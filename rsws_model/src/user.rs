use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, schemars::JsonSchema)]
pub struct User {
    pub id: i64, // 雪花ID
    pub email: String,
    pub password_hash: String,
    pub username: Option<String>,
    pub avatar_url: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, schemars::JsonSchema)]
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

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct RegisterRequest {
    pub nickname: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SendVerificationCodeRequest {
    pub email: String,
    pub code_type: String, // "registration"
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct VerifyCodeRequest {
    pub email: String,
    pub code: String,
    pub nickname: String,
    pub password: String,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct RegisterResponse {
    pub success: bool,
    pub message: String,
    pub user_id: Option<i64>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct SendCodeResponse {
    pub success: bool,
    pub message: String,
    pub expires_in: i64, // 过期时间（秒）
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
    pub device_info: Option<serde_json::Value>,
    pub user_agent: Option<String>,
}

// 添加传统登录响应结构
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct TraditionalLoginResponse {
    pub success: bool,
    pub message: String,
    pub user_info: Option<UserInfo>,
    pub session_data: Option<SessionData>,
}

// 添加用户资料相关结构体
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct UserProfile {
    pub id: i64,
    pub nickname: String,
    pub email: String,
    pub avatar: Option<String>,
    pub is_email_verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct UpdateProfileRequest {
    pub nickname: Option<String>,
    pub avatar: Option<String>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct ProfileCompletionResponse {
    pub completion_percentage: f32,
    pub missing_fields: Vec<String>,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SendEmailChangeCodeRequest {
    pub new_email: String,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct SendEmailChangeCodeResponse {
    pub success: bool,
    pub message: String,
    pub expires_in: i64,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct VerifyEmailChangeRequest {
    pub new_email: String,
    pub code: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct LogoutRequest {
    pub session_token: Option<String>,
    pub logout_all_devices: bool,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct LogoutResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SendLoginCodeRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct VerifyLoginCodeRequest {
    pub email: String,
    pub password: String,
    pub code: String,
    pub device_info: Option<serde_json::Value>,
    pub user_agent: Option<String>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct LoginResponse {
    pub success: bool,
    pub message: String,
    pub user_info: Option<UserInfo>,
    pub session_data: Option<SessionData>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct UserInfo {
    pub id: i64,
    pub nickname: String,
    pub email: String,
    pub avatar: Option<String>,
    pub is_email_verified: bool,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct SessionData {
    pub session_token: String,
    pub api_key: String,
    pub api_secret: String,
    pub expires_at: DateTime<Utc>,
    pub signature_info: SignatureInfo,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct SignatureInfo {
    pub algorithm: String,        // "HMAC-SHA256"
    pub timestamp_header: String, // "X-Timestamp"
    pub signature_header: String, // "X-Signature"
    pub api_key_header: String,   // "X-API-Key"
}
