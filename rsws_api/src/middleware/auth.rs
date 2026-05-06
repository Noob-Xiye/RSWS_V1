//! 认证中间件
//!
//! 提供 API Key 认证、管理员权限检查、速率限制中间件

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
        || path.contains("/payment/pal/cancel")
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
                    let record_id = api_key_record.id;
                    tokio::spawn(async move {
                        let _ = api_key_service.update_last_used(record_id).await;
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
/// 
/// 要求请求已通过 api_key_auth 认证（depot 中有 user_id）
/// 检查 admins 表验证用户是否为管理员
#[handler]
pub async fn admin_auth(
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
    ctrl: &mut FlowCtrl,
) {
    let user_id = match depot.get::<i64>("user_id") {
        Ok(id) => *id,
        Err(_) => {
            res.status_code(StatusCode::UNAUTHORIZED);
            res.render(Json(rsws_common::response::ApiResponse::<()>::error_with_message(
                rsws_common::error_code::ErrorCode::AUTH_MISSING_CREDENTIALS,
                "Authentication required"
            )));
            return;
        }
    };

    let state = get_state(depot);

    // 查询 admins 表检查用户是否为管理员
    let is_admin: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM admins WHERE user_id = $1 AND is_active = true"
    )
    .bind(user_id)
    .fetch_one(state.config_service.pool())
    .await
    .unwrap_or(0);

    if is_admin == 0 {
        tracing::warn!("Non-admin user {} attempted to access admin resource", user_id);
        res.status_code(StatusCode::FORBIDDEN);
        res.render(Json(rsws_common::response::ApiResponse::<()>::error_with_message(
            rsws_common::error_code::ErrorCode::AUTH_PERMISSION_DENIED,
            "Admin access required"
        )));
        return;
    }

    // 注入管理员标记到 Depot
    depot.insert("is_admin", true);

    ctrl.call_next(req, depot, res).await;
}

/// 速率限制中间件（Redis 滑动窗口）
/// 
/// 使用 Redis 实现固定窗口限速：key = "ratelimit:{client_id}:{minute_window}"
/// 默认限制 100 次/分钟，可通过 system_configs 表配置 api_key.default_rate_limit
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
                    // 取第一个 IP（如果有代理）
                    s.split(',').next().unwrap_or(s).trim().to_string()
                })
                .map(|ip| format!("ip:{}", ip))
        })
        .unwrap_or_else(|| "unknown".to_string());

    let state = get_state(depot);
    let redis = state.config_service.redis_client();

    // 获取速率限制配置（默认 100 次/分钟）
    let limit = state
        .config_service
        .get_int("api_key.default_rate_limit")
        .await
        .unwrap_or(Some(100))
        .unwrap_or(100);

    // 获取当前分钟窗口
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let window = now / 60;
    let key = format!("ratelimit:{}:{}", client_id, window);

    // Redis INCR + EXPIRE
    match redis.incr(&key, 1).await {
        Ok(count) => {
            // 第一次访问，设置过期时间 120s（多窗口保护）
            if count == 1 {
                let _ = redis.expire(&key, 120).await;
            }

            // 添加速率限制响应头
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
            // Redis 故障时放行（fail-open），避免单点故障阻塞服务
            tracing::warn!("Redis rate limit check failed, allowing request: {}", e);
        }
    }

    ctrl.call_next(req, depot, res).await;
}
