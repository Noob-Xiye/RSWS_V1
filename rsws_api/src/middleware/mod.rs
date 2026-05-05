//! 中间件

pub mod auth;

pub use auth::{api_key_auth, admin_auth, rate_limit};

/// 认证上下文（从中间件传递到 handler）
#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user_id: i64,
    pub api_key_id: i64,
    pub permissions: Vec<String>,
    pub rate_limit: i32,
}