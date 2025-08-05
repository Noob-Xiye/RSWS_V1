use salvo::prelude::*;
use salvo::oapi::endpoint;
use rsws_service::{OrderService, PaymentService};
use rsws_model::payment::*;
use std::sync::Arc;

#[derive(Clone)]
pub struct OrderHandler {
    order_service: Arc<OrderService>,
    payment_service: Arc<PaymentService>,
}

impl OrderHandler {
    pub fn new(
        order_service: Arc<OrderService>,
        payment_service: Arc<PaymentService>,
    ) -> Self {
        Self {
            order_service,
            payment_service,
        }
    }
}

#[endpoint(
    tags("订单管理"),
    responses(
        (status = 200, description = "创建订单成功", body = OrderResponse),
        (status = 400, description = "请求参数错误"),
        (status = 500, description = "服务器内部错误")
    )
)]
pub async fn create_order(
    req: &mut Request,
    res: &mut Response,
    depot: &mut Depot,
) -> Result<(), salvo::Error> {
    let order_handler = depot.obtain::<OrderHandler>().unwrap();
    let user_id = req.headers().get("X-User-ID")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<i64>().ok())
        .ok_or_else(|| salvo::Error::other("Missing or invalid user ID"))?;
    
    let request: CreateOrderRequest = req.parse_json().await
        .map_err(|e| salvo::Error::other(format!("Invalid request: {}", e)))?;
        
    let response = order_handler.order_service
        .create_order(user_id, request)
        .await
        .map_err(|e| salvo::Error::other(format!("Service error: {}", e)))?;
        
    res.render(Json(response));
    Ok(())
}

#[endpoint(
    tags("订单管理"),
    responses(
        (status = 200, description = "获取订单成功", body = OrderResponse),
        (status = 404, description = "订单不存在"),
        (status = 500, description = "服务器内部错误")
    )
)]
pub async fn get_order(
    req: &mut Request,
    res: &mut Response,
    depot: &mut Depot,
) -> Result<(), salvo::Error> {
    let order_handler = depot.obtain::<OrderHandler>().unwrap();
    let user_id = req.headers().get("X-User-ID")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<i64>().ok());
    
    let order_id = req.param::<i64>("id")
        .ok_or_else(|| salvo::Error::other("Missing order ID"))?;
        
    let response = order_handler.order_service
        .get_order_by_id(order_id, user_id)
        .await
        .map_err(|e| salvo::Error::other(format!("Service error: {}", e)))?;
        
    res.render(Json(response));
    Ok(())
}

#[endpoint(
    tags("订单管理"),
    responses(
        (status = 200, description = "支付订单成功", body = PayOrderResponse),
        (status = 400, description = "请求参数错误"),
        (status = 500, description = "服务器内部错误")
    )
)]
pub async fn pay_order(
    req: &mut Request,
    res: &mut Response,
    depot: &mut Depot,
) -> Result<(), salvo::Error> {
    let order_handler = depot.obtain::<OrderHandler>().unwrap();
    let user_id = req.headers().get("X-User-ID")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<i64>().ok())
        .ok_or_else(|| salvo::Error::other("Missing or invalid user ID"))?;
    
    let order_id = req.param::<i64>("id")
        .ok_or_else(|| salvo::Error::other("Missing order ID"))?;
    
    let request: PayOrderRequest = req.parse_json().await
        .map_err(|e| salvo::Error::other(format!("Invalid request: {}", e)))?;
        
    let response = order_handler.payment_service
        .create_payment(order_id, user_id, request)
        .await
        .map_err(|e| salvo::Error::other(format!("Service error: {}", e)))?;
        
    res.render(Json(response));
    Ok(())
}

#[endpoint(
    tags("订单管理"),
    responses(
        (status = 200, description = "取消订单成功"),
        (status = 400, description = "请求参数错误"),
        (status = 500, description = "服务器内部错误")
    )
)]
pub async fn cancel_order(
    req: &mut Request,
    res: &mut Response,
    depot: &mut Depot,
) -> Result<(), salvo::Error> {
    let order_handler = depot.obtain::<OrderHandler>().unwrap();
    let user_id = req.headers().get("X-User-ID")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<i64>().ok())
        .ok_or_else(|| salvo::Error::other("Missing or invalid user ID"))?;
    
    let order_id = req.param::<i64>("id")
        .ok_or_else(|| salvo::Error::other("Missing order ID"))?;
        
    order_handler.order_service
        .cancel_order(order_id, user_id)
        .await
        .map_err(|e| salvo::Error::other(format!("Service error: {}", e)))?;
        
    res.render(Json(serde_json::json!({
        "success": true,
        "message": "订单已取消"
    })));
    Ok(())
}

#[endpoint(
    tags("订单管理"),
    responses(
        (status = 200, description = "获取订单列表成功", body = PaginatedResponse<OrderResponse>),
        (status = 500, description = "服务器内部错误")
    )
)]
pub async fn get_user_orders(
    req: &mut Request,
    res: &mut Response,
    depot: &mut Depot,
) -> Result<(), salvo::Error> {
    let order_handler = depot.obtain::<OrderHandler>().unwrap();
    let user_id = req.headers().get("X-User-ID")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<i64>().ok())
        .ok_or_else(|| salvo::Error::other("Missing or invalid user ID"))?;
    
    let request = OrderListRequest {
        status: req.query::<String>("status")
            .and_then(|s| serde_json::from_str(&format!("\"{}\"", s)).ok()),
        start_date: req.query::<String>("start_date")
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc)),
        end_date: req.query::<String>("end_date")
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc)),
        page: req.query::<u32>("page"),
        page_size: req.query::<u32>("page_size"),
    };
        
    let response = order_handler.order_service
        .get_user_orders(user_id, request)
        .await
        .map_err(|e| salvo::Error::other(format!("Service error: {}", e)))?;
        
    res.render(Json(response));
    Ok(())
}

// 支付相关API
#[endpoint(
    tags("支付管理"),
    responses(
        (status = 200, description = "获取支付方式成功", body = Vec<PaymentMethod>),
        (status = 500, description = "服务器内部错误")
    )
)]
pub async fn get_payment_methods(
    _req: &mut Request,
    res: &mut Response,
    depot: &mut Depot,
) -> Result<(), salvo::Error> {
    let order_handler = depot.obtain::<OrderHandler>().unwrap();
        
    let response = order_handler.payment_service
        .get_payment_methods()
        .await
        .map_err(|e| salvo::Error::other(format!("Service error: {}", e)))?;
        
    res.render(Json(response));
    Ok(())
}

#[endpoint(
    tags("支付管理"),
    responses(
        (status = 200, description = "验证支付成功", body = VerifyPaymentResponse),
        (status = 404, description = "支付记录不存在"),
        (status = 500, description = "服务器内部错误")
    )
)]
pub async fn verify_payment(
    req: &mut Request,
    res: &mut Response,
    depot: &mut Depot,
) -> Result<(), salvo::Error> {
    let order_handler = depot.obtain::<OrderHandler>().unwrap();
    
    let payment_id = req.param::<String>("payment_id")
        .ok_or_else(|| salvo::Error::other("Missing payment ID"))?;
        
    let response = order_handler.payment_service
        .verify_payment(&payment_id)
        .await
        .map_err(|e| salvo::Error::other(format!("Service error: {}", e)))?;
        
    res.render(Json(response));
    Ok(())
}