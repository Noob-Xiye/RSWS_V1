use std::sync::Arc;
use rsws_model::user::password::{ChangePasswordRequest, ChangePasswordResponse};
use rsws_service::user_service::UserService;
use salvo::prelude::*;

pub struct UserPasswordHandler {
    user_service: Arc<UserService>,
}

impl UserPasswordHandler {
    pub fn new(user_service: Arc<UserService>) -> Self {
        Self { user_service }
    }
    
    #[handler]
    pub async fn change_password(&self, req: &mut Request, res: &mut Response) {
        // 从请求中获取用户ID（通过JWT或API Key认证）
        let user_id = match req.ext::<i64>("user_id") {
            Some(id) => *id,
            None => {
                res.status_code(StatusCode::UNAUTHORIZED);
                res.render(Json(ChangePasswordResponse {
                    success: false,
                    message: "未授权".to_string(),
                }));
                return;
            }
        };
        
        // 解析请求体
        let change_req = match req.parse_json::<ChangePasswordRequest>().await {
            Ok(req) => req,
            Err(_) => {
                res.status_code(StatusCode::BAD_REQUEST);
                res.render(Json(ChangePasswordResponse {
                    success: false,
                    message: "无效的请求格式".to_string(),
                }));
                return;
            }
        };
        
        // 调用服务层方法修改密码
        match self.user_service.change_password(user_id, change_req).await {
            Ok(_) => {
                res.render(Json(ChangePasswordResponse {
                    success: true,
                    message: "密码修改成功".to_string(),
                }));
            },
            Err(e) => {
                res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                res.render(Json(ChangePasswordResponse {
                    success: false,
                    message: format!("密码修改失败: {}", e),
                }));
            }
        }
    }
}