pub struct UserProfileHandler {
    user_service: Arc<UserService>,
}

impl UserProfileHandler {
    pub fn new(user_service: Arc<UserService>) -> Self {
        Self { user_service }
    }
    
    #[handler]
    pub async fn get_profile(&self, req: &mut Request, res: &mut Response) {
        // 从请求中获取用户ID（通过JWT或API Key认证）
        let user_id = req.ext::<i64>("user_id")
            .ok_or_else(|| {
                res.status_code(StatusCode::UNAUTHORIZED);
                res.render(Json(serde_json::json!({
                    "error": "Unauthorized"
                })));
                return;
            });
            
        match self.user_service.get_user_profile(*user_id).await {
            Ok(profile) => {
                res.render(Json(profile));
            },
            Err(e) => {
                res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                res.render(Json(serde_json::json!({
                    "error": format!("Failed to get profile: {}", e)
                })));
            }
        }
    }
    
    #[handler]
    pub async fn update_profile(&self, req: &mut Request, res: &mut Response) {
        let user_id = req.ext::<i64>("user_id")
            .ok_or_else(|| {
                res.status_code(StatusCode::UNAUTHORIZED);
                res.render(Json(serde_json::json!({
                    "error": "Unauthorized"
                })));
                return;
            });
            
        let update_req: UpdateProfileRequest = match req.parse_json().await {
            Ok(req) => req,
            Err(_) => {
                res.status_code(StatusCode::BAD_REQUEST);
                res.render(Json(serde_json::json!({
                    "error": "Invalid request format"
                })));
                return;
            }
        };
        
        match self.user_service.update_user_profile(*user_id, update_req).await {
            Ok(_) => {
                res.render(Json(serde_json::json!({
                    "message": "Profile updated successfully"
                })));
            },
            Err(e) => {
                res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                res.render(Json(serde_json::json!({
                    "error": format!("Failed to update profile: {}", e)
                })));
            }
        }
    }
    
    // 在 UserProfileHandler 中添加以下方法
    
    #[handler]
    pub async fn check_profile_completion(&self, req: &mut Request, res: &mut Response) {
        // 从请求中获取用户ID（通过JWT或API Key认证）
        let user_id = match req.ext::<i64>("user_id") {
            Some(id) => *id,
            None => {
                res.status_code(StatusCode::UNAUTHORIZED);
                res.render(Json(serde_json::json!({
                    "error": "未授权"
                })));
                return;
            }
        };
        
        // 调用服务层方法检查资料完整性
        match self.user_service.check_profile_completion(user_id).await {
            Ok(completion) => {
                res.render(Json(completion));
            },
            Err(e) => {
                res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                res.render(Json(serde_json::json!({
                    "error": format!("检查资料完整性失败: {}", e)
                })));
            }
        }
    }
}