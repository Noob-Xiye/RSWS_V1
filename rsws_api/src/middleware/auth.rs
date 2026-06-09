//! 认证中间件
//!
//! 统一 API Key 认证（Cregis 方案）：
//! - 前端传 user_id + timestamp + nonce + sign
//! - 后端通过 user_id 查 Redis 获取 api_key，重算签名验签
//!
//! 速率限制中间件
//! Admin 权限检查中间件

use crate::state::get_state;
use rsws_common::error::RswsError;
use rsws_db::redis::RedisService;
use salvo::prelude::*;

/// 获取真实客户端 IP（考虑可信代理）
fn get_real_client_ip(req: &Request, trusted_proxies: &[String]) -> String {
    let remote_ip = match req.remote_addr() {
        salvo::conn::SocketAddr::IPv4(v4) => v4.ip().to_string(),
        salvo::conn::SocketAddr::IPv6(v6) => v6.ip().to_string(),
        _ => "unknown".to_string(),
    };

    if trusted_proxies.is_empty() {
        return remote_ip;
    }

    if !trusted_proxies.contains(&remote_ip) {
        return remote_ip;
    }

    if let Some(xff) = req
        .headers()
        .get("X-Forwarded-For")
        .and_then(|v| v.to_str().ok())
    {
        let ips: Vec<&str> = xff.split(',').map(|s| s.trim()).collect();
        for ip in ips.iter().rev() {
            if !trusted_proxies.contains(&ip.to_string()) {
                return ip.to_string();
            }
        }
        if let Some(first_ip) = ips.first() {
            return first_ip.to_string();
        }
    }

    remote_ip
}

/// Nonce 去重检查（Redis SET NX EX）
///
/// 返回 Ok(true) 表示 nonce 首次使用（允许），Ok(false) 表示 nonce 已存在（拒绝重放）
async fn check_nonce_once(redis: &RedisService, nonce: &str) -> Result<bool, RswsError> {
    let key = format!("nonce:{}", nonce);
    redis.set_nx_ex(&key, "1", 300).await
}

/// API Key 认证中间件（Cregis 方案）
///
/// 认证流程：
/// 1. 从请求参数获取 user_id, timestamp, nonce, sign
/// 2. 检查时间戳防重放
/// 3. 先尝试 user 验签，失败则尝试 admin 验签
/// 4. 验签通过后注入 depot：user_id, api_key_id, is_admin
#[handler]
pub async fn api_key_auth(
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
    ctrl: &mut FlowCtrl,
) {
    let path = req.uri().path();
    // 跳过不需要认证的端点
    if path.contains("/user/login")
        || path.contains("/user/register")
        || path.contains("/health")
        || path.contains("/webhook/")
        || path.contains("/payment/paypal/success")
        || path.contains("/payment/paypal/cancel")
        || path.contains("/payment/usdt/")
        || path == "/api/v1/admin/login" || path.ends_with("/admin/login")
    {
        ctrl.call_next(req, depot, res).await;
        return;
    }

    // ========== Cregis 签名认证 ==========
    // 前端传: user_id, timestamp, nonce, sign
    if let (Some(user_id_str), Some(timestamp), Some(nonce), Some(sign)) = (
        req.query::<String>("user_id"),
        req.query::<String>("timestamp"),
        req.query::<String>("nonce"),
        req.query::<String>("sign"),
    ) {
        // 解析 user_id
        let user_id = match user_id_str.parse::<i64>() {
            Ok(id) => id,
            Err(_) => {
                res.status_code(StatusCode::BAD_REQUEST);
                res.render(Json(
                    rsws_common::response::ApiResponse::<()>::error_with_message(
                        rsws_common::error_code::ErrorCode::INVALID_PARAMETER,
                        "Invalid user_id format",
                    ),
                ));
                return;
            }
        };

        // 检查时间戳防重放
        if let Err(e) = check_timestamp(&timestamp) {
            res.status_code(StatusCode::UNAUTHORIZED);
            res.render(Json(
                rsws_common::response::ApiResponse::<()>::error_with_message(
                    e.error_code(),
                    e.to_string(),
                ),
            ));
            return;
        }

        // 收集所有参数用于签名验证（排除 sign 本身）
        let mut params: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        params.insert("user_id".to_string(), user_id_str);
        params.insert("timestamp".to_string(), timestamp);
        params.insert("nonce".to_string(), nonce.clone());

        // 从查询参数收集其他业务参数
        let query = req.queries();
        for (k, v) in query {
            if k != "sign" && k != "user_id" && k != "timestamp" && k != "nonce" {
                if let Some(first_value) = v.first() {
                    params.insert(k.clone(), first_value.clone());
                }
            }
        }

        let state = get_state(depot);

        // 先尝试 admin 验签（管理员优先级更高，避免 user 验签 Err 时跳过 admin 验签）
        if let Ok(Some(api_key_record)) = state
            .admin_api_key_manager
            .validate_signature(user_id, &params, &sign)
            .await
        {
            depot.insert("user_id", user_id);
            depot.insert("api_key_id", api_key_record.id);
            depot.insert("is_admin", true);

            tracing::info!("Admin user_id={} authenticated via admin_api_key", user_id);

            // Nonce 去重检查
            let redis = state.config_service.redis_client();
            if let Ok(false) = check_nonce_once(redis, &nonce).await {
                res.status_code(StatusCode::UNAUTHORIZED);
                res.render(Json(
                    rsws_common::response::ApiResponse::<()>::error_with_message(
                        rsws_common::error_code::ErrorCode::AUTH_SIGNATURE_INVALID,
                        "Nonce already used (replay detected)",
                    ),
                ));
                return;
            }

            ctrl.call_next(req, depot, res).await;
            return;
        }

        // Admin 验签失败，尝试 user 验签
        if let Ok(Some(api_key_record)) = state
            .user_api_key_manager
            .validate_signature(user_id, &params, &sign)
            .await
        {
            // 用户验签通过，检查是否同时持有 admin API Key
            let is_admin = state
                .admin_api_key_manager
                .get(user_id)
                .await
                .ok()
                .flatten()
                .is_some();

            depot.insert("user_id", user_id);
            depot.insert("api_key_id", api_key_record.id);
            depot.insert("is_admin", is_admin);

            // Nonce 去重检查
            let redis = state.config_service.redis_client();
            if let Ok(false) = check_nonce_once(redis, &nonce).await {
                res.status_code(StatusCode::UNAUTHORIZED);
                res.render(Json(
                    rsws_common::response::ApiResponse::<()>::error_with_message(
                        rsws_common::error_code::ErrorCode::AUTH_SIGNATURE_INVALID,
                        "Nonce already used (replay detected)",
                    ),
                ));
                return;
            }

            if is_admin {
                tracing::info!("Admin user_id={} passed auth (also holds admin api key)", user_id);
            }

            ctrl.call_next(req, depot, res).await;
            return;
        }

        // 验签失败，返回 401
        res.status_code(StatusCode::UNAUTHORIZED);
        res.render(Json(
            rsws_common::response::ApiResponse::<()>::error_with_message(
                rsws_common::error_code::ErrorCode::AUTH_SIGNATURE_INVALID,
                "Signature verification failed",
            ),
        ));
        return;
    }

    // ========== 无认证信息 ==========
    res.status_code(StatusCode::UNAUTHORIZED);
    res.render(Json(
        rsws_common::response::ApiResponse::<()>::error_with_message(
            rsws_common::error_code::ErrorCode::AUTH_MISSING_CREDENTIALS,
            "Missing authentication credentials. Please provide user_id, timestamp, nonce, and sign parameters.",
        ),
    ));
}

/// 速率限制中间件（Redis 固定窗口）
#[handler]
pub async fn rate_limit(
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
    ctrl: &mut FlowCtrl,
) {
    let state = get_state(depot);
    let trusted_proxies = &state.config.server.trusted_proxies;

    let client_id = depot
        .get::<i64>("api_key_id")
        .ok()
        .map(|id| format!("apikey:{}", *id))
        .unwrap_or_else(|| {
            let ip = get_real_client_ip(req, trusted_proxies);
            format!("ip:{}", ip)
        });

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

            res.add_header("X-RateLimit-Limit", limit.to_string(), true)
                .ok();
            res.add_header(
                "X-RateLimit-Remaining",
                (limit.saturating_sub(count)).max(0).to_string(),
                true,
            )
            .ok();

            if count > limit as i64 {
                tracing::warn!(
                    "Rate limit exceeded for {}: {}/{} in window {}",
                    client_id,
                    count,
                    limit,
                    window
                );
                res.status_code(StatusCode::TOO_MANY_REQUESTS);
                res.add_header("Retry-After", "60", true).ok();
                res.render(Json(
                    rsws_common::response::ApiResponse::<()>::error_with_message(
                        rsws_common::error_code::ErrorCode::RATE_LIMIT_EXCEEDED,
                        format!(
                            "Rate limit exceeded. Maximum {} requests per minute.",
                            limit
                        ),
                    ),
                ));
                return;
            }
        }
        Err(e) => {
            tracing::warn!("Redis rate limit check failed, allowing request: {}", e);
        }
    }

    ctrl.call_next(req, depot, res).await;
}

/// 检查时间戳防重放
/// 允许的时间偏差：5 分钟（300000ms）
fn check_timestamp(timestamp_str: &str) -> Result<(), RswsError> {
    let timestamp = timestamp_str
        .parse::<i64>()
        .map_err(|_| RswsError::unauthorized("Invalid timestamp format"))?;

    let now = chrono::Utc::now().timestamp_millis();
    let tolerance = 300_000i64; // 5 分钟

    if (now - timestamp).abs() > tolerance {
        tracing::warn!(
            "Timestamp out of range: {} (now: {}, tolerance: {}ms)",
            timestamp,
            now,
            tolerance
        );
        return Err(RswsError::unauthorized("Timestamp out of range"));
    }

    Ok(())
}

/// Admin 权限检查中间件
#[handler]
pub async fn require_admin(
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
    ctrl: &mut FlowCtrl,
) {
    let is_admin: bool = depot.get("is_admin").copied().unwrap_or(false);

    if !is_admin {
        tracing::warn!(
            "Non-admin attempted to access admin endpoint: {}",
            req.uri().path()
        );
        res.status_code(StatusCode::FORBIDDEN);
        res.render(Json(
            rsws_common::response::ApiResponse::<()>::error_with_message(
                rsws_common::error_code::ErrorCode::FORBIDDEN,
                "Admin access required",
            ),
        ));
        return;
    }

    ctrl.call_next(req, depot, res).await;
}

