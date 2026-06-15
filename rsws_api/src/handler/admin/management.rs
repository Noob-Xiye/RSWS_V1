//! 管理员账号管理
//!
//! 创建、列表、查询、启用/停用、重置密码

use crate::state::{get_state, require_user_id};
use rsws_common::ResponseExt;
use rsws_common::RswsError;
use salvo::http::StatusCode;
use salvo::prelude::*;
use salvo_oapi::endpoint;
use serde::Deserialize;

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
    let operator_id = match require_user_id(depot) {
        Ok(id) => id,
        Err(status) => {
            res.status_code(status);
            return;
        }
    };

    let body: Result<CreateAdminBody, _> = req.parse_json().await;

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
                .create_admin(
                    &data.email,
                    &data.password,
                    &data.username,
                    &data.role,
                    Some(operator_id),
                    ip.as_deref(),
                )
                .await
            {
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

    match state
        .admin_service
        .list_admins(page, page_size, role.as_deref())
        .await
    {
        Ok((admins, total)) => {
            let total_pages = if page_size > 0 {
                (total + page_size - 1) / page_size
            } else {
                0
            };
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
    responses(
        (status_code = 200, description = "成功"),
        (status_code = 403, description = "非管理员"),
        (status_code = 404, description = "管理员不存在"),
    )
)]
pub async fn get_admin(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    eprintln!(
        "=== get_admin handler reached, id query={:?} ===",
        req.query::<i64>("id")
    );
    let id: i64 = req.query("id").unwrap_or(0);
    // DEBUG: 标记请求是否到达此 handler
    if id <= 0 {
        res.error_msg(
            RswsError::from(rsws_common::error_code::ErrorCode::INVALID_PARAMETER),
            "Invalid admin ID [from get_admin handler]",
        );
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
    responses(
        (status_code = 200, description = "成功"),
        (status_code = 403, description = "非管理员"),
        (status_code = 404, description = "管理员不存在"),
    )
)]
pub async fn deactivate_admin(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let id: i64 = req.query("id").unwrap_or(0);
    let operator_id = match require_user_id(depot) {
        Ok(id) => id,
        Err(status) => {
            res.status_code(status);
            return;
        }
    };

    if id <= 0 {
        res.error_msg(
            RswsError::from(rsws_common::error_code::ErrorCode::INVALID_PARAMETER),
            "Invalid admin ID [deactivate_admin]",
        );
        return;
    }

    let state = get_state(depot);
    let ip = req
        .headers()
        .get("X-Forwarded-For")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    match state
        .admin_service
        .deactivate_admin(id, operator_id, ip.as_deref())
        .await
    {
        Ok(()) => res.success(serde_json::json!({
            "id": id,
            "message": "Admin deactivated successfully"
        })),
        Err(e) => res.error(e),
    }
}

/// 激活管理员
#[endpoint(
    responses(
        (status_code = 200, description = "成功"),
        (status_code = 403, description = "非管理员"),
        (status_code = 404, description = "管理员不存在"),
    )
)]
pub async fn activate_admin(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let id: i64 = req.query("id").unwrap_or(0);
    let operator_id = match require_user_id(depot) {
        Ok(id) => id,
        Err(status) => {
            res.status_code(status);
            return;
        }
    };

    if id <= 0 {
        res.error_msg(
            RswsError::from(rsws_common::error_code::ErrorCode::INVALID_PARAMETER),
            "Invalid admin ID [activate_admin]",
        );
        return;
    }

    let state = get_state(depot);
    let ip_address = req
        .header::<String>("X-Forwarded-For")
        .or_else(|| req.header::<String>("X-Real-IP"))
        .map(|s| s.to_string());

    match state
        .admin_service
        .activate_admin(id, operator_id, ip_address.as_deref())
        .await
    {
        Ok(()) => res.success(serde_json::json!({
            "id": id,
            "message": "Admin activated successfully"
        })),
        Err(e) => res.error(e),
    }
}

/// 重置管理员密码
#[endpoint(
    request_body = ResetPasswordBody,
    responses(
        (status_code = 200, description = "成功"),
        (status_code = 403, description = "非管理员"),
        (status_code = 404, description = "管理员不存在"),
    )
)]
pub async fn reset_admin_password(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let id: i64 = req.query("id").unwrap_or(0);
    let operator_id = match require_user_id(depot) {
        Ok(id) => id,
        Err(status) => {
            res.status_code(status);
            return;
        }
    };

    if id <= 0 {
        res.error_msg(
            RswsError::from(rsws_common::error_code::ErrorCode::INVALID_PARAMETER),
            "Invalid admin ID [reset_admin_password]",
        );
        return;
    }

    let ip_address = req
        .header::<String>("X-Forwarded-For")
        .or_else(|| req.header::<String>("X-Real-IP"))
        .map(|s| s.to_string());

    let body: Result<ResetPasswordBody, _> = req.parse_json().await;
    match body {
        Ok(data) => {
            let state = get_state(depot);
            match state
                .admin_service
                .reset_password(id, &data.password, operator_id, ip_address.as_deref())
                .await
            {
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
