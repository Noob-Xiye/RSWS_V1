use salvo::prelude::*;
use std::sync::Arc;
use rsws_service::{AuthService, AdminService};
use rsws_model::response::ApiResponse;
use std::collections::BTreeMap;
use chrono::Utc;

pub struct UnifiedAuthMiddleware {
    auth_service: Arc<AuthService>,
    admin_service: Arc<AdminService>,
}

impl UnifiedAuthMiddleware {
    pub fn new(auth_service: Arc<AuthService>, admin_service: Arc<AdminService>) -> Self {
        Self {
            auth_service,
            admin_service,
        }
    }
}

#[async_trait]
impl Handler for UnifiedAuthMiddleware {
    async fn handle(
        &self,
        req: &mut Request,
        depot: &mut Depot,
        res: &mut Response,
        ctrl: &mut FlowCtrl,
    ) {
        // 获取请求头
        let api_key = req.headers().get("X-API-Key")
            .and_then(|v| v.to_str().ok());
        let timestamp = req.headers().get("X-Timestamp")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<i64>().ok());
        let nonce = req.headers().get("X-Nonce")
            .and_then(|v| v.to_str().ok());
        let signature = req.headers().get("X-Signature")
            .and_then(|v| v.to_str().ok());
        
        if let (Some(api_key), Some(timestamp), Some(nonce), Some(signature)) = 
            (api_key, timestamp, nonce, signature) {
            
            let method = req.method().as_str();
            let path = req.uri().path();
            
            // 获取请求体
            let body = if method == "POST" || method == "PUT" || method == "PATCH" {
                req.payload().await.ok()
                    .and_then(|bytes| String::from_utf8(bytes.to_vec()).ok())
            } else {
                None
            };
            
            // 获取查询参数
            let query_params: BTreeMap<String, String> = req.queries()
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            let query_params = if query_params.is_empty() { None } else { Some(&query_params) };
            
            // 判断是管理员API还是用户API
            let is_admin_api = path.starts_with("/api/admin/");
            
            let validation_result = if is_admin_api {
                // 管理员API验证
                self.admin_service.validate_admin_api_request(
                    method,
                    path,
                    api_key,
                    timestamp,
                    nonce,
                    signature,
                    body.as_deref(),
                    query_params,
                ).await
            } else {
                // 用户API验证
                self.auth_service.validate_api_request(
                    method,
                    path,
                    api_key,
                    timestamp,
                    nonce,
                    signature,
                    body.as_deref(),
                    query_params,
                ).await
            };
            
            match validation_result {
                Ok(validation) => {
                    if validation.is_valid {
                        if is_admin_api {
                            if let Some(admin_session) = validation.admin_session {
                                depot.insert("admin_id", admin_session.admin_id);
                                depot.insert("admin_session", admin_session);
                                req.headers_mut().insert("X-Admin-ID", admin_session.admin_id.to_string().parse().unwrap());
                            }
                        } else {
                            if let Some(user_session) = validation.user_session {
                                depot.insert("user_id", user_session.user_id);
                                depot.insert("user_session", user_session);
                                req.headers_mut().insert("X-User-ID", user_session.user_id.to_string().parse().unwrap());
                            }
                        }
                    } else {
                        res.status_code(StatusCode::UNAUTHORIZED);
                        res.render(Json(ApiResponse::unauthorized(
                            &validation.error_message.unwrap_or("Authentication failed".to_string())
                        )));
                        ctrl.skip_rest();
                        return;
                    }
                }
                Err(e) => {
                    res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                    res.render(Json(ApiResponse::internal_error(&e.to_string())));
                    ctrl.skip_rest();
                    return;
                }
            }
        } else {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(ApiResponse::bad_request("Missing required authentication headers")));
            ctrl.skip_rest();
            return;
        }
    }
}