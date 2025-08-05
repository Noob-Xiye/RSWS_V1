use std::sync::Arc;
use rsws_model::user::email::{SendEmailChangeCodeRequest, SendEmailChangeCodeResponse, VerifyEmailChangeRequest, VerifyEmailChangeResponse};
use rsws_service::user_service::UserService;
use salvo::prelude::*;

pub struct UserEmailHandler {
    user_service: Arc<UserService>,
}

impl UserEmailHandler {
    pub fn new(user_service: Arc<UserService>) -> Self {
        Self { user_service }
    }
    
    #[handler]
    pub async fn send_email_change_code(&self, req: &mut Request, res: &mut Response) {
        // 从请求中获取用户ID（通过JWT或API Key认证）
        let user_id = match req.ext::<i64>("user_id") {
            Some(id) => *id,
            None => {
                res.status_code(StatusCode::UNAUTHORIZED);
                res.render(Json(SendEmailChangeCodeResponse {
                    success: false,
                    message: "未授权".to_string(),
                    expires_in: 0,
                }));
                return;
            }
        };
        
        // 解析请求体
        let send_req = match req.parse_json::<SendEmailChangeCodeRequest>().await {
            Ok(req) => req,
            Err(_) => {
                res.status_code(StatusCode::BAD_REQUEST);
                res.render(Json(SendEmailChangeCodeResponse {
                    success: false,
                    message: "无效的请求格式".to_string(),
                    expires_in: 0,
                }));
                return;
            }
        };
        
        // 调用服务层方法发送邮箱修改验证码
        match self.user_service.send_email_change_code(user_id, send_req).await {
            Ok(response) => {
                res.render(Json(response));
            },
            Err(e) => {
                res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                res.render(Json(SendEmailChangeCodeResponse {
                    success: false,
                    message: format!("发送验证码失败: {}", e),
                    expires_in: 0,
                }));
            }
        }
    }
    
    #[handler]
    pub async fn verify_email_change(&self, req: &mut Request, res: &mut Response) {
        // 从请求中获取用户ID（通过JWT或API Key认证）
        let user_id = match req.ext::<i64>("user_id") {
            Some(id) => *id,
            None => {
                res.status_code(StatusCode::UNAUTHORIZED);
                res.render(Json(VerifyEmailChangeResponse {
                    success: false,
                    message: "未授权".to_string(),
                }));
                return;
            }
        };
        
        // 解析请求体
        let verify_req = match req.parse_json::<VerifyEmailChangeRequest>().await {
            Ok(req) => req,
            Err(_) => {
                res.status_code(StatusCode::BAD_REQUEST);
                res.render(Json(VerifyEmailChangeResponse {
                    success: false,
                    message: "无效的请求格式".to_string(),
                }));
                return;
            }
        };
        
        // 调用服务层方法验证邮箱修改
        match self.user_service.verify_email_change(user_id, verify_req).await {
            Ok(_) => {
                res.render(Json(VerifyEmailChangeResponse {
                    success: true,
                    message: "邮箱修改成功".to_string(),
                }));
            },
            Err(e) => {
                res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                res.render(Json(VerifyEmailChangeResponse {
                    success: false,
                    message: format!("邮箱修改失败: {}", e),
                }));
            }
        }
    }
}