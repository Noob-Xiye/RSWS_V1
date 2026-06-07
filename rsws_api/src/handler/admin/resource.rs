//! 管理员资源管理
//!
//! 列表、创建、更新、删除、切换上下架

use crate::state::{get_state, require_user_id};
use rsws_common::{error_code::ErrorCode, ResponseExt, RswsError};
use rsws_model::resource::{CreateResourceRequest, UpdateResourceRequest};
use salvo::http::StatusCode;
use salvo::prelude::*;
use salvo_oapi::endpoint;

/// 管理员列出所有资源
#[endpoint(
    responses(
        (status_code = 200, description = "成功"),
        (status_code = 401, description = "未认证"),
        (status_code = 403, description = "非管理员"),
    )
)]
pub async fn list_resources(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let _admin_id = match require_user_id(depot) {
        Ok(id) => id,
        Err(status) => {
            res.status_code(status);
            return;
        }
    };

    let category_id: Option<i64> = req.query("category_id").unwrap_or(None);
    let search: Option<String> = req.query("search").unwrap_or(None);
    let page: i64 = req.query("page").unwrap_or(1);
    let page_size: i64 = req.query("page_size").unwrap_or(20);

    let state = get_state(depot);

    let result = if search.as_ref().is_none_or(|s| s.is_empty()) {
        state
            .resource_service
            .list(category_id, page, page_size)
            .await
    } else {
        state
            .resource_service
            .search(category_id, search.as_deref(), page, page_size)
            .await
    };

    match result {
        Ok((resources, total)) => {
            let total_pages = if page_size > 0 {
                (total + page_size - 1) / page_size
            } else {
                1
            };
            res.success(serde_json::json!({
                "items": resources,
                "total": total,
                "page": page,
                "page_size": page_size,
                "total_pages": total_pages,
            }));
        }
        Err(e) => res.error(e),
    }
}

/// 管理员创建平台资源
#[endpoint(
    request_body = CreateResourceRequest,
    responses(
        (status_code = 201, description = "创建成功"),
        (status_code = 400, description = "请求格式错误"),
        (status_code = 401, description = "未认证"),
        (status_code = 403, description = "非管理员"),
    )
)]
pub async fn create_platform_resource(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let admin_id = match require_user_id(depot) {
        Ok(id) => id,
        Err(status) => {
            res.status_code(status);
            return;
        }
    };

    let body = req.parse_json::<CreateResourceRequest>().await;

    match body {
        Ok(data) => {
            if data.title.trim().is_empty() {
                res.error_msg(
                    RswsError::from(ErrorCode::INVALID_PARAMETER),
                    "Title cannot be empty",
                );
                return;
            }

            let state = get_state(depot);

            match state
                .resource_service
                .create(data, rsws_model::resource::OWNER_TYPE_PLATFORM, admin_id)
                .await
            {
                Ok(resource) => {
                    res.status_code(StatusCode::CREATED);
                    res.success(resource);
                }
                Err(e) => {
                    res.error(e);
                }
            }
        }
        Err(e) => {
            res.error_msg(
                RswsError::from(ErrorCode::INVALID_REQUEST_FORMAT),
                format!("Invalid request: {}", e),
            );
        }
    }
}

/// 管理员更新资源（任意资源，跳过归属校验）
#[endpoint(
    request_body = UpdateResourceRequest,
    responses(
        (status_code = 200, description = "更新成功"),
        (status_code = 401, description = "未认证"),
        (status_code = 403, description = "非管理员"),
        (status_code = 404, description = "资源不存在"),
    )
)]
pub async fn update_platform_resource(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let id: i64 = req.param("id").unwrap_or(0);
    if id <= 0 {
        res.error_msg(
            RswsError::from(ErrorCode::INVALID_PARAMETER),
            "Invalid resource ID",
        );
        return;
    }

    let _admin_id = match require_user_id(depot) {
        Ok(id) => id,
        Err(status) => {
            res.status_code(status);
            return;
        }
    };

    let body = req.parse_json::<UpdateResourceRequest>().await;
    match body {
        Ok(data) => {
            let state = get_state(depot);
            match state.resource_service.admin_update(id, data).await {
                Ok(resource) => res.success(resource),
                Err(e) => res.error(e),
            }
        }
        Err(e) => {
            res.error_msg(
                RswsError::from(ErrorCode::INVALID_REQUEST_FORMAT),
                format!("Invalid request: {}", e),
            );
        }
    }
}

/// 管理员删除资源（任意资源，跳过归属校验）
#[endpoint(
    responses(
        (status_code = 200, description = "删除成功"),
        (status_code = 401, description = "未认证"),
        (status_code = 403, description = "非管理员"),
        (status_code = 404, description = "资源不存在"),
    )
)]
pub async fn delete_platform_resource(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let id: i64 = req.param("id").unwrap_or(0);
    if id <= 0 {
        res.error_msg(
            RswsError::from(ErrorCode::INVALID_PARAMETER),
            "Invalid resource ID",
        );
        return;
    }

    let _admin_id = match require_user_id(depot) {
        Ok(id) => id,
        Err(status) => {
            res.status_code(status);
            return;
        }
    };

    let state = get_state(depot);
    match state.resource_service.admin_delete(id).await {
        Ok(()) => {
            res.success(serde_json::json!({
                "id": id,
                "message": "Resource deleted successfully by admin"
            }));
        }
        Err(e) => res.error(e),
    }
}

/// 管理员切换资源上下架
#[endpoint(
    responses(
        (status_code = 200, description = "切换成功"),
        (status_code = 401, description = "未认证"),
        (status_code = 403, description = "非管理员"),
        (status_code = 404, description = "资源不存在"),
    )
)]
pub async fn toggle_platform_resource(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let id: i64 = req.param("id").unwrap_or(0);
    if id <= 0 {
        res.error_msg(
            RswsError::from(ErrorCode::INVALID_PARAMETER),
            "Invalid resource ID",
        );
        return;
    }

    let _admin_id = match require_user_id(depot) {
        Ok(id) => id,
        Err(status) => {
            res.status_code(status);
            return;
        }
    };

    let state = get_state(depot);
    // 先获取当前资源状态
    let resource = match state.resource_service.get(id).await {
        Ok(Some(r)) => r,
        Ok(None) => {
            res.error_msg(
                RswsError::from(ErrorCode::RESOURCE_NOT_FOUND),
                "Resource not found",
            );
            return;
        }
        Err(e) => {
            res.error(e);
            return;
        }
    };

    let data = UpdateResourceRequest {
        is_active: Some(!resource.is_active),
        ..Default::default()
    };

    match state.resource_service.admin_update(id, data).await {
        Ok(resource) => res.success(resource),
        Err(e) => res.error(e),
    }
}
