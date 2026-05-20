//! 订单处理器
//!
//! 使用 ResponseExt 和 AuthHandler trait 简化样板代码

use crate::state::get_state;
use rsws_common::{error_code::ErrorCode, AuthHandler, ResponseExt, RswsError};
use salvo::prelude::*;
use salvo_oapi::endpoint;
use salvo_oapi::ToSchema;
use serde::Deserialize;

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
    let page_size: i32 = req.query("page_size").unwrap_or(20);

    let state = get_state(depot);

    match state
        .order_service
        .list_detail_by_user(user_id, page, page_size)
        .await
    {
        Ok((orders, total)) => {
            let total_pages = if page_size > 0 {
                (total + page_size as i64 - 1) / page_size as i64
            } else {
                0
            };
            res.success(serde_json::json!({
                "items": orders,
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

/// 获取订单详情
#[endpoint(
    parameters(
        ("id", Path, description = "订单ID"),
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
            "Invalid order ID",
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
                    format!("Unsupported payment method: {}", data.payment_method),
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

            match state
                .order_service
                .create(user_id, data.resource_id, amount, &data.payment_method)
                .await
            {
                Ok(order) => {
                    // 如果是 PayPal 支付，需要创建 PayPal 订单
                    if data.payment_method == "paypal" {
                        match state
                            .paypal_service
                            .create_order(
                                amount as f64 / 100.0, // 分转元
                                "USDT",
                                &format!("Resource #{}", data.resource_id),
                                order.id,
                            )
                            .await
                        {
                            Ok(paypal_order) => {
                                let paypal_order_id =
                                    paypal_order["id"].as_str().unwrap_or("").to_string();
                                let approve_url = paypal_order["links"]
                                    .as_array()
                                    .and_then(|links| links.iter().find(|l| l["rel"] == "approve"))
                                    .and_then(|l| l["href"].as_str().map(|s| s.to_string()));

                                // 创建支付交易记录
                                let _ = state
                                    .payment_service
                                    .create(order.id, user_id, amount, "USDT", "paypal")
                                    .await;

                                res.status_code(StatusCode::CREATED);
                                res.success(serde_json::json!({
                                    "id": order.id,
                                    "resource_id": order.resource_id,
                                    "amount": order.amount,
                                    "payment_method": order.payment_method,
                                    "status": order.status,
                                    "paypal_order_id": paypal_order_id,
                                    "approve_url": approve_url,
                                }));
                            }
                            Err(e) => {
                                tracing::error!("Failed to create PayPal order: {}", e);
                                res.status_code(StatusCode::CREATED);
                                res.success(serde_json::json!({
                                    "id": order.id,
                                    "resource_id": order.resource_id,
                                    "amount": order.amount,
                                    "payment_method": order.payment_method,
                                    "status": order.status,
                                    "message": "Order created but PayPal unavailable. Please use USDT payment.",
                                }));
                            }
                        }
                    } else {
                        res.status_code(StatusCode::CREATED);
                        res.success(serde_json::json!({
                            "id": order.id,
                            "resource_id": order.resource_id,
                            "amount": order.amount,
                            "payment_method": order.payment_method,
                            "status": order.status,
                        }));
                    }
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

/// 取消订单
#[endpoint(
    parameters(
        ("id", Path, description = "订单ID"),
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
            "Invalid order ID",
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

/// 退款订单
#[endpoint(
    parameters(
        ("id", Path, description = "订单ID"),
    ),
    responses(
        (status_code = 200, description = "退款成功"),
        (status_code = 403, description = "非管理员"),
        (status_code = 404, description = "订单不存在"),
    )
)]
pub async fn refund_order(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let id: i64 = req.param("id").unwrap_or(0);

    if id <= 0 {
        res.error_msg(
            RswsError::from(ErrorCode::INVALID_PARAMETER),
            "Invalid order ID",
        );
        return;
    }

    let is_admin: bool = depot.get("is_admin").copied().unwrap_or(false);
    if !is_admin {
        res.http_error(StatusCode::FORBIDDEN, "Admin access required");
        return;
    }

    let state = get_state(depot);
    match state.order_service.refund(id).await {
        Ok(()) => {
            res.success(serde_json::json!({
                "id": id,
                "status": "refunded",
                "message": "Order refunded successfully"
            }));
        }
        Err(e) => {
            res.error(e);
        }
    }
}

/// 完成订单
#[endpoint(
    parameters(
        ("id", Path, description = "订单ID"),
    ),
    responses(
        (status_code = 200, description = "完成成功"),
        (status_code = 403, description = "非管理员"),
        (status_code = 404, description = "订单不存在"),
    )
)]
pub async fn complete_order(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let id: i64 = req.param("id").unwrap_or(0);

    if id <= 0 {
        res.error_msg(
            RswsError::from(ErrorCode::INVALID_PARAMETER),
            "Invalid order ID",
        );
        return;
    }

    let is_admin: bool = depot.get("is_admin").copied().unwrap_or(false);
    if !is_admin {
        res.http_error(StatusCode::FORBIDDEN, "Admin access required");
        return;
    }

    let state = get_state(depot);
    match state.order_service.complete(id).await {
        Ok(()) => {
            res.success(serde_json::json!({
                "id": id,
                "status": "completed",
                "message": "Order completed successfully"
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
        ("id", Path, description = "订单ID"),
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
            "Invalid order ID",
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

/// 检查用户是否已购买某资源
#[endpoint(
    parameters(
        ("resource_id", Path, description = "资源ID"),
    ),
    responses(
        (status_code = 200, description = "成功"),
        (status_code = 401, description = "未认证"),
    )
)]
pub async fn check_purchase(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let resource_id: i64 = req.param("resource_id").unwrap_or(0);

    let user_id = match res.auth_require_user_id(depot) {
        Some(id) => id,
        None => return,
    };

    if resource_id <= 0 {
        res.error_msg(
            RswsError::from(ErrorCode::INVALID_PARAMETER),
            "Invalid resource ID",
        );
        return;
    }

    let state = get_state(depot);

    match state
        .order_service
        .check_purchased(user_id, resource_id)
        .await
    {
        Ok(purchased) => {
            res.success(serde_json::json!({
                "purchased": purchased,
                "resource_id": resource_id
            }));
        }
        Err(e) => {
            res.error(e);
        }
    }
}

/// 获取资源下载信息
#[endpoint(
    parameters(
        ("resource_id", Path, description = "资源ID"),
    ),
    responses(
        (status_code = 200, description = "成功"),
        (status_code = 401, description = "未认证"),
        (status_code = 403, description = "未购买，无权下载"),
    )
)]
pub async fn get_resource_download(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let resource_id: i64 = req.param("resource_id").unwrap_or(0);

    let user_id = match res.auth_require_user_id(depot) {
        Some(id) => id,
        None => return,
    };

    if resource_id <= 0 {
        res.error_msg(
            RswsError::from(ErrorCode::INVALID_PARAMETER),
            "Invalid resource ID",
        );
        return;
    }

    let state = get_state(depot);

    // 检查是否已购买
    match state
        .order_service
        .check_purchased(user_id, resource_id)
        .await
    {
        Ok(false) => {
            res.error_msg(
                RswsError::from(ErrorCode::AUTH_PERMISSION_DENIED),
                "Please purchase this resource first",
            );
        }
        Ok(true) => {
            // 获取资源下载链接
            match state.resource_service.get(resource_id).await {
                Ok(Some(resource)) => {
                    // 递增下载计数
                    let _ = state
                        .resource_service
                        .increment_download_count(resource_id)
                        .await;

                    res.success(serde_json::json!({
                        "file_url": resource.file_url,
                        "file_name": format!("{}.zip", resource.title.replace(" ", "_"))
                    }));
                }
                Ok(None) => {
                    res.error(RswsError::from(ErrorCode::RESOURCE_NOT_FOUND));
                }
                Err(e) => {
                    res.error(e);
                }
            }
        }
        Err(e) => {
            res.error(e);
        }
    }
}

// ==================== 管理员订单 ====================

/// 管理员订单查询参数
#[derive(Debug, Deserialize, ToSchema)]
pub struct AdminOrderQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub status: Option<String>,
    pub user_id: Option<i64>,
    pub payment_method: Option<String>,
}

/// 管理员获取全部订单列表
#[endpoint]
pub async fn admin_list_orders(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let is_admin: bool = depot.get("is_admin").copied().unwrap_or(false);
    if !is_admin {
        res.http_error(StatusCode::FORBIDDEN, "Admin access required");
        return;
    }

    let query: AdminOrderQuery = req.parse_queries().unwrap_or(AdminOrderQuery {
        page: Some(1),
        page_size: Some(20),
        status: None,
        user_id: None,
        payment_method: None,
    });

    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).clamp(1, 100);

    let state = get_state(depot);

    match state
        .order_service
        .admin_list_orders(
            query.status.as_deref(),
            query.user_id,
            query.payment_method.as_deref(),
            page,
            page_size,
        )
        .await
    {
        Ok((orders, total)) => {
            let total_pages = if page_size > 0 {
                (total + page_size - 1) / page_size
            } else {
                0
            };
            res.success(serde_json::json!({
                "items": orders,
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
