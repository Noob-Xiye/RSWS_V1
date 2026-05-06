//! 支付处理器
//!
//! 使用 ResponseExt 和 AuthHandler trait 简化样板代码

use salvo::prelude::*;
use salvo_oapi::endpoint;
use rsws_common::{ResponseExt, AuthHandler};
use crate::state::get_state;

/// PayPal Webhook
#[endpoint(
    responses(
        (status_code = 200, description = "处理成功"),
        (status_code = 400, description = "无效载荷"),
    )
)]
pub async fn paypal_webhook(req: &mut Request, _depot: &mut Depot, res: &mut Response) {
    let body = req.parse_json::<serde_json::Value>().await;

    match body {
        Ok(data) => {
            // TODO: 验证 PayPal 签名
            // TODO: 处理支付事件
            tracing::info!("PayPal webhook received: {:?}", data);
            res.success(serde_json::json!({ "status": "ok" }));
        }
        Err(e) => {
            tracing::error!("PayPal webhook error: {}", e);
            res.http_error(StatusCode::BAD_REQUEST, "Invalid payload");
        }
    }
}

/// USDT 支付确认 Webhook (内部)
#[endpoint(
    responses(
        (status_code = 200, description = "处理成功"),
        (status_code = 400, description = "无效载荷"),
    )
)]
pub async fn usdt_webhook(req: &mut Request, _depot: &mut Depot, res: &mut Response) {
    let body = req.parse_json::<serde_json::Value>().await;

    match body {
        Ok(data) => {
            // TODO: 验证交易
            // TODO: 更新订单状态
            tracing::info!("USDT webhook received: {:?}", data);
            res.success(serde_json::json!({ "status": "ok" }));
        }
        Err(e) => {
            tracing::error!("USDT webhook error: {}", e);
            res.http_error(StatusCode::BAD_REQUEST, "Invalid payload");
        }
    }
}

/// 获取 USDT 收款地址
#[endpoint(
    parameters(
        ("network", description = "网络类型: tron/ethereum"),
    ),
    responses(
        (status_code = 200, description = "成功"),
        (status_code = 401, description = "未认证"),
    )
)]
pub async fn get_usdt_address(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let network: String = req.param("network").unwrap_or_else(|| "tron".to_string());

    let _user_id = match res.auth_require_user_id(depot) {
        Some(id) => id,
        None => return,
    };

    let state = get_state(depot);

    // 合约地址（固定值，真实 USDT 合约）
    let contract = match network.as_str() {
        "tron" => "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t",
        "ethereum" => "0xdAC17F958D2ee523a2206206994597C13D831ec7",
        _ => {
            res.http_error(StatusCode::BAD_REQUEST, "Unsupported network, use 'tron' or 'ethereum'");
            return;
        }
    };

    // 从数据库获取收款地址（async，fallback 到配置文件）
    let address = match network.as_str() {
        "tron" => state.blockchain_service.get_trc20_address().await,
        "ethereum" => state.blockchain_service.get_erc20_address().await,
        _ => {
            res.http_error(StatusCode::BAD_REQUEST, "Unsupported network");
            return;
        }
    };

    tracing::info!("User {} requesting USDT address for network: {}", _user_id, network);

    res.success(serde_json::json!({
        "network": network,
        "address": address,
        "contract": contract
    }));
}

/// PayPal 支付成功回调
#[endpoint(
    responses(
        (status_code = 302, description = "重定向到前端"),
    )
)]
pub async fn paypal_success(req: &mut Request, _depot: &mut Depot, res: &mut Response) {
    let _order_id: i64 = req.query("order_id").unwrap_or(0);
    let _token: Option<String> = req.query("token");

    // TODO: 验证支付并更新订单

    // 重定向到前端支付成功页面
    res.status_code(StatusCode::FOUND);
    res.add_header("Location", "/payment/success", true).unwrap();
}

/// PayPal 支付取消回调
#[endpoint(
    responses(
        (status_code = 302, description = "重定向到前端"),
    )
)]
pub async fn paypal_cancel(req: &mut Request, _depot: &mut Depot, res: &mut Response) {
    let _order_id: i64 = req.query("order_id").unwrap_or(0);

    // TODO: 更新订单状态为取消

    // 重定向到前端支付取消页面
    res.status_code(StatusCode::FOUND);
    res.add_header("Location", "/payment/cancel", true).unwrap();
}
