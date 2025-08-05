use std::sync::Arc;
use rsws_model::user::avatar::UploadAvatarResponse;
use rsws_service::user_service::UserService;
use salvo::prelude::*;
use salvo::http::form::FilePart;

pub struct UserAvatarHandler {
    user_service: Arc<UserService>,
}

impl UserAvatarHandler {
    pub fn new(user_service: Arc<UserService>) -> Self {
        Self { user_service }
    }
    
    #[handler]
    pub async fn upload_avatar(&self, req: &mut Request, res: &mut Response) {
        // 从请求中获取用户ID（通过JWT或API Key认证）
        let user_id = match req.ext::<i64>("user_id") {
            Some(id) => *id,
            None => {
                res.status_code(StatusCode::UNAUTHORIZED);
                res.render(Json(UploadAvatarResponse {
                    success: false,
                    message: "未授权".to_string(),
                    avatar_url: None,
                }));
                return;
            }
        };
        
        // 获取上传的文件
        let file = match req.file("avatar").await {
            Some(file) => file,
            None => {
                res.status_code(StatusCode::BAD_REQUEST);
                res.render(Json(UploadAvatarResponse {
                    success: false,
                    message: "未找到上传的文件".to_string(),
                    avatar_url: None,
                }));
                return;
            }
        };
        
        // 读取文件内容
        let file_data = match file.file_data().await {
            Some(data) => data,
            None => {
                res.status_code(StatusCode::BAD_REQUEST);
                res.render(Json(UploadAvatarResponse {
                    success: false,
                    message: "无法读取文件内容".to_string(),
                    avatar_url: None,
                }));
                return;
            }
        };
        
        // 获取文件名和内容类型
        let file_name = file.name().unwrap_or("unknown.jpg").to_string();
        let content_type = file.content_type().unwrap_or("application/octet-stream").to_string();
        
        // 调用服务层方法上传头像
        match self.user_service.upload_avatar(user_id, file_data, file_name, content_type).await {
            Ok(avatar_url) => {
                res.render(Json(UploadAvatarResponse {
                    success: true,
                    message: "头像上传成功".to_string(),
                    avatar_url: Some(avatar_url),
                }));
            },
            Err(e) => {
                res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                res.render(Json(UploadAvatarResponse {
                    success: false,
                    message: format!("头像上传失败: {}", e),
                    avatar_url: None,
                }));
            }
        }
    }
}