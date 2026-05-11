//! 管理员处理器
//!
//! 管理员登录、CRUD、API Key 管理
//!
//! **权限说明：**
//! - 所有 handler 已通过 `require_admin` 中间件保护
//! - 中间件确保 `is_admin == true` 才能访问
//! - handler 内部无需再检查权限

use salvo::prelude::*;
use salvo_oapi::endpoint;
use sqlx::PgPool;
use rsws_common::{ResponseExt, error_code::ErrorCode, RswsError};
use serde::Deserialize;
use chrono::{Duration, Utc};
use uuid::Uuid;
use crate::state::get_state;
use rsws_service::UpdateLogConfigRequest;
use rsws_model::user_models::admin::{
    AdminLoginResponse, DashboardStats, DailyOrderCount,
};
use rsws_db::{user::UserRepository, order::OrderRepository, resource::ResourceRepository};

/// 管理员登录请求
#[derive(Debug, Deserialize, salvo_oapi::ToSchema)]
pub struct AdminLoginBody {
    pub email: String,
    pub password: String,
}

/// 管理员登录（无需 API Key，使用邮箱+密码）
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
            let ip = req.headers()
                .get("X-Forwarded-For")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string());

            match state.admin_service.login(
                &data.email,
                &data.password,
                ip.as_deref(),
            ).await {
                Ok(info) => {
                    let token = Uuid::new_v4().to_string();
                    let expires_at = Utc::now() + Duration::days(30);
                    let login_resp = AdminLoginResponse {
                        admin: info,
                        token,
                        expires_at,
                    };
                    res.success(login_resp)
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
    let admin_id: i64 = match depot.get("user_id") {
        Ok(id) => *id,
        Err(_) => {
            res.http_error(StatusCode::UNAUTHORIZED, "Not authenticated");
            return;
        }
    };

    let state = get_state(depot);
    match state.admin_service.get_admin_info(admin_id).await {
        Ok(info) => res.success(info),
        Err(e) => res.error(e),
    }
}

/// 创建管理员请求体
#[derive(Debug, Deserialize, salvo_oapi::ToSchema)]
pub struct CreateAdminBody {
    pub email: String,
    pub password: String,
    pub username: String,
    pub role: String,
}

/// 创建管理员
#[endpoint(
    request_body = CreateAdminBody,
    responses(
        (status_code = 201, description = "创建成功"),
        (status_code = 403, description = "非管理员"),
        (status_code = 409, description = "邮箱已存在"),
    )
)]
pub async fn create_admin(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let operator_id: i64 = match depot.get("user_id") {
        Ok(id) => *id,
        Err(_) => {
            res.http_error(StatusCode::UNAUTHORIZED, "Not authenticated");
            return;
        }
    };

    let body: Result<CreateAdminBody, _> = req.parse_json().await;

    match body {
        Ok(data) => {
            let state = get_state(depot);
            let ip = req.headers()
                .get("X-Forwarded-For")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string());

            match state.admin_service.create_admin(
                &data.email,
                &data.password,
                &data.username,
                &data.role,
                Some(operator_id),
                ip.as_deref(),
            ).await {
                Ok(admin) => {
                    res.status_code(StatusCode::CREATED);
                    res.success(serde_json::json!({
                        "id": admin.id,
                        "email": admin.email,
                        "username": admin.username,
                        "role": admin.role,
                    }));
                }
                Err(e) => res.error(e),
            }
        }
        Err(e) => {
            res.http_error(StatusCode::BAD_REQUEST, format!("Invalid request: {}", e));
        }
    }
}

/// 获取管理员列表
#[endpoint(
    parameters(
        ("page", Query, description = "页码"),
        ("page_size", Query, description = "每页数量"),
        ("role", Query, description = "按角色筛选"),
    ),
    responses(
        (status_code = 200, description = "成功"),
        (status_code = 403, description = "非管理员"),
    )
)]
pub async fn list_admins(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let page: i64 = req.query("page").unwrap_or(1);
    let page_size: i64 = req.query("page_size").unwrap_or(20);
    let role: Option<String> = req.query("role");

    let state = get_state(depot);

    match state.admin_service.list_admins(page, page_size, role.as_deref()).await {
        Ok((admins, total)) => {
            let total_pages = if page_size > 0 { (total + page_size - 1) / page_size } else { 0 };
            res.success(serde_json::json!({
                "items": admins,
                "total": total,
                "page": page,
                "page_size": page_size,
                "total_pages": total_pages,
            }));
        }
        Err(e) => res.error(e),
    }
}

/// 获取指定管理员信息
#[endpoint(
    parameters(
        ("id", description = "管理员ID"),
    ),
    responses(
        (status_code = 200, description = "成功"),
        (status_code = 403, description = "非管理员"),
        (status_code = 404, description = "管理员不存在"),
    )
)]
pub async fn get_admin(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let id: i64 = req.param("id").unwrap_or(0);
    if id <= 0 {
        res.error_msg(RswsError::from(ErrorCode::INVALID_PARAMETER), "Invalid admin ID");
        return;
    }

    let state = get_state(depot);
    match state.admin_service.get_admin_info(id).await {
        Ok(info) => res.success(info),
        Err(e) => res.error(e),
    }
}

/// 停用管理员
#[endpoint(
    parameters(
        ("id", description = "管理员ID"),
    ),
    responses(
        (status_code = 200, description = "成功"),
        (status_code = 403, description = "非管理员"),
        (status_code = 404, description = "管理员不存在"),
    )
)]
pub async fn deactivate_admin(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let id: i64 = req.param("id").unwrap_or(0);
    let operator_id: i64 = match depot.get("user_id") {
        Ok(id) => *id,
        Err(_) => {
            res.http_error(StatusCode::UNAUTHORIZED, "Not authenticated");
            return;
        }
    };

    if id <= 0 {
        res.error_msg(RswsError::from(ErrorCode::INVALID_PARAMETER), "Invalid admin ID");
        return;
    }

    let state = get_state(depot);
    let ip = req.headers()
        .get("X-Forwarded-For")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    match state.admin_service.deactivate_admin(id, operator_id, ip.as_deref()).await {
        Ok(()) => res.success(serde_json::json!({
            "id": id,
            "message": "Admin deactivated successfully"
        })),
        Err(e) => res.error(e),
    }
}

/// 激活管理员
#[endpoint(
    parameters(
        ("id", description = "管理员ID"),
    ),
    responses(
        (status_code = 200, description = "成功"),
        (status_code = 403, description = "非管理员"),
        (status_code = 404, description = "管理员不存在"),
    )
)]
pub async fn activate_admin(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let operator_id: i64 = match depot.get("user_id") {
        Ok(id) => *id,
        Err(_) => {
            res.http_error(StatusCode::UNAUTHORIZED, "Not authenticated");
            return;
        }
    };

    let id: i64 = req.param("id").unwrap_or(0);
    if id <= 0 {
        res.error_msg(RswsError::from(ErrorCode::INVALID_PARAMETER), "Invalid admin ID");
        return;
    }

    let ip_address = req.header::<String>("X-Forwarded-For")
        .or_else(|| req.header::<String>("X-Real-IP"))
        .map(|s| s.to_string());

    let state = get_state(depot);
    match state.admin_service.activate_admin(id, operator_id, ip_address.as_deref()).await {
        Ok(()) => res.success(serde_json::json!({
            "id": id,
            "message": "Admin activated successfully"
        })),
        Err(e) => res.error(e),
    }
}

/// 重置管理员密码
#[endpoint(
    parameters(
        ("id", description = "管理员ID"),
    ),
    request_body = ResetPasswordBody,
    responses(
        (status_code = 200, description = "成功"),
        (status_code = 403, description = "非管理员"),
        (status_code = 404, description = "管理员不存在"),
    )
)]
pub async fn reset_admin_password(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let operator_id: i64 = match depot.get("user_id") {
        Ok(id) => *id,
        Err(_) => {
            res.http_error(StatusCode::UNAUTHORIZED, "Not authenticated");
            return;
        }
    };

    let id: i64 = req.param("id").unwrap_or(0);
    if id <= 0 {
        res.error_msg(RswsError::from(ErrorCode::INVALID_PARAMETER), "Invalid admin ID");
        return;
    }

    let ip_address = req.header::<String>("X-Forwarded-For")
        .or_else(|| req.header::<String>("X-Real-IP"))
        .map(|s| s.to_string());

    let body: Result<ResetPasswordBody, _> = req.parse_json().await;
    match body {
        Ok(data) => {
            let state = get_state(depot);
            match state.admin_service.reset_password(id, &data.password, operator_id, ip_address.as_deref()).await {
                Ok(()) => res.success(serde_json::json!({
                    "id": id,
                    "message": "Password reset successfully"
                })),
                Err(e) => res.error(e),
            }
        }
        Err(e) => {
            res.http_error(StatusCode::BAD_REQUEST, format!("Invalid request: {}", e));
        }
    }
}

#[derive(Debug, Deserialize, salvo_oapi::ToSchema)]
pub struct ResetPasswordBody {
    pub password: String,
}

// ==================== Admin API Key 管理 ====================

/// 创建管理员 API Key 请求
#[derive(Debug, Deserialize, salvo_oapi::ToSchema)]
pub struct CreateAdminApiKeyBody {
    pub name: String,
    pub permissions: Vec<String>,
    pub rate_limit: Option<i32>,
    pub expires_in_days: Option<i32>,
}

/// 创建管理员 API Key
#[endpoint(
    request_body = CreateAdminApiKeyBody,
    responses(
        (status_code = 201, description = "创建成功"),
        (status_code = 403, description = "非管理员"),
    )
)]
pub async fn create_api_key(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let admin_id: i64 = match depot.get("user_id") {
        Ok(id) => *id,
        Err(_) => {
            res.http_error(StatusCode::UNAUTHORIZED, "Not authenticated");
            return;
        }
    };

    let body: Result<CreateAdminApiKeyBody, _> = req.parse_json().await;

    match body {
        Ok(data) => {
            let state = get_state(depot);
            match state.admin_service.create_api_key(
                admin_id,
                &data.name,
                data.permissions,
                data.rate_limit,
                data.expires_in_days,
            ).await {
                Ok(response) => {
                    res.status_code(StatusCode::CREATED);
                    res.success(response);
                }
                Err(e) => res.error(e),
            }
        }
        Err(e) => {
            res.http_error(StatusCode::BAD_REQUEST, format!("Invalid request: {}", e));
        }
    }
}

/// 获取管理员的 API Key 列表
#[endpoint(
    responses(
        (status_code = 200, description = "成功"),
        (status_code = 403, description = "非管理员"),
    )
)]
pub async fn list_api_keys(_req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let admin_id: i64 = match depot.get("user_id") {
        Ok(id) => *id,
        Err(_) => {
            res.http_error(StatusCode::UNAUTHORIZED, "Not authenticated");
            return;
        }
    };

    let state = get_state(depot);
    match state.admin_service.list_api_keys(admin_id).await {
        Ok(keys) => res.success(keys),
        Err(e) => res.error(e),
    }
}

/// 删除管理员 API Key
#[endpoint(
    parameters(
        ("id", description = "管理员ID"),
        ("key_id", description = "API Key ID"),
    ),
    responses(
        (status_code = 200, description = "删除成功"),
        (status_code = 403, description = "非管理员"),
    )
)]
pub async fn delete_api_key(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let admin_id: i64 = match depot.get("user_id") {
        Ok(id) => *id,
        Err(_) => {
            res.http_error(StatusCode::UNAUTHORIZED, "Not authenticated");
            return;
        }
    };

    let key_id: i64 = req.param("key_id").unwrap_or(0);
    if key_id <= 0 {
        res.error_msg(RswsError::from(ErrorCode::INVALID_PARAMETER), "Invalid API key ID");
        return;
    }

    let state = get_state(depot);
    match state.admin_service.delete_api_key(key_id, admin_id).await {
        Ok(deleted) => res.success(serde_json::json!({
            "deleted": deleted
        })),
        Err(e) => res.error(e),
    }
}

// ==================== Admin API Key 管理 (续) ====================

/// 切换管理员 API Key 状态请求
#[derive(Debug, Deserialize, salvo_oapi::ToSchema)]
pub struct ToggleApiKeyStatusBody {
    pub is_active: bool,
}

/// 切换管理员 API Key 状态
#[endpoint(
    parameters(
        ("key_id", description = "API Key ID"),
    ),
    request_body = ToggleApiKeyStatusBody,
    responses(
        (status_code = 200, description = "切换成功"),
        (status_code = 403, description = "非管理员"),
    )
)]
pub async fn toggle_api_key_status(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let is_admin: bool = depot.get("is_admin").copied()
        .unwrap_or(false);

    if !is_admin {
        res.http_error(StatusCode::FORBIDDEN, "Admin access required");
        return;
    }

    let admin_id: i64 = match depot.get("user_id") {
        Ok(id) => *id,
        Err(_) => {
            res.http_error(StatusCode::UNAUTHORIZED, "Not authenticated");
            return;
        }
    };

    let key_id: i64 = req.param("key_id").unwrap_or(0);
    if key_id <= 0 {
        res.error_msg(RswsError::from(ErrorCode::INVALID_PARAMETER), "Invalid API key ID");
        return;
    }

    let body: Result<ToggleApiKeyStatusBody, _> = req.parse_json().await;
    
    match body {
        Ok(data) => {
            let state = get_state(depot);
            match state.admin_service.toggle_api_key_status(key_id, admin_id, data.is_active).await {
                Ok(_) => res.success("API key status updated"),
                Err(e) => res.error(e),
            }
        }
        Err(e) => {
            res.http_error(StatusCode::BAD_REQUEST, format!("Invalid request: {}", e));
        }
    }
}

// ==================== 日志配置管理 ====================

/// 获取所有日志配置
#[endpoint(
    responses(
        (status_code = 200, description = "成功"),
        (status_code = 403, description = "非管理员"),
    )
)]
pub async fn list_log_configs(_req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let state = get_state(depot);
    match state.log_service.list_log_configs().await {
        Ok(configs) => res.success(configs),
        Err(e) => res.error(e),
    }
}

/// 获取指定日志配置
#[endpoint(
    parameters(
        ("key", description = "配置键名"),
    ),
    responses(
        (status_code = 200, description = "成功"),
        (status_code = 403, description = "非管理员"),
        (status_code = 404, description = "配置不存在"),
    )
)]
pub async fn get_log_config(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let key: String = req.param("key").unwrap_or_default();
    if key.is_empty() {
        res.error_msg(RswsError::from(ErrorCode::INVALID_PARAMETER), "Config key is required");
        return;
    }

    let state = get_state(depot);
    match state.log_service.get_log_config(&key).await {
        Ok(Some(config)) => res.success(config),
        Ok(None) => res.http_error(StatusCode::NOT_FOUND, "Config not found"),
        Err(e) => res.error(e),
    }
}

/// 创建或更新日志配置请求
#[derive(Debug, Deserialize, salvo_oapi::ToSchema)]
pub struct SetLogConfigBody {
    pub config_key: String,
    pub config_value: String,
    pub config_type: Option<String>,
    pub description: Option<String>,
}

/// 创建日志配置（不存在则创建）
#[endpoint(
    request_body = SetLogConfigBody,
    responses(
        (status_code = 201, description = "创建成功"),
        (status_code = 403, description = "非管理员"),
    )
)]
pub async fn create_log_config(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let body: Result<SetLogConfigBody, _> = req.parse_json().await;
    match body {
        Ok(data) => {
            let state = get_state(depot);
            let config_type = data.config_type.as_deref().unwrap_or("string");
            match state.log_service.set_log_config(
                &data.config_key,
                &data.config_value,
                config_type,
                data.description.as_deref(),
            ).await {
                Ok(config) => {
                    res.status_code(StatusCode::CREATED);
                    res.success(config);
                }
                Err(e) => res.error(e),
            }
        }
        Err(e) => {
            res.http_error(StatusCode::BAD_REQUEST, format!("Invalid request: {}", e));
        }
    }
}

/// 更新日志配置请求
#[derive(Debug, Deserialize, salvo_oapi::ToSchema)]
pub struct UpdateLogConfigBody {
    pub config_value: String,
    pub config_type: Option<String>,
    pub description: Option<String>,
    pub is_active: Option<bool>,
}

/// 更新日志配置
#[endpoint(
    parameters(
        ("key", description = "配置键名"),
    ),
    request_body = UpdateLogConfigBody,
    responses(
        (status_code = 200, description = "更新成功"),
        (status_code = 403, description = "非管理员"),
        (status_code = 404, description = "配置不存在"),
    )
)]
pub async fn update_log_config(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let key: String = req.param("key").unwrap_or_default();
    if key.is_empty() {
        res.error_msg(RswsError::from(ErrorCode::INVALID_PARAMETER), "Config key is required");
        return;
    }

    let body: Result<UpdateLogConfigBody, _> = req.parse_json().await;
    match body {
        Ok(data) => {
            let state = get_state(depot);
            let request = UpdateLogConfigRequest {
                config_key: key,
                config_value: data.config_value,
                config_type: data.config_type,
                description: data.description,
                is_active: data.is_active,
            };
            match state.log_service.update_log_config(&request).await {
                Ok(config) => res.success(config),
                Err(e) => res.error(e),
            }
        }
        Err(e) => {
            res.http_error(StatusCode::BAD_REQUEST, format!("Invalid request: {}", e));
        }
    }
}

/// 删除日志配置
#[endpoint(
    parameters(
        ("key", description = "配置键名"),
    ),
    responses(
        (status_code = 200, description = "删除成功"),
        (status_code = 403, description = "非管理员"),
    )
)]
pub async fn delete_log_config(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let key: String = req.param("key").unwrap_or_default();
    if key.is_empty() {
        res.error_msg(RswsError::from(ErrorCode::INVALID_PARAMETER), "Config key is required");
        return;
    }

    let state = get_state(depot);
    match state.log_service.delete_log_config(&key).await {
        Ok(deleted) => res.success(serde_json::json!({
            "deleted": deleted
        })),
        Err(e) => res.error(e),
    }
}

// ==================== 日志查询 ====================

/// 查询系统日志
#[endpoint(
    parameters(
        ("level", Query, description = "日志级别筛选"),
        ("page", Query, description = "页码"),
        ("page_size", Query, description = "每页数量"),
    ),
    responses(
        (status_code = 200, description = "成功"),
        (status_code = 403, description = "非管理员"),
    )
)]
pub async fn query_system_logs(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let level: Option<String> = req.query("level");
    let page: i64 = req.query("page").unwrap_or(1);
    let page_size: i64 = req.query("page_size").unwrap_or(20);

    let state = get_state(depot);
    match state.log_service.query_system_logs(
        level.as_deref(),
        None, // module
        None, // user_id
        page,
        page_size,
    ).await {
        Ok((logs, total)) => {
            let total_pages = if page_size > 0 { (total + page_size - 1) / page_size } else { 0 };
            res.success(serde_json::json!({
                "items": logs,
                "total": total,
                "page": page,
                "page_size": page_size,
                "total_pages": total_pages,
            }));
        }
        Err(e) => res.error(e),
    }
}

// ==================== USDT 钱包管理 ====================

/// USDT 钱包请求体
#[derive(Debug, Deserialize, salvo_oapi::ToSchema)]
pub struct UsdtWalletRequest {
    pub address: String,
    pub name: Option<String>,
}

/// 列出所有 USDT 钱包
#[endpoint(
    responses(
        (status_code = 200, description = "成功"),
        (status_code = 403, description = "非管理员"),
    )
)]
pub async fn list_usdt_wallets(_req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let state = get_state(depot);
    match state.blockchain_service.list_usdt_wallets().await {
        Ok(wallets) => res.success(serde_json::json!({ "items": wallets })),
        Err(e) => res.error(e),
    }
}

/// 更新或创建 USDT 钱包
#[endpoint(
    parameters(
        ("network", description = "网络类型: tron/ethereum"),
    ),
    request_body = UsdtWalletRequest,
    responses(
        (status_code = 200, description = "更新成功"),
        (status_code = 403, description = "非管理员"),
    )
)]
pub async fn update_usdt_wallet(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let network: String = req.param("network").unwrap_or_else(|| "tron".to_string());
    if network != "tron" && network != "ethereum" {
        res.http_error(StatusCode::BAD_REQUEST, "Invalid network, use 'tron' or 'ethereum'");
        return;
    }

    let body = req.parse_json::<UsdtWalletRequest>().await;
    let state = get_state(depot);
    match body {
        Ok(data) => {
            let valid = if network == "tron" {
                state.blockchain_service.validate_trc20_address(&data.address)
            } else {
                state.blockchain_service.validate_erc20_address(&data.address)
            };
            if !valid {
                res.http_error(StatusCode::BAD_REQUEST, "Invalid address format");
                return;
            }

            match state.blockchain_service.upsert_usdt_wallet(&network, &data.address, data.name.as_deref()).await {
                Ok(wallet) => res.success(wallet),
                Err(e) => res.error(e),
            }
        }
        Err(e) => {
            res.http_error(StatusCode::BAD_REQUEST, format!("Invalid request: {}", e));
        }
    }
}

// ==================== // ==================== Dashboard 统计面板 ====================

/// 获取 Dashboard 统计面板数据
#[endpoint(
    responses(
        (status_code = 200, description = "成功"),
        (status_code = 403, description = "非管理员"),
    )
)]
pub async fn dashboard_stats(_req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let state = get_state(depot);
    let pool: PgPool = state.pool();

    let user_repo = UserRepository::new(pool.clone());
    let order_repo = OrderRepository::new(pool.clone());
    let resource_repo = ResourceRepository::new(pool.clone());

    // 并行查询所有统计数据
    let (user_result, order_result, resource_result) = tokio::join!(
        user_repo.get_basic_stats(),
        order_repo.get_basic_stats(),
        resource_repo.get_basic_stats(),
    );

    let (total_users, new_users_30d) = match user_result {
        Ok(v) => v,
        Err(e) => {
            res.error(e);
            return;
        }
    };

    let (total_orders, completed_orders, total_revenue, _orders_30d, revenue_30d) = match order_result {
        Ok(v) => v,
        Err(e) => {
            res.error(e);
            return;
        }
    };

    let (total_resources, active_resources, new_resources_30d) = match resource_result {
        Ok(v) => v,
        Err(e) => {
            res.error(e);
            return;
        }
    };

    // 查询过去30天订单趋势
    let orders_trend: Vec<DailyOrderCount> = match sqlx::query_as(
        r#"
        SELECT DATE(created_at AT TIME ZONE 'UTC')::text AS date, COUNT(*)::bigint AS count
        FROM orders
        WHERE created_at >= NOW() - INTERVAL '30 days'
        GROUP BY DATE(created_at AT TIME ZONE 'UTC')
        ORDER BY date ASC
        "#
    )
    .fetch_all(&pool)
    .await
    {
        Ok(v) => v,
        Err(e) => {
            res.error(RswsError::internal(format!("Failed to query orders trend: {}", e)));
            return;
        }
    };

    let stats = DashboardStats {
        total_users,
        new_users_30d,
        total_orders,
        completed_orders,
        total_revenue,      // 单位：分，前端除以100转元
        revenue_30d,         // 单位：分，前端除以100转元
        total_resources,
        active_resources,
        new_resources_30d,
        orders_trend,
    };

    res.success(stats);
}

/// 收入图表
#[endpoint(
    parameters(
        ("days", description = "天数，默认30天"),
    ),
    responses(
        (status_code = 200, description = "获取成功"),
    )
)]
pub async fn revenue_chart(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let state = get_state(depot);
    let pool = &state.pool;

    // 解析参数
    let days: i64 = req.param("days").unwrap_or(30).max(1).min(365);

    // 查询每日收入
    let rows: Vec<(String, i64)> = match sqlx::query_as(
        r#"
        SELECT DATE(completed_at AT TIME ZONE 'UTC')::text AS date, COALESCE(SUM(amount), 0)::bigint AS revenue
        FROM orders
        WHERE status = 'completed'
          AND completed_at >= NOW() - ($1 || ' days')::interval
        GROUP BY DATE(completed_at AT TIME ZONE 'UTC')
        ORDER BY date ASC
        "#,
    )
    .bind(days)
    .fetch_all(pool)
    .await
    {
        Ok(v) => v,
        Err(e) => {
            res.error(RswsError::internal(format!("Failed to query revenue chart: {}", e)));
            return;
        }
    };

    let dates: Vec<String> = rows.iter().map(|(d, _)| d.clone()).collect();
    let revenues: Vec<i64> = rows.iter().map(|(_, r)| *r).collect();

    let chart = serde_json::json!({
        "dates": dates,
        "revenues": revenues,
    });

    res.success(chart);
}

/// 禁用用户
#[endpoint(
    parameters(
        ("id", description = "用户ID"),
    ),
    responses(
        (status_code = 200, description = "禁用成功"),
        (status_code = 403, description = "无权限"),
        (status_code = 404, description = "用户不存在"),
    )
)]
pub async fn deactivate_user(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let id: i64 = req.param("id").unwrap_or(0);
    if id <= 0 {
        res.http_error(StatusCode::BAD_REQUEST, "Invalid user ID");
        return;
    }

    let state = get_state(depot);

    match state.user_service.deactivate_user(id).await {
        Ok(()) => {
            res.success(serde_json::json!({
                "message": "User deactivated successfully"
            }));
        }
        Err(e) => {
            res.error(e);
        }
    }
}

/// 启用用户
#[endpoint(
    parameters(
        ("id", description = "用户ID"),
    ),
    responses(
        (status_code = 200, description = "启用成功"),
        (status_code = 403, description = "无权限"),
        (status_code = 404, description = "用户不存在"),
    )
)]
pub async fn activate_user(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let id: i64 = req.param("id").unwrap_or(0);
    if id <= 0 {
        res.http_error(StatusCode::BAD_REQUEST, "Invalid user ID");
        return;
    }

    let state = get_state(depot);

    match state.user_service.activate_user(id).await {
        Ok(()) => {
            res.success(serde_json::json!({
                "message": "User activated successfully"
            }));
        }
        Err(e) => {
            res.error(e);
        }
    }
}

/// 用户列表（管理员分页查询）
#[endpoint(
    parameters(
        ("page", Query, description = "页码，默认1"),
        ("page_size", Query, description = "每页条数，默认20"),
        ("email", Query, description = "邮箱筛选（模糊匹配）"),
        ("username", Query, description = "用户名筛选（模糊匹配）"),
        ("is_active", Query, description = "是否启用筛选"),
    ),
    responses(
        (status_code = 200, description = "获取成功"),
        (status_code = 403, description = "无权限"),
    )
)]
pub async fn list_users(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let state = get_state(depot);

    let page: i64 = req.query::<i64>("page").unwrap_or(1).max(1);
    let page_size: i64 = req.query::<i64>("page_size").unwrap_or(20).clamp(1, 100);
    let email: Option<String> = req.query("email");
    let username: Option<String> = req.query("username");
    let is_active: Option<bool> = req.query::<bool>("is_active");

    let (users, total) = match state.user_service
        .list_users(page, page_size, email.as_deref(), username.as_deref(), is_active)
        .await
    {
        Ok(result) => result,
        Err(e) => {
            res.error(e);
            return;
        }
    };

    res.success(serde_json::json!({
        "data": users,
        "total": total,
        "page": page,
        "page_size": page_size,
    }));
}
