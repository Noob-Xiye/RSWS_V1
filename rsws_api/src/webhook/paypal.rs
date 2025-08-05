use axum::{extract::State, http::StatusCode, response::Json, routing::post, Router};
use serde::{Deserialize, Serialize};
use rsws_service::PaymentService;
use std::sync::Arc;

#[derive(Deserialize)]
struct PayPalWebhook {
    id: String,
    event_type: String,
    resource: PayPalResource,
}

#[derive(Deserialize)]
struct PayPalResource {
    id: String,
    status: String,
    amount: PayPalAmount,
}

#[derive(Deserialize)]
struct PayPalAmount {
    total: String,
    currency: String,
}

pub async fn handle_paypal_webhook(
    State(payment_service): State<Arc<PaymentService>>,
    Json(webhook): Json<PayPalWebhook>,
) -> Result<StatusCode, StatusCode> {
    match webhook.event_type.as_str() {
        "PAYMENT.CAPTURE.COMPLETED" => {
            // 处理支付完成
            if let Err(e) = payment_service.verify_payment(&webhook.resource.id).await {
                log::error!("Failed to verify PayPal payment: {}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        },
        "PAYMENT.CAPTURE.DENIED" => {
            // 处理支付失败
            log::info!("PayPal payment denied: {}", webhook.resource.id);
        },
        _ => {
            log::info!("Unhandled PayPal webhook event: {}", webhook.event_type);
        }
    }
    
    Ok(StatusCode::OK)
}

pub fn paypal_webhook_routes() -> Router<Arc<PaymentService>> {
    Router::new()
        .route("/paypal", post(handle_paypal_webhook))
}