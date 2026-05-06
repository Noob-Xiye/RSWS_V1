//! 认证中间件
//!
//! 统一 API Key 认证：先查 api_keys（普通用户），未命中再查 admin_api_keys（管理员）
//! 速率限制中间件

use salvo::prelude::*;
use crate::state::get_state;

/// API Key 认证中间件
///
/// 统一认证流程：
/// 1. 从 X-API-Key + X-API-Secret 头获取凭据
/// 2. 先查 api_keys 表（普通用户）
/// 3. 未命中则查 admin_api_keys 表（管理员）
/// 4. 验证通过后注入 depot：
///    - user_id (i64) — 普通用户或管理员对应的 user_id/admin_id
///    - api_key_id (i64) — API Key 记录 ID
///    - is_admin (bool) — 是否为管理员
///    - admin_role (String) — 管理员角色（仅管理员）
///    - admin_permissions (Vec<String>) — 管理员权限（仅管理员）
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
        || path.contains("/admin/login")
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
            let state = get_state(depot);

            // 1) 先尝试普通用户 API Key
            match state.api_key_service.validate(&key, &secret).await {
                Ok(Some(api_key_record)) => {
                    depot.insert("user_id", api_key_record.user_id);
                    depot.insert("api_key_id", api_key_record.id);
                    depot.insert("is_admin", false);

                    // 异步更新最后使用时间
                    let svc = state.api_key_service.clone();
                    let record_id = api_key_record.id;
                    tokio::spawn(async move {
                        let _ = svc.update_last_used(record_id).await;
                    });

                    ctrl.call_next(req, depot, res).await;
                    return;
                }
                Ok(None) => {
                    // 未命中 api_keys，继续尝试 admin_api_keys
                }
                Err(e) => {
                    tracing::error!("API key validation error: {}", e);
                    res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                    res.render(Json(rsws_common::response::ApiResponse::<()>::internal_error(
                        "Authentication service error"
                    )));
                    return;
                }
            }

            // 2) 尝试管理员 API Key
            match state.admin_service.validate_admin_api_key(&key, &secret).await {
                Ok(Some((key_record, admin))) => {
                    let permissions: Vec<String> = serde_json::from_value(admin.permissions.clone())
                        .unwrap_or_default();

                    depot.insert("user_id", admin.id);
                    depot.insert("api_key_id", key_record.id);
                    depot.insert("is_admin", true);
                    depot.insert("admin_role", admin.role.clone());
                    depot.insert("admin_permissions", permissions);

                    // 异步更新最后使用时间
                    let repo = state.admin_repo_clone();
                    let record_id = key_record.id;
                    tokio::spawn(async move {
                        let _ = repo.update_admin_api_key_last_used(record_id).await;
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
                    tracing::error!("Admin API key validation error: {}", e);
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

/// 速率限制中间件（Redis 固定窗口）
#[handler]
pub async fn rate_limit(
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
    ctrl: &mut FlowCtrl,
) {
    // 获取客户端标识（优先用 API Key，未认证则用 IP）
    let client_id = depot
        .get::<i64>("api_key_id")
        .ok()
        .map(|id| format!("apikey:{}", *id))
        .or_else(|| {
            req.headers()
                .get("X-Forwarded-For")
                .and_then(|v| v.to_str().ok())
                .map(|s| {
                    s.split(',').next().unwrap_or(s).trim().to_string()
                })
                .map(|ip| format!("ip:{}", ip))
        })
        .unwrap_or_else(|| "unknown".to_string());

    let state = get_state(depot);
    let redis = state.config_service.redis_client();

    let limit = state
        .config_service
        .get_int("api_key.default_rate_limit")
        .await
        .unwrap_or(Some(100))
        .unwrap_or(100);

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let window = now / 60;
    let key = format!("ratelimit:{}:{}", client_id, window);

    match redis.incr(&key, 1).await {
        Ok(count) => {
            if count == 1 {
                let _ = redis.expire(&key, 120).await;
            }

            res.add_header("X-RateLimit-Limit", limit.to_string(), true).ok();
            res.add_header("X-RateLimit-Remaining", (limit.saturating_sub(count)).max(0).to_string(), true).ok();

            if count > limit as i64 {
                tracing::warn!(
                    "Rate limit exceeded for {}: {}/{} in window {}",
                    client_id, count, limit, window
                );
                res.status_code(StatusCode::TOO_MANY_REQUESTS);
                res.add_header("Retry-After", "60", true).ok();
                res.render(Json(rsws_common::response::ApiResponse::<()>::error_with_message(
                    rsws_common::error_code::ErrorCode::RATE_LIMIT_EXCEEDED,
                    format!("Rate limit exceeded. Maximum {} requests per minute.", limit)
                )));
                return;
            }
        }
        Err(e) => {
            tracing::warn!("Redis rate limit check failed, allowing request: {}", e);
        }
    }

    ctrl.call_next(req, depot, res).await;
}
