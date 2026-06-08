//! 管理员认证
//!
//! 登录、获取当前管理员信息

use crate::state::{get_state, require_user_id};
use chrono::{Duration, Utc};
use rsws_common::ResponseExt;
use rsws_model::user_models::admin::AdminLoginResponse;
use salvo::prelude::*;
use salvo_oapi::endpoint;
use serde::Deserialize;

/// 管理员登录请求
#[derive(Debug, Deserialize, salvo_oapi::ToSchema)]
pub struct AdminLoginBody {
    pub email: String,
    pub password: String,
}

/// 管理员登录（无需 API Key，使用邮箱+密码）
///
/// 流程：
/// 1. 验证邮箱+密码
/// 2. 创建 admin_api_key（Redis 存储）
/// 3. 返回 admin 信息 + api_key
#[endpoint(
    request_body = AdminLoginBody,
    responses(
        (status_code = 200, description = "登录成功"),
        (status_code = 401, description = "认证失败"),
    )
)]
pub async fn login(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let body: Result<AdminLoginBody, _> = req.parse_json().await;

    match body {
        Ok(data) => {
            let state = get_state(depot);
            let ip = req
                .headers()
                .get("X-Forwarded-For")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string());

            match state
                .admin_service
                .login(&data.email, &data.password, ip.as_deref())
                .await
            {
                Ok(info) => {
                    // 为管理员创建 admin_api_key
                    let create_req = rsws_model::api_key::CreateApiKeyRequest {
                        name: "login_session".to_string(),
                        permissions: vec!["all".to_string()],
                        rate_limit: Some(1000),
                        expires_in_days: Some(30),
                    };
                    match state.admin_api_key_manager.create(info.id, create_req).await
                    {
                        Ok(api_key_resp) => {
                            let login_resp = AdminLoginResponse {
                                admin: info,
                                api_key: api_key_resp.api_key,
                                expires_at: api_key_resp
                                    .expires_at
                                    .unwrap_or_else(|| Utc::now() + Duration::days(30)),
                            };
                            res.success(login_resp);
                        }
                        Err(e) => {
                            tracing::error!("Failed to create admin api_key: {}", e);
                            res.http_error(
                                StatusCode::INTERNAL_SERVER_ERROR,
                                "Login succeeded but session creation failed",
                            );
                        }
                    }
                }
                Err(e) => res.error(e),
            }
        }
        Err(e) => {
            res.http_error(StatusCode::BAD_REQUEST, format!("Invalid request: {}", e));
        }
    }
}

/// 获取当前管理员信息
#[endpoint(
    responses(
        (status_code = 200, description = "成功"),
        (status_code = 401, description = "未认证"),
        (status_code = 403, description = "非管理员"),
    )
)]
pub async fn get_current_admin(_req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let admin_id = match require_user_id(depot) {
        Ok(id) => id,
        Err(status) => {
            res.status_code(status);
            return;
        }
    };

    let state = get_state(depot);
    match state.admin_service.get_admin_info(admin_id).await {
        Ok(info) => res.success(info),
        Err(e) => res.error(e),
    }
}
