//! 认证辅助 Handler Trait
//!
//! 提供需要认证的 handler 的通用认证检查逻辑
//! 
//! 在 handler 中简化为 require_user_id 检查的样板代码

use salvo::prelude::*;
use crate::error_code::ErrorCode;
use crate::response::ApiResponse;

/// 认证辅助 Trait
/// 
/// 在 handler 中简化为 require_user_id 检查的样板代码
/// 
/// # 示例
/// ```ignore
/// use rsws_common::auth_handler::AuthHandler;
///
/// pub async fn get_user(req: &mut Request, depot: &mut Depot, res: &mut Response) {
///     let user_id = match res.auth_require_user_id(depot) {
///         Some(id) => id,
///         None => return,
///     };
///     // ... 业务逻辑
/// }
/// ```
pub trait AuthHandler {
    /// 尝试获取用户ID，失败则自动发送 401 响应
    fn auth_require_user_id(&mut self, depot: &Depot) -> Option<i64>;
    
    /// 发送未认证错误响应
    fn auth_unauthorized(&mut self, msg: impl Into<String>);
}

impl AuthHandler for Response {
    fn auth_require_user_id(&mut self, depot: &Depot) -> Option<i64> {
        // 从 depot 获取 user_id
        let user_id = depot.get::<i64>("user_id").ok().copied();
        
        if user_id.is_none() {
            self.auth_unauthorized("Authentication required");
        }
        
        user_id
    }
    
    fn auth_unauthorized(&mut self, msg: impl Into<String>) {
        self.status_code(salvo::http::StatusCode::UNAUTHORIZED);
        self.render(Json(ApiResponse::<()>::error_with_message(
            ErrorCode::AUTH_MISSING_CREDENTIALS,
            msg,
        )));
    }
}