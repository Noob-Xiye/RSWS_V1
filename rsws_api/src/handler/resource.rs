//! 资源处理器

use salvo::prelude::*;
use rsws_common::response::ApiResponse;
use rsws_common::error_code::ErrorCode;
use serde::Deserialize;
use crate::state::{get_state, require_user_id};

/// 资源列表查询参数
#[derive(Debug, Deserialize)]
pub struct ResourceQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub category_id: Option<i64>,
    pub search: Option<String>,
}

/// 获取资源列表
#[handler]
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

    match state.resource_service.list(query.category_id, page, page_size).await {
        Ok((resources, total)) => {
            let total_pages = if page_size > 0 { (total + page_size - 1) / page_size } else { 0 };
            res.render(Json(ApiResponse::success(serde_json::json!({
                "items": resources,
                "total": total,
                "page": page,
                "page_size": page_size,
                "total_pages": total_pages,
            }))));
        }
        Err(e) => {
            let code = e.error_code();
            let status = salvo::http::StatusCode::from_u16(code.http_status())
                .unwrap_or(salvo::http::StatusCode::INTERNAL_SERVER_ERROR);
            res.status_code(status);
            res.render(Json(ApiResponse::<()>::error_with_message(code, e.to_string())));
        }
    }
}

/// 获取资源详情
#[handler]
pub async fn get_resource(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let id: i64 = req.param("id").unwrap_or(0);

    if id <= 0 {
        res.status_code(StatusCode::BAD_REQUEST);
        res.render(Json(ApiResponse::<()>::error_with_message(
            ErrorCode::INVALID_PARAMETER,
            "Invalid resource ID"
        )));
        return;
    }

    let state = get_state(depot);

    match state.resource_service.get(id).await {
        Ok(Some(resource)) => {
            res.render(Json(ApiResponse::success(resource)));
        }
        Ok(None) => {
            res.status_code(StatusCode::NOT_FOUND);
            res.render(Json(ApiResponse::<()>::error(ErrorCode::RESOURCE_NOT_FOUND)));
        }
        Err(e) => {
            let code = e.error_code();
            let status = salvo::http::StatusCode::from_u16(code.http_status())
                .unwrap_or(salvo::http::StatusCode::INTERNAL_SERVER_ERROR);
            res.status_code(status);
            res.render(Json(ApiResponse::<()>::error_with_message(code, e.to_string())));
        }
    }
}

/// 创建资源
#[handler]
pub async fn create_resource(_req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let _user_id = match require_user_id(depot) {
        Ok(id) => id,
        Err(_) => {
            res.status_code(StatusCode::UNAUTHORIZED);
            res.render(Json(ApiResponse::<()>::error(ErrorCode::AUTH_MISSING_CREDENTIALS)));
            return;
        }
    };

    res.status_code(StatusCode::NOT_IMPLEMENTED);
    res.render(Json(ApiResponse::<()>::error_with_message(
        ErrorCode::NOT_FOUND,
        "Resource creation not yet implemented"
    )));
}

/// 更新资源
#[handler]
pub async fn update_resource(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let _id: i64 = req.param("id").unwrap_or(0);
    let _user_id = match require_user_id(depot) {
        Ok(id) => id,
        Err(_) => {
            res.status_code(StatusCode::UNAUTHORIZED);
            res.render(Json(ApiResponse::<()>::error(ErrorCode::AUTH_MISSING_CREDENTIALS)));
            return;
        }
    };

    res.status_code(StatusCode::NOT_IMPLEMENTED);
    res.render(Json(ApiResponse::<()>::error_with_message(
        ErrorCode::NOT_FOUND,
        "Resource update not yet implemented"
    )));
}

/// 删除资源
#[handler]
pub async fn delete_resource(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let _id: i64 = req.param("id").unwrap_or(0);
    let _user_id = match require_user_id(depot) {
        Ok(id) => id,
        Err(_) => {
            res.status_code(StatusCode::UNAUTHORIZED);
            res.render(Json(ApiResponse::<()>::error(ErrorCode::AUTH_MISSING_CREDENTIALS)));
            return;
        }
    };

    res.status_code(StatusCode::NOT_IMPLEMENTED);
    res.render(Json(ApiResponse::<()>::error_with_message(
        ErrorCode::NOT_FOUND,
        "Resource deletion not yet implemented"
    )));
}
