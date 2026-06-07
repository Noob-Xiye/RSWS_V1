//! 管理员操作用户
//!
//! 用户列表、启用/停用

use crate::state::get_state;
use rsws_common::ResponseExt;
use salvo::prelude::*;
use salvo_oapi::endpoint;

/// 禁用用户
#[endpoint(
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

    let (users, total) = match state
        .user_service
        .list_users(
            page,
            page_size,
            email.as_deref(),
            username.as_deref(),
            is_active,
        )
        .await
    {
        Ok(result) => result,
        Err(e) => {
            res.error(e);
            return;
        }
    };

    let total_pages = if page_size > 0 {
        (total + page_size - 1) / page_size
    } else {
        1
    };

    res.success(serde_json::json!({
        "items": users,
        "total": total,
        "page": page,
        "page_size": page_size,
        "total_pages": total_pages,
    }));
}
