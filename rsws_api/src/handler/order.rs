//! 订单处理器

use salvo::prelude::*;
use rsws_common::response::ApiResponse;
use rsws_common::error_code::ErrorCode;
use serde::Deserialize;
use crate::state::{get_state, require_user_id};

/// 订单创建请求
#[derive(Debug, Deserialize)]
pub struct CreateOrderRequest {
    pub resource_id: i64,
    pub payment_method: String, // "paypal" | "usdt_trc20" | "usdt_erc20"
}

/// 获取订单列表
#[handler]
pub async fn list_orders(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let user_id = match require_user_id(depot) {
        Ok(id) => id,
        Err(_) => {
            res.status_code(StatusCode::UNAUTHORIZED);
            res.render(Json(ApiResponse::<()>::error(ErrorCode::AUTH_MISSING_CREDENTIALS)));
            return;
        }
    };

    let page: i32 = req.query("page").unwrap_or(1);
    let limit: i32 = req.query("limit").unwrap_or(20);

    let state = get_state(depot);

    match state.order_service.list_by_user(user_id, page, limit).await {
        Ok((orders, total)) => {
            let total_pages = if limit > 0 { (total + limit as i64 - 1) / limit as i64 } else { 0 };
            res.render(Json(ApiResponse::success(serde_json::json!({
                "items": orders,
                "total": total,
                "page": page,
                "limit": limit,
                "total_pages": total_pages,
            }))));
        }
        Err(e) => {
            let code = e.error_code();
            let status = salvo::http::StatusCode::from_u16(code.http_status())
                .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
            res.status_code(status);
            res.render(Json(ApiResponse::<()>::error_with_message(code, e.to_string())));
        }
    }
}

/// 获取订单详情
#[handler]
pub async fn get_order(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let id: i64 = req.param("id").unwrap_or(0);

    if id <= 0 {
        res.status_code(StatusCode::BAD_REQUEST);
        res.render(Json(ApiResponse::<()>::error_with_message(
            ErrorCode::INVALID_PARAMETER,
            "Invalid order ID"
        )));
        return;
    }

    let state = get_state(depot);

    match state.order_service.get(id).await {
        Ok(Some(order)) => {
            res.render(Json(ApiResponse::success(order)));
        }
        Ok(None) => {
            res.status_code(StatusCode::NOT_FOUND);
            res.render(Json(ApiResponse::<()>::error(ErrorCode::ORDER_NOT_FOUND)));
        }
        Err(e) => {
            let code = e.error_code();
            let status = salvo::http::StatusCode::from_u16(code.http_status())
                .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
            res.status_code(status);
            res.render(Json(ApiResponse::<()>::error_with_message(code, e.to_string())));
        }
    }
}

/// 创建订单
#[handler]
pub async fn create_order(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let user_id = match require_user_id(depot) {
        Ok(id) => id,
        Err(_) => {
            res.status_code(StatusCode::UNAUTHORIZED);
            res.render(Json(ApiResponse::<()>::error(ErrorCode::AUTH_MISSING_CREDENTIALS)));
            return;
        }
    };

    let body = req.parse_json::<CreateOrderRequest>().await;

    match body {
        Ok(data) => {
            let valid_methods = ["paypal", "usdt_trc20", "usdt_erc20"];
            if !valid_methods.contains(&data.payment_method.as_str()) {
                res.status_code(StatusCode::BAD_REQUEST);
                res.render(Json(ApiResponse::<()>::error_with_message(
                    ErrorCode::PAYMENT_METHOD_NOT_SUPPORTED,
                    format!("Unsupported payment method: {}", data.payment_method)
                )));
                return;
            }

            let state = get_state(depot);

            let amount = match state.resource_service.get(data.resource_id).await {
                Ok(Some(resource)) => resource.price,
                Ok(None) => {
                    res.status_code(StatusCode::NOT_FOUND);
                    res.render(Json(ApiResponse::<()>::error(ErrorCode::RESOURCE_NOT_FOUND)));
                    return;
                }
                Err(e) => {
                    let code = e.error_code();
                    let status = salvo::http::StatusCode::from_u16(code.http_status())
                        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
                    res.status_code(status);
                    res.render(Json(ApiResponse::<()>::error_with_message(code, e.to_string())));
                    return;
                }
            };

            match state.order_service.create(
                user_id,
                data.resource_id,
                amount,
                &data.payment_method,
            ).await {
                Ok(order) => {
                    res.status_code(StatusCode::CREATED);
                    res.render(Json(ApiResponse::success(serde_json::json!({
                        "id": order.id,
                        "resource_id": order.resource_id,
                        "amount": order.amount,
                        "payment_method": order.payment_method,
                        "status": order.status,
                        "expired_at": order.expired_at,
                    }))));
                }
                Err(e) => {
                    let code = e.error_code();
                    let status = salvo::http::StatusCode::from_u16(code.http_status())
                        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
                    res.status_code(status);
                    res.render(Json(ApiResponse::<()>::error_with_message(code, e.to_string())));
                }
            }
        }
        Err(e) => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(ApiResponse::<()>::error_with_message(
                ErrorCode::INVALID_REQUEST_FORMAT,
                format!("Invalid request: {}", e)
            )));
        }
    }
}

/// 取消订单
#[handler]
pub async fn cancel_order(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let id: i64 = req.param("id").unwrap_or(0);

    let user_id = match require_user_id(depot) {
        Ok(uid) => uid,
        Err(_) => {
            res.status_code(StatusCode::UNAUTHORIZED);
            res.render(Json(ApiResponse::<()>::error(ErrorCode::AUTH_MISSING_CREDENTIALS)));
            return;
        }
    };

    if id <= 0 {
        res.status_code(StatusCode::BAD_REQUEST);
        res.render(Json(ApiResponse::<()>::error_with_message(
            ErrorCode::INVALID_PARAMETER,
            "Invalid order ID"
        )));
        return;
    }

    let state = get_state(depot);

    match state.order_service.cancel(id, user_id).await {
        Ok(()) => {
            res.render(Json(ApiResponse::success(serde_json::json!({
                "id": id,
                "status": "cancelled",
                "message": "Order cancelled successfully"
            }))));
        }
        Err(e) => {
            let code = e.error_code();
            let status = salvo::http::StatusCode::from_u16(code.http_status())
                .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
            res.status_code(status);
            res.render(Json(ApiResponse::<()>::error_with_message(code, e.to_string())));
        }
    }
}

/// 检查订单状态（USDT 支付轮询）
#[handler]
pub async fn check_order_status(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let id: i64 = req.param("id").unwrap_or(0);

    if id <= 0 {
        res.status_code(StatusCode::BAD_REQUEST);
        res.render(Json(ApiResponse::<()>::error_with_message(
            ErrorCode::INVALID_PARAMETER,
            "Invalid order ID"
        )));
        return;
    }

    let state = get_state(depot);

    match state.order_service.get(id).await {
        Ok(Some(order)) => {
            res.render(Json(ApiResponse::success(serde_json::json!({
                "id": order.id,
                "status": order.status,
                "confirmations": 0,
                "required_confirmations": 3
            }))));
        }
        Ok(None) => {
            res.status_code(StatusCode::NOT_FOUND);
            res.render(Json(ApiResponse::<()>::error(ErrorCode::ORDER_NOT_FOUND)));
        }
        Err(e) => {
            let code = e.error_code();
            let status = salvo::http::StatusCode::from_u16(code.http_status())
                .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
            res.status_code(status);
            res.render(Json(ApiResponse::<()>::error_with_message(code, e.to_string())));
        }
    }
}
