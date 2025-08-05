pub struct UserSecurityHandler {
    user_service: Arc<UserService>,
}

impl UserSecurityHandler {
    pub fn new(user_service: Arc<UserService>) -> Self {
        Self { user_service }
    }
    
    #[handler]
    pub async fn change_password(&self, req: &mut Request, res: &mut Response) {
        let user_id = req.ext::<i64>("user_id")
            .ok_or_else(|| {
                res.status_code(StatusCode::UNAUTHORIZED);
                res.render(Json(serde_json::json!({
                    "error": "Unauthorized"
                })));
                return;
            });
            
        let change_req: ChangePasswordRequest = match req.parse_json().await {
            Ok(req) => req,
            Err(_) => {
                res.status_code(StatusCode::BAD_REQUEST);
                res.render(Json(serde_json::json!({
                    "error": "Invalid request format"
                })));
                return;
            }
        };
        
        match self.user_service.change_password(*user_id, change_req).await {
            Ok(_) => {
                res.render(Json(serde_json::json!({
                    "message": "Password changed successfully"
                })));
            },
            Err(e) => {
                res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                res.render(Json(serde_json::json!({
                    "error": format!("Failed to change password: {}", e)
                })));
            }
        }
    }
}