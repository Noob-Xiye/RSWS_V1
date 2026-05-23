//! 资源处理器
//!
//! 使用 ResponseExt 和 AuthHandler trait 简化样板代码

use crate::state::get_state;
use rsws_common::{error_code::ErrorCode, AuthHandler, ResponseExt, RswsError};
use rsws_model::resource::{CreateResourceRequest, UpdateResourceRequest};
use salvo::prelude::*;
use salvo_oapi::endpoint;
use serde::Deserialize;

/// 资源列表查询参数
#[derive(Debug, Deserialize, salvo_oapi::ToSchema)]
pub struct ResourceQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub category_id: Option<i64>,
    pub search: Option<String>,
}

/// 获取资源列表
#[endpoint(
    parameters(
        ("page", Query, description = "页码"),
        ("page_size", Query, description = "每页数量"),
        ("category_id", Query, description = "分类ID"),
        ("search", Query, description = "搜索关键词"),
    ),
    responses(
        (status_code = 200, description = "成功"),
    )
)]
pub async fn list_resources(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let query: ResourceQuery = req.parse_queries().unwrap_or(ResourceQuery {
        page: Some(1),
        page_size: Some(20),
        category_id: None,
        search: None,
    });

    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);

    let state = get_state(depot);

    match state
        .resource_service
        .search(query.category_id, query.search.as_deref(), page, page_size)
        .await
    {
        Ok((resources, total)) => {
            let total_pages = if page_size > 0 {
                (total + page_size - 1) / page_size
            } else {
                0
            };
            res.success(serde_json::json!({
                "items": resources,
                "total": total,
                "page": page,
                "page_size": page_size,
                "total_pages": total_pages,
            }));
        }
        Err(e) => {
            res.error(e);
        }
    }
}

/// 获取资源详情
#[endpoint(
    responses(
        (status_code = 200, description = "成功"),
        (status_code = 404, description = "资源不存在"),
    )
)]
pub async fn get_resource(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let id: i64 = req.param("id").unwrap_or(0);

    if id <= 0 {
        res.error_msg(
            RswsError::from(ErrorCode::INVALID_PARAMETER),
            "Invalid resource ID",
        );
        return;
    }

    let state = get_state(depot);

    match state.resource_service.get(id).await {
        Ok(Some(resource)) => {
            res.success(resource);
        }
        Ok(None) => {
            res.error(RswsError::from(ErrorCode::RESOURCE_NOT_FOUND));
        }
        Err(e) => {
            res.error(e);
        }
    }
}

/// 创建资源
#[endpoint(
    request_body = CreateResourceRequest,
    responses(
        (status_code = 201, description = "创建成功"),
        (status_code = 400, description = "请求格式错误"),
        (status_code = 401, description = "未认证"),
    )
)]
pub async fn create_resource(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let user_id = match res.auth_require_user_id(depot) {
        Some(id) => id,
        None => return,
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

            match state.resource_service.create(data, rsws_model::resource::OWNER_TYPE_USER, user_id).await {
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

/// 更新资源
#[endpoint(
    request_body = UpdateResourceRequest,
    responses(
        (status_code = 200, description = "更新成功"),
        (status_code = 400, description = "请求格式错误"),
        (status_code = 401, description = "未认证"),
        (status_code = 403, description = "无权限"),
        (status_code = 404, description = "资源不存在"),
    )
)]
pub async fn update_resource(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let id: i64 = req.param("id").unwrap_or(0);

    if id <= 0 {
        res.error_msg(
            RswsError::from(ErrorCode::INVALID_PARAMETER),
            "Invalid resource ID",
        );
        return;
    }

    let user_id = match res.auth_require_user_id(depot) {
        Some(uid) => uid,
        None => return,
    };

    let body = req.parse_json::<UpdateResourceRequest>().await;

    match body {
        Ok(data) => {
            let state = get_state(depot);

            match state.resource_service.update(id, data, user_id).await {
                Ok(resource) => {
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

/// 删除资源
#[endpoint(
    responses(
        (status_code = 200, description = "删除成功"),
        (status_code = 401, description = "未认证"),
        (status_code = 403, description = "无权限"),
        (status_code = 404, description = "资源不存在"),
    )
)]
pub async fn delete_resource(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let id: i64 = req.param("id").unwrap_or(0);

    if id <= 0 {
        res.error_msg(
            RswsError::from(ErrorCode::INVALID_PARAMETER),
            "Invalid resource ID",
        );
        return;
    }

    let user_id = match res.auth_require_user_id(depot) {
        Some(uid) => uid,
        None => return,
    };

    let state = get_state(depot);

    match state.resource_service.delete(id, user_id).await {
        Ok(()) => {
            res.success(serde_json::json!({
                "id": id,
                "message": "Resource deleted successfully"
            }));
        }
        Err(e) => {
            res.error(e);
        }
    }
}
