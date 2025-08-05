use axum::{extract::State, http::StatusCode, response::Json, routing::post, Router};
use serde::{Deserialize, Serialize};
use rsws_service::PaymentService;
use std::sync::Arc;

#[derive(Deserialize)]
struct BlockchainWebhook {
    txid: String,
    from: String,
    to: String,
    amount: String,
    confirmations: u32,
    block_number: u64,
}

pub async fn handle_blockchain_webhook(
    State(payment_service): State<Arc<PaymentService>>,
    Json(webhook): Json<BlockchainWebhook>,
) -> Result<StatusCode, StatusCode> {
    // 根据接收地址和金额查找对应的支付记录
    let payment_id = format!("{}_{}", webhook.to, webhook.amount);
    
    if let Err(e) = payment_service.verify_payment(&payment_id).await {
        log::error!("Failed to verify blockchain payment: {}", e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    
    Ok(StatusCode::OK)
}

pub fn blockchain_webhook_routes() -> Router<Arc<PaymentService>> {
    Router::new()
        .route("/blockchain", post(handle_blockchain_webhook))
}