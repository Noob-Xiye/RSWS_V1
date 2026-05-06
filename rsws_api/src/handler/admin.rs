//! 管理员处理器
//!
//! 管理员登录、CRUD、API Key 管理
//! 认证统一走 API Key，handler 内检查 is_admin 标识

use salvo::prelude::*;
use salvo_oapi::endpoint;
use rsws_common::{ResponseExt, error_code::ErrorCode, RswsError};
use serde::Deserialize;
use crate::state::get_state;
use rsws_service::UpdateLogConfigRequest;

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
                Ok(info) => res.success(info),
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
    // 检查是否管理员
    let is_admin: bool = depot.get("is_admin")
        .map(|v| *v)
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
    let is_admin: bool = depot.get("is_admin")
        .map(|v| *v)
        .unwrap_or(false);

    if !is_admin {
        res.http_error(StatusCode::FORBIDDEN, "Admin access required");
        return;
    }

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
    let is_admin: bool = depot.get("is_admin")
        .map(|v| *v)
        .unwrap_or(false);

    if !is_admin {
        res.http_error(StatusCode::FORBIDDEN, "Admin access required");
        return;
    }

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
    let is_admin: bool = depot.get("is_admin")
        .map(|v| *v)
        .unwrap_or(false);

    if !is_admin {
        res.http_error(StatusCode::FORBIDDEN, "Admin access required");
        return;
    }

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
    let is_admin: bool = depot.get("is_admin")
        .map(|v| *v)
        .unwrap_or(false);

    if !is_admin {
        res.http_error(StatusCode::FORBIDDEN, "Admin access required");
        return;
    }

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
    let is_admin: bool = depot.get("is_admin")
        .map(|v| *v)
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
    let is_admin: bool = depot.get("is_admin")
        .map(|v| *v)
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
    let is_admin: bool = depot.get("is_admin")
        .map(|v| *v)
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

    let state = get_state(depot);
    match state.admin_service.delete_api_key(key_id, admin_id).await {
        Ok(deleted) => res.success(serde_json::json!({
            "deleted": deleted
        })),
        Err(e) => res.error(e),
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
    let is_admin: bool = depot.get("is_admin")
        .map(|v| *v)
        .unwrap_or(false);

    if !is_admin {
        res.http_error(StatusCode::FORBIDDEN, "Admin access required");
        return;
    }

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
    let is_admin: bool = depot.get("is_admin")
        .map(|v| *v)
        .unwrap_or(false);

    if !is_admin {
        res.http_error(StatusCode::FORBIDDEN, "Admin access required");
        return;
    }

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
    let is_admin: bool = depot.get("is_admin")
        .map(|v| *v)
        .unwrap_or(false);

    if !is_admin {
        res.http_error(StatusCode::FORBIDDEN, "Admin access required");
        return;
    }

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
    let is_admin: bool = depot.get("is_admin")
        .map(|v| *v)
        .unwrap_or(false);

    if !is_admin {
        res.http_error(StatusCode::FORBIDDEN, "Admin access required");
        return;
    }

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
    let is_admin: bool = depot.get("is_admin")
        .map(|v| *v)
        .unwrap_or(false);

    if !is_admin {
        res.http_error(StatusCode::FORBIDDEN, "Admin access required");
        return;
    }

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
    let is_admin: bool = depot.get("is_admin")
        .map(|v| *v)
        .unwrap_or(false);

    if !is_admin {
        res.http_error(StatusCode::FORBIDDEN, "Admin access required");
        return;
    }

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
