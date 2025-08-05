use salvo::prelude::*;
use rsws_service::AuthService;
use std::sync::Arc;
use chrono::Utc;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use hex;

pub struct SignatureAuthMiddleware {
    auth_service: Arc<AuthService>,
}

impl SignatureAuthMiddleware {
    pub fn new(auth_service: Arc<AuthService>) -> Self {
        Self { auth_service }
    }
}

#[async_trait]
impl Handler for SignatureAuthMiddleware {
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
        let signature = req.headers().get("X-Signature")
            .and_then(|v| v.to_str().ok());
        
        if let (Some(api_key), Some(timestamp), Some(signature)) = (api_key, timestamp, signature) {
            // 验证时间戳（5分钟内有效）
            let now = Utc::now().timestamp();
            if (now - timestamp).abs() > 300 {
                res.status_code(StatusCode::UNAUTHORIZED);
                res.render(Json(serde_json::json!({
                    "error": "Request timestamp expired"
                })));
                ctrl.skip_rest();
                return;
            }
            
            // 验证API Key和签名
            match self.auth_service.validate_api_request(api_key, timestamp, signature, req).await {
                Ok(validation) => {
                    if validation.is_valid {
                        if let Some(session) = validation.user_session {
                            // 将用户信息存储到depot中
                            depot.insert("user_id", session.user_id);
                            depot.insert("session", session);
                            req.headers_mut().insert("X-User-ID", session.user_id.to_string().parse().unwrap());
                        }
                    } else {
                        res.status_code(StatusCode::UNAUTHORIZED);
                        res.render(Json(serde_json::json!({
                            "error": validation.error_message.unwrap_or("Invalid signature".to_string())
                        })));
                        ctrl.skip_rest();
                        return;
                    }
                }
                Err(e) => {
                    res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                    res.render(Json(serde_json::json!({
                        "error": format!("Authentication error: {}", e)
                    })));
                    ctrl.skip_rest();
                    return;
                }
            }
        } else {
            res.status_code(StatusCode::UNAUTHORIZED);
            res.render(Json(serde_json::json!({
                "error": "Missing authentication headers"
            })));
            ctrl.skip_rest();
            return;
        }
    }
}