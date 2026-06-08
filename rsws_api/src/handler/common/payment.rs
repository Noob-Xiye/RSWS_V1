//! 支付回调处理器
//!
//! USDT 地址获取和 PayPal 支付回调（成功/取消）。

use crate::state::get_state;
use rsws_common::{AuthHandler, ResponseExt};
use salvo::prelude::*;
use salvo_oapi::endpoint;

/// 获取 USDT 收款地址
#[endpoint(
    responses(
        (status_code = 200, description = "成功"),
        (status_code = 401, description = "未认证"),
    )
)]
pub async fn get_usdt_address(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let network: String = req.param("network").unwrap_or_else(|| "tron".to_string());

    let user_id = match res.auth_require_user_id(depot) {
        Some(id) => id,
        None => return,
    };

    let state = get_state(depot);

    // 从 DB 配置读取合约地址
    let blockchain_configs = match state.config_service.get_blockchain_configs().await {
        Ok(configs) => configs,
        Err(e) => {
            tracing::error!("Failed to load blockchain configs: {}", e);
            res.http_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to load blockchain config",
            );
            return;
        }
    };

    let bc_config = blockchain_configs
        .iter()
        .find(|c| c.network == network && c.is_active);
    let contract = match bc_config {
        Some(c) => c.usdt_contract.clone(),
        None => {
            res.http_error(
                StatusCode::BAD_REQUEST,
                "Unsupported or inactive network, use 'tron' or 'ethereum'",
            );
            return;
        }
    };

    let address = match network.as_str() {
        "tron" => state.blockchain_service.get_trc20_address().await,
        "ethereum" => state.blockchain_service.get_erc20_address().await,
        _ => {
            res.http_error(StatusCode::BAD_REQUEST, "Unsupported network");
            return;
        }
    };

    tracing::info!(
        "User {} requesting USDT address for network: {}",
        user_id,
        network
    );

    res.success(serde_json::json!({
        "network": network,
        "address": address,
        "contract": contract
    }));
}

/// PayPal 支付成功回调
///
/// PayPal 重定向回此端点时携带 order_id 参数（由前端传入）。
/// 真实确认由 PayPal Webhook 完成，这里仅做重定向。
#[endpoint(
    responses(
        (status_code = 302, description = "重定向到前端"),
    )
)]
pub async fn paypal_success(req: &mut Request, _depot: &mut Depot, res: &mut Response) {
    let _order_id: Option<i64> = req.query("order_id");
    // 真实订单状态由 PayPal Webhook 更新（CHECKOUT.ORDER.APPROVED / PAYMENT.CAPTURE.COMPLETED）
    // 这里仅做重定向
    res.status_code(StatusCode::FOUND);
    res.add_header("Location", "/payment/success", true).ok();
}

/// PayPal 支付取消回调
#[endpoint(
    responses(
        (status_code = 302, description = "重定向到前端"),
    )
)]
pub async fn paypal_cancel(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let order_id: Option<i64> = req.query("order_id");

    if let Some(order_id) = order_id {
        let state = get_state(depot);
        match state.order_service.cancel(order_id, 0).await {
            Ok(()) => tracing::info!("PayPal cancel: order {} cancelled", order_id),
            Err(e) => tracing::error!("Failed to cancel order {}: {}", order_id, e),
        }
    }

    res.status_code(StatusCode::FOUND);
    res.add_header("Location", "/payment/cancel", true).ok();
}
