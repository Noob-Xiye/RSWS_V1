//! 认证中间件
//!
//! 统一 API Key 认证：先查 api_keys（普通用户），未命中再查 admin_api_keys（管理员）
//! 支持两种认证方式：
//!   1. 签名认证（推荐）: api_key, timestamp, nonce, sign 作为请求参数
//!   2. Header 认证（降级兼容）: X-API-Key, X-API-Secret headers
//!
//! 速率限制中间件
//!
//! Admin 权限检查中间件

use crate::state::get_state;
use md5;
use rsws_common::error::RswsError;
use salvo::prelude::*;
use std::collections::HashMap;

/// 获取真实客户端 IP（考虑可信代理）
///
/// 安全策略：
/// 1. 如果可信代理列表为空，不信任 X-Forwarded-For，使用连接 IP
/// 2. 如果连接 IP 不在可信代理列表中，使用连接 IP
/// 3. 否则，从 X-Forwarded-For 找到最右边一个不在可信代理列表中的 IP
fn get_real_client_ip(req: &Request, trusted_proxies: &[String]) -> String {
    // 获取连接 IP
    let remote_ip = match req.remote_addr() {
        salvo::conn::SocketAddr::IPv4(v4) => v4.ip().to_string(),
        salvo::conn::SocketAddr::IPv6(v6) => v6.ip().to_string(),
        _ => "unknown".to_string(),
    };

    // 如果没有可信代理配置，不信任 X-Forwarded-For
    if trusted_proxies.is_empty() {
        return remote_ip;
    }

    // 检查连接 IP 是否在可信代理列表中
    if !trusted_proxies.contains(&remote_ip) {
        return remote_ip;
    }

    // 连接来自可信代理，解析 X-Forwarded-For
    if let Some(xff) = req
        .headers()
        .get("X-Forwarded-For")
        .and_then(|v| v.to_str().ok())
    {
        // X-Forwarded-For 格式: client, proxy1, proxy2, ...
        // 找到最右边一个不在可信代理列表中的 IP
        let ips: Vec<&str> = xff.split(',').map(|s| s.trim()).collect();

        for ip in ips.iter().rev() {
            if !trusted_proxies.contains(&ip.to_string()) {
                return ip.to_string();
            }
        }

        // 所有 IP 都在可信代理列表中，返回最左边的（原始客户端）
        if let Some(first_ip) = ips.first() {
            return first_ip.to_string();
        }
    }

    // 无法解析 X-Forwarded-For，使用连接 IP
    remote_ip
}

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

    // 尝试签名认证（参数方式，符合 Cregis 方案）
    if let (Some(api_key), Some(timestamp), Some(nonce), Some(sign)) = (
        req.query::<String>("api_key"),
        req.query::<String>("timestamp"),
        req.query::<String>("nonce"),
        req.query::<String>("sign"),
    ) {
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
        let mut params: HashMap<String, String> = HashMap::new();
        params.insert("api_key".to_string(), api_key.clone());
        params.insert("timestamp".to_string(), timestamp.clone());
        params.insert("nonce".to_string(), nonce.clone());

        // 从查询参数和表单数据收集其他参数
        if let Some(query) = req.query::<HashMap<String, String>>("") {
            for (k, v) in query {
                if k != "sign" && k != "api_key" && k != "timestamp" && k != "nonce" {
                    params.insert(k, v);
                }
            }
        }

        let state = get_state(depot);

        // 先尝试普通用户
        match state
            .api_key_service
            .validate_signature(&api_key, &params, &sign)
            .await
        {
            Ok(Some(api_key_record)) => {
                depot.insert("user_id", api_key_record.user_id);
                depot.insert("api_key_id", api_key_record.id);
                depot.insert("is_admin", false);

                let svc = state.api_key_service.clone();
                let record_id = api_key_record.id;
                tokio::spawn(async move {
                    let _ = svc.update_last_used(record_id).await;
                });

                ctrl.call_next(req, depot, res).await;
                return;
            }
            Ok(None) => {
                // 未命中普通用户，继续尝试管理员
            }
            Err(e) => {
                tracing::error!("Signature validation error: {}", e);
                res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                res.render(Json(
                    rsws_common::response::ApiResponse::<()>::internal_error(
                        "Authentication service error",
                    ),
                ));
                return;
            }
        }

        // 尝试管理员签名认证
        match state
            .admin_service
            .validate_admin_api_key_signature(&api_key, &params, &sign)
            .await
        {
            Ok(Some((key_record, admin))) => {
                let permissions: Vec<String> =
                    serde_json::from_value(admin.permissions.clone()).unwrap_or_default();

                depot.insert("user_id", admin.id);
                depot.insert("api_key_id", key_record.id);
                depot.insert("is_admin", true);
                depot.insert("admin_role", admin.role.clone());
                depot.insert("admin_permissions", permissions);

                let repo = state.admin_repo_clone();
                let record_id = key_record.id;
                tokio::spawn(async move {
                    let _ = repo.update_admin_api_key_last_used(record_id).await;
                });

                ctrl.call_next(req, depot, res).await;
                return;
            }
            Ok(None) => {
                // 签名验证失败，尝试降级到旧方式
            }
            Err(e) => {
                tracing::error!("Admin signature validation error: {}", e);
                res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                res.render(Json(
                    rsws_common::response::ApiResponse::<()>::internal_error(
                        "Authentication service error",
                    ),
                ));
                return;
            }
        }
    }

    // 降级兼容：Header 认证（X-API-Key + X-API-Secret）
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
                    res.render(Json(
                        rsws_common::response::ApiResponse::<()>::internal_error(
                            "Authentication service error",
                        ),
                    ));
                    return;
                }
            }

            // 2) 尝试管理员 API Key
            match state
                .admin_service
                .validate_admin_api_key(&key, &secret)
                .await
            {
                Ok(Some((key_record, admin))) => {
                    let permissions: Vec<String> =
                        serde_json::from_value(admin.permissions.clone()).unwrap_or_default();

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
                    res.render(Json(
                        rsws_common::response::ApiResponse::<()>::error_with_message(
                            rsws_common::error_code::ErrorCode::AUTH_API_KEY_NOT_FOUND,
                            "Invalid API credentials",
                        ),
                    ));
                }
                Err(e) => {
                    tracing::error!("Admin API key validation error: {}", e);
                    res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                    res.render(Json(
                        rsws_common::response::ApiResponse::<()>::internal_error(
                            "Authentication service error",
                        ),
                    ));
                }
            }
        }
        _ => {
            res.status_code(StatusCode::UNAUTHORIZED);
            res.render(Json(
                rsws_common::response::ApiResponse::<()>::error_with_message(
                    rsws_common::error_code::ErrorCode::AUTH_MISSING_CREDENTIALS,
                    "Missing API credentials. Please provide X-API-Key and X-API-Secret headers.",
                ),
            ));
        }
    }
}

/// 速率限制中间件（Redis 固定窗口）
///
/// 安全改进：使用可信代理配置获取真实客户端 IP，防止 X-Forwarded-For 伪造
#[handler]
pub async fn rate_limit(
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
    ctrl: &mut FlowCtrl,
) {
    let state = get_state(depot);
    let trusted_proxies = &state.config.server.trusted_proxies;

    // 获取客户端标识（优先用 API Key，未认证则用真实 IP）
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

// ============================================
// 签名认证辅助函数（Cregis 方案）
// ============================================

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

/// 计算签名（符合 Cregis 方案）
///
/// 算法：
/// 1. 排除 sign 字段，按 key ASCII 升序排序
/// 2. 拼接参数字符串（key + value）
/// 3. 拼接 api_secret 到字符串末尾
/// 4. MD5 计算并转小写 hex
///
/// 注意：Cregis 方案将 api_secret 拼在参数前面，但关键是保持前后端一致
#[allow(dead_code)]
fn compute_signature(params: &HashMap<String, String>, api_secret: &str) -> String {
    // 1. 获取所有 key（排除 sign），排序
    let mut keys: Vec<&String> = params.keys().filter(|k| (*k).as_str() != "sign").collect();
    keys.sort();

    // 2. 按 ASCII 顺序拼接 key + value
    let param_str: String = keys
        .iter()
        .map(|k| format!("{}{}", k, params[*k]))
        .collect();

    // 3. 拼接 api_secret（拼在前面，与 Cregis 方案一致）
    let sign_str = format!("{}{}", api_secret, param_str);

    // 4. MD5 + 小写 hex
    format!("{:x}", md5::compute(sign_str.as_bytes()))
}
///
/// 必须在 api_key_auth 之后使用，检查 is_admin 标志
/// 如果不是管理员，返回 403 Forbidden
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
