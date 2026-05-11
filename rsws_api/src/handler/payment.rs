//! 支付处理器
//!
//! 使用 ResponseExt 和 AuthHandler trait 简化样板代码

use crate::state::get_state;
use rsws_common::{AuthHandler, ResponseExt};
use salvo::prelude::*;
use salvo_oapi::endpoint;

/// PayPal Webhook — 接收并处理 PayPal 事件通知
///
/// 事件类型：
/// - CHECKOUT.ORDER.APPROVED: 用户批准支付（PayPal 重定向回调）
/// - PAYMENT.CAPTURE.COMPLETED: 支付已完成（真正的付款确认）
/// - PAYMENT.CAPTURE.DENIED: 支付被拒绝
#[endpoint(
    responses(
        (status_code = 200, description = "处理成功"),
        (status_code = 400, description = "无效载荷"),
    )
)]
pub async fn paypal_webhook(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    // 提取关键请求头（用于签名验证）
    let transmission_id = req
        .headers()
        .get("paypal-transmission-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let transmission_time = req
        .headers()
        .get("paypal-transmission-time")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let transmission_sig = req
        .headers()
        .get("paypal-transmission-sig")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    // 解析事件体
    let event: serde_json::Value = match req.parse_json().await {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("Failed to parse PayPal webhook body: {}", e);
            res.http_error(StatusCode::BAD_REQUEST, "Invalid JSON");
            return;
        }
    };

    let state = get_state(depot);

    // 构造签名验证参数
    let _webhook_id = state
        .config_service
        .get("paypal.webhook_id")
        .await
        .ok()
        .flatten()
        .unwrap_or_default();
    let event_json = event.to_string();
    let headers_for_verify: Vec<(String, String)> = vec![
        (
            "PAYPAL-TRANSMISSION-ID".to_string(),
            transmission_id.unwrap_or_default(),
        ),
        (
            "PAYPAL-TRANSMISSION-TIME".to_string(),
            transmission_time.unwrap_or_default(),
        ),
        (
            "PAYPAL-TRANSMISSION-SIG".to_string(),
            transmission_sig.unwrap_or_default(),
        ),
        (
            "PAYPAL-CERT-URL".to_string(),
            req.headers()
                .get("paypal-cert-url")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("")
                .to_string(),
        ),
        ("CONTENT-TYPE".to_string(), "application/json".to_string()),
    ];

    // 验证 PayPal Webhook 签名（dev 模式下失败不影响处理）
    match state
        .paypal_service
        .verify_webhook(&headers_for_verify, event_json.as_bytes())
        .await
    {
        Ok(true) => {}
        Ok(false) => {
            tracing::warn!("PayPal webhook signature verification failed");
            res.http_error(StatusCode::FORBIDDEN, "Invalid signature");
            return;
        }
        Err(e) => {
            tracing::warn!("PayPal signature verify error (dev mode): {}", e);
        }
    }

    let event_type = event["event_type"].as_str().unwrap_or("UNKNOWN");
    let resource = &event["resource"];

    tracing::info!(
        "PayPal webhook: {} | id={}",
        event_type,
        event["id"].as_str().unwrap_or("?")
    );

    match event_type {
        // 用户批准了 PayPal 订单 / 支付已完成
        "CHECKOUT.ORDER.APPROVED" | "PAYMENT.CAPTURE.COMPLETED" => {
            let paypal_order_id = resource["id"].as_str().unwrap_or("");
            tracing::info!(
                "PayPal order {} completed. Status: {}",
                paypal_order_id,
                resource["status"].as_str().unwrap_or("")
            );

            if let Ok(Some(tx)) = state
                .payment_service
                .get_by_paypal_order(paypal_order_id)
                .await
            {
                let order_id = tx.order_id;
                let tx_id = tx.id;

                if let Err(e) = state.order_service.mark_paid(order_id, "paypal").await {
                    tracing::error!("Failed to mark order {} as paid: {}", order_id, e);
                    res.http_error(StatusCode::INTERNAL_SERVER_ERROR, "Failed to update order");
                    return;
                }
                if let Err(e) = state
                    .payment_service
                    .update_status(tx_id, "completed", Some(paypal_order_id))
                    .await
                {
                    tracing::error!("Failed to update transaction {}: {}", tx_id, e);
                }
                tracing::info!(
                    "Order {} paid via PayPal. TX: {}",
                    order_id,
                    paypal_order_id
                );
            } else {
                tracing::warn!("PayPal order {} not found in our records", paypal_order_id);
            }
            res.success(serde_json::json!({ "status": "processed" }));
        }

        // 支付被拒绝/退款
        "PAYMENT.CAPTURE.DENIED" | "PAYMENT.CAPTURE.REFUNDED" => {
            let paypal_order_id = resource["id"].as_str().unwrap_or("");
            tracing::warn!(
                "PayPal {} {}: {}",
                event_type,
                paypal_order_id,
                resource["status"].as_str().unwrap_or("")
            );

            if let Ok(Some(tx)) = state
                .payment_service
                .get_by_paypal_order(paypal_order_id)
                .await
            {
                let order_id = tx.order_id;
                let _ = state.order_service.cancel(order_id, 0).await;
                let status = if event_type == "PAYMENT.CAPTURE.REFUNDED" {
                    "refunded"
                } else {
                    "failed"
                };
                let _ = state
                    .payment_service
                    .update_status(tx.id, status, None)
                    .await;
            }
            res.success(serde_json::json!({ "status": "processed" }));
        }

        _ => {
            tracing::info!("Unhandled PayPal event: {}", event_type);
            res.success(serde_json::json!({ "status": "ignored" }));
        }
    }
}

/// USDT 支付确认 Webhook — 接收链上 USDT 转账通知
#[endpoint(
    responses(
        (status_code = 200, description = "处理成功"),
        (status_code = 400, description = "无效载荷"),
    )
)]
pub async fn usdt_webhook(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    #[derive(serde::Deserialize)]
    struct UsdtWebhookPayload {
        tx_hash: String,
        from_address: String,
        to_address: String,
        amount: String,
        _block_number: Option<i64>,
        network: String,
    }

    let payload: Result<UsdtWebhookPayload, _> = req.parse_json().await;

    match payload {
        Ok(data) => {
            tracing::info!(
                "USDT webhook: {} {} from {} to {} amount {}",
                data.network,
                data.tx_hash,
                data.from_address,
                data.to_address,
                data.amount
            );

            let state = get_state(depot);

            // 验证 to_address 是否是我们的收款地址
            let expected_address = match data.network.as_str() {
                "tron" => state.blockchain_service.get_trc20_address().await,
                "ethereum" => state.blockchain_service.get_erc20_address().await,
                _ => {
                    tracing::warn!("Unknown USDT network: {}", data.network);
                    res.success(serde_json::json!({ "status": "ignored" }));
                    return;
                }
            };

            if data.to_address.to_lowercase() != expected_address.to_lowercase() {
                tracing::warn!(
                    "USDT webhook: to_address mismatch. Expected: {}, Got: {}",
                    expected_address,
                    data.to_address
                );
                res.success(serde_json::json!({ "status": "ignored" }));
                return;
            }

            tracing::info!(
                "USDT deposit confirmed: {} {} amount {}",
                data.network,
                data.tx_hash,
                data.amount
            );

            // 真实确认由 processor.rs 链上监听完成，这里只记录
            res.success(serde_json::json!({ "status": "received", "tx_hash": data.tx_hash }));
        }
        Err(e) => {
            tracing::error!("USDT webhook parse error: {}", e);
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
