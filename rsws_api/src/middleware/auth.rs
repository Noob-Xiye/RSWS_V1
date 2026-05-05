//! 认证中间件

use salvo::prelude::*;
use crate::state::get_state;

/// API Key 认证中间件
#[handler]
pub async fn api_key_auth(
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
    ctrl: &mut FlowCtrl,
) {
    // 跳过不需要认证的端点
    let path = req.uri().path();
    if path.contains("/user/login")
        || path.contains("/user/register")
        || path.contains("/health")
        || path.contains("/webhook/")
        || path.contains("/payment/paypal/success")
        || path.contains("/payment/paypal/cancel")
        || path.contains("/payment/usdt/")
    {
        ctrl.call_next(req, depot, res).await;
        return;
    }

    // 从请求头获取 API Key
    let api_key = req
        .headers()
        .get("X-API-Key")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let api_secret = req
        .headers()
        .get("X-API-Secret")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    // 备选：从查询参数获取
    let api_key = api_key.or_else(|| req.query::<String>("api_key"));
    let api_secret = api_secret.or_else(|| req.query::<String>("api_secret"));

    match (api_key, api_secret) {
        (Some(key), Some(secret)) => {
            // 从数据库验证 API Key
            let state = get_state(depot);
            match state.api_key_service.validate(&key, &secret).await {
                Ok(Some(api_key_record)) => {
                    // 将用户 ID 注入到 Depot
                    depot.insert("user_id", api_key_record.user_id);
                    depot.insert("api_key_id", api_key_record.id);

                    // 异步更新最后使用时间（不阻塞请求）
                    let api_key_service = state.api_key_service.clone();
                    tokio::spawn(async move {
                        let _ = api_key_service.update_last_used(api_key_record.id).await;
                    });

                    ctrl.call_next(req, depot, res).await;
                }
                Ok(None) => {
                    res.status_code(StatusCode::UNAUTHORIZED);
                    res.render(Json(rsws_common::response::ApiResponse::<()>::error_with_message(
                        rsws_common::error_code::ErrorCode::AUTH_API_KEY_NOT_FOUND,
                        "Invalid API credentials"
                    )));
                }
                Err(e) => {
                    tracing::error!("API key validation error: {}", e);
                    res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                    res.render(Json(rsws_common::response::ApiResponse::<()>::internal_error(
                        "Authentication service error"
                    )));
                }
            }
        }
        _ => {
            res.status_code(StatusCode::UNAUTHORIZED);
            res.render(Json(rsws_common::response::ApiResponse::<()>::error_with_message(
                rsws_common::error_code::ErrorCode::AUTH_MISSING_CREDENTIALS,
                "Missing API credentials. Please provide X-API-Key and X-API-Secret headers."
            )));
        }
    }
}

/// 管理员权限检查中间件
#[handler]
pub async fn admin_auth(
    _req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
    ctrl: &mut FlowCtrl,
) {
    // TODO: 检查用户是否为管理员
    let _has_admin = depot.get::<i64>("user_id").is_ok();

    ctrl.call_next(_req, depot, res).await;
    // 如果用户不是管理员，返回 403
    // 目前所有认证用户都视为普通用户
}

/// 速率限制中间件（基础版）
#[handler]
pub async fn rate_limit(
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
    ctrl: &mut FlowCtrl,
) {
    // TODO: 实现 Redis 速率限制
    ctrl.call_next(req, depot, res).await;
}
