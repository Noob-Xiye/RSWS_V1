use salvo::prelude::*;
use std::sync::Arc;
use rsws_service::api_key_service::ApiKeyService;
use rsws_model::api_key::{ApiKeySession, Permission};
use std::net::IpAddr;
use chrono::Utc;

pub struct ApiKeyAuthMiddleware {
    api_key_service: Arc<ApiKeyService>,
}

impl ApiKeyAuthMiddleware {
    pub fn new(api_key_service: Arc<ApiKeyService>) -> Self {
        Self { api_key_service }
    }
}

#[async_trait]
impl Handler for ApiKeyAuthMiddleware {
    async fn handle(&self, req: &mut Request, depot: &mut Depot, res: &mut Response, ctrl: &mut FlowCtrl) {
        // 从请求头获取API Key和Secret
        let api_key = req.headers().get("X-API-Key")
            .and_then(|v| v.to_str().ok());
        let api_secret = req.headers().get("X-API-Secret")
            .and_then(|v| v.to_str().ok());
        
        if let (Some(api_key), Some(api_secret)) = (api_key, api_secret) {
            match self.api_key_service.authenticate(api_key, api_secret).await {
                Ok(Some(session)) => {
                    // 检查速率限制
                    match self.api_key_service.check_rate_limit(api_key, session.rate_limit).await {
                        Ok(true) => {
                            // 记录使用日志
                            let ip = req.remote_addr().map(|addr| addr.ip());
                            let user_agent = req.headers().get("User-Agent")
                                .and_then(|v| v.to_str().ok());
                            let endpoint = Some(req.uri().path());
                            let method = Some(req.method().as_str());
                            
                            let _ = self.api_key_service.log_usage(
                                session.api_key_id,
                                ip,
                                user_agent,
                                endpoint,
                                method,
                                None, // status_code 在响应时记录
                                None, // response_time 在响应时记录
                            ).await;
                            
                            // 将会话信息存储到depot中
                            depot.insert("api_session", session);
                            depot.insert("api_key", api_key.to_string());
                            
                            ctrl.call_next(req, depot, res).await;
                            return;
                        },
                        Ok(false) => {
                            res.status_code(StatusCode::TOO_MANY_REQUESTS);
                            res.render(Json(serde_json::json!({
                                "error": "Rate limit exceeded"
                            })));
                            return;
                        },
                        Err(_) => {
                            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                            res.render(Json(serde_json::json!({
                                "error": "Authentication service error"
                            })));
                            return;
                        }
                    }
                },
                Ok(None) => {
                    res.status_code(StatusCode::UNAUTHORIZED);
                    res.render(Json(serde_json::json!({
                        "error": "Invalid API credentials"
                    })));
                    return;
                },
                Err(_) => {
                    res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                    res.render(Json(serde_json::json!({
                        "error": "Authentication service error"
                    })));
                    return;
                }
            }
        }
        
        // 未提供认证信息
        res.status_code(StatusCode::UNAUTHORIZED);
        res.render(Json(serde_json::json!({
            "error": "API Key and Secret required"
        })));
    }
}

// 权限检查中间件
pub struct PermissionMiddleware {
    required_permission: Permission,
}

impl PermissionMiddleware {
    pub fn new(required_permission: Permission) -> Self {
        Self { required_permission }
    }
}

#[async_trait]
impl Handler for PermissionMiddleware {
    async fn handle(&self, _req: &mut Request, depot: &mut Depot, res: &mut Response, ctrl: &mut FlowCtrl) {
        if let Some(session) = depot.get::<ApiKeySession>("api_session") {
            // 检查是否有所需权限
            if session.permissions.contains(&format!("{:?}", self.required_permission).to_lowercase()) ||
               session.permissions.contains(&"admin".to_string()) {
                ctrl.call_next(_req, depot, res).await;
                return;
            }
        }
        
        res.status_code(StatusCode::FORBIDDEN);
        res.render(Json(serde_json::json!({
            "error": "Insufficient permissions"
        })));
    }
}