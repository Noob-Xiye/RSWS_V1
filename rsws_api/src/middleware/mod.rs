//! 中间件

pub mod auth;
pub mod request_id;

pub use auth::{api_key_auth, rate_limit, require_admin};
pub use request_id::{request_id_middleware, get_request_id, REQUEST_ID_HEADER};

/// 认证上下文（从中间件传递到 handler）
#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user_id: i64,
    pub api_key_id: i64,
    pub is_admin: bool,
    pub permissions: Vec<String>,
    pub rate_limit: i32,
}
