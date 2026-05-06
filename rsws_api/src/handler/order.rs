//! 订单处理器
//!
//! 使用 ResponseExt 和 AuthHandler trait 简化样板代码

use salvo::prelude::*;
use salvo_oapi::endpoint;
use rsws_common::{ResponseExt, AuthHandler, error_code::ErrorCode, RswsError};
use serde::Deserialize;
use salvo_oapi::ToSchema;
use crate::state::get_state;

/// 订单创建请求
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateOrderRequest {
    pub resource_id: i64,
    pub payment_method: String, // "paypal" | "usdt_trc20" | "usdt_erc20"
}

/// 获取订单列表
#[endpoint(
    parameters(
        ("page", Query, description = "页码"),
        ("limit", Query, description = "每页数量"),
    ),
    responses(
        (status_code = 200, description = "成功"),
        (status_code = 401, description = "未认证"),
    )
)]
pub async fn list_orders(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let user_id = match res.auth_require_user_id(depot) {
        Some(id) => id,
        None => return,
    };

    let page: i32 = req.query("page").unwrap_or(1);
    let limit: i32 = req.query("limit").unwrap_or(20);

    let state = get_state(depot);

    match state.order_service.list_by_user(user_id, page, limit).await {
        Ok((orders, total)) => {
            let total_pages = if limit > 0 { (total + limit as i64 - 1) / limit as i64 } else { 0 };
            res.success(serde_json::json!({
                "items": orders,
                "total": total,
                "page": page,
                "limit": limit,
                "total_pages": total_pages,
            }));
        }
        Err(e) => {
            res.error(e);
        }
    }
}

/// 获取订单详情
#[endpoint(
    parameters(
        ("id", description = "订单ID"),
    ),
    responses(
        (status_code = 200, description = "成功"),
        (status_code = 404, description = "订单不存在"),
    )
)]
pub async fn get_order(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let id: i64 = req.param("id").unwrap_or(0);

    if id <= 0 {
        res.error_msg(
            RswsError::from(ErrorCode::INVALID_PARAMETER),
            "Invalid order ID"
        );
        return;
    }

    let state = get_state(depot);

    match state.order_service.get(id).await {
        Ok(Some(order)) => {
            res.success(order);
        }
        Ok(None) => {
            res.error(RswsError::from(ErrorCode::ORDER_NOT_FOUND));
        }
        Err(e) => {
            res.error(e);
        }
    }
}

/// 创建订单
#[endpoint(
    request_body = CreateOrderRequest,
    responses(
        (status_code = 201, description = "创建成功"),
        (status_code = 400, description = "请求格式错误"),
        (status_code = 401, description = "未认证"),
        (status_code = 404, description = "资源不存在"),
    )
)]
pub async fn create_order(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let user_id = match res.auth_require_user_id(depot) {
        Some(id) => id,
        None => return,
    };

    let body = req.parse_json::<CreateOrderRequest>().await;

    match body {
        Ok(data) => {
            let valid_methods = ["paypal", "usdt_trc20", "usdt_erc20"];
            if !valid_methods.contains(&data.payment_method.as_str()) {
                res.error_msg(
                    RswsError::from(ErrorCode::PAYMENT_METHOD_NOT_SUPPORTED),
                    format!("Unsupported payment method: {}", data.payment_method)
                );
                return;
            }

            let state = get_state(depot);

            // 获取资源价格
            let amount = match state.resource_service.get(data.resource_id).await {
                Ok(Some(resource)) => resource.price,
                Ok(None) => {
                    res.error(RswsError::from(ErrorCode::RESOURCE_NOT_FOUND));
                    return;
                }
                Err(e) => {
                    res.error(e);
                    return;
                }
            };

            match state.order_service.create(user_id, data.resource_id, amount, &data.payment_method).await {
                Ok(order) => {
                    res.status_code(StatusCode::CREATED);
                    res.success(serde_json::json!({
                        "id": order.id,
                        "resource_id": order.resource_id,
                        "amount": order.amount,
                        "payment_method": order.payment_method,
                        "status": order.status,
                        "expired_at": order.expired_at,
                    }));
                }
                Err(e) => {
                    res.error(e);
                }
            }
        }
        Err(e) => {
            res.error_msg(
                RswsError::from(ErrorCode::INVALID_REQUEST_FORMAT),
                format!("Invalid request: {}", e)
            );
        }
    }
}

/// 取消订单
#[endpoint(
    parameters(
        ("id", description = "订单ID"),
    ),
    responses(
        (status_code = 200, description = "取消成功"),
        (status_code = 401, description = "未认证"),
        (status_code = 404, description = "订单不存在"),
    )
)]
pub async fn cancel_order(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let id: i64 = req.param("id").unwrap_or(0);

    let _user_id = match res.auth_require_user_id(depot) {
        Some(uid) => uid,
        None => return,
    };

    if id <= 0 {
        res.error_msg(
            RswsError::from(ErrorCode::INVALID_PARAMETER),
            "Invalid order ID"
        );
        return;
    }

    let state = get_state(depot);

    match state.order_service.cancel(id, _user_id).await {
        Ok(()) => {
            res.success(serde_json::json!({
                "id": id,
                "status": "cancelled",
                "message": "Order cancelled successfully"
            }));
        }
        Err(e) => {
            res.error(e);
        }
    }
}

/// 检查订单状态（USDT 支付轮询）
#[endpoint(
    parameters(
        ("id", description = "订单ID"),
    ),
    responses(
        (status_code = 200, description = "成功"),
        (status_code = 404, description = "订单不存在"),
    )
)]
pub async fn check_order_status(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let id: i64 = req.param("id").unwrap_or(0);

    if id <= 0 {
        res.error_msg(
            RswsError::from(ErrorCode::INVALID_PARAMETER),
            "Invalid order ID"
        );
        return;
    }

    let state = get_state(depot);

    match state.order_service.get(id).await {
        Ok(Some(order)) => {
            res.success(serde_json::json!({
                "id": order.id,
                "status": order.status,
                "confirmations": 0,
                "required_confirmations": 3
            }));
        }
        Ok(None) => {
            res.error(RswsError::from(ErrorCode::ORDER_NOT_FOUND));
        }
        Err(e) => {
            res.error(e);
        }
    }
}
