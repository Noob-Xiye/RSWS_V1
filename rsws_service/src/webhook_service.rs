use chrono::Utc;
use hex;
use hmac::{Hmac, Mac};
use rsws_common::error::ServiceError;
use rsws_common::snowflake;
use rsws_model::log::{CreateWebhookLogRequest, UpdateWebhookLogRequest, WebhookLog};
use serde_json::Value;
use sha2::Sha256;
use sqlx::PgPool;
use std::sync::Arc;
use tracing::{error, info, warn};

type HmacSha256 = Hmac<Sha256>;

pub struct WebhookService {
    db_pool: PgPool,
    log_service: Arc<crate::log_service::LogService>,
}

impl WebhookService {
    pub fn new(db_pool: PgPool, log_service: Arc<crate::log_service::LogService>) -> Self {
        Self {
            db_pool,
            log_service,
        }
    }

    // 记录Webhook日志
    pub async fn log_webhook(&self, request: CreateWebhookLogRequest) -> Result<i64, ServiceError> {
        let log_id = snowflake::next_id();

        sqlx::query!(
            r#"
            INSERT INTO webhook_logs (
                id, webhook_type, source, event_type, payload, headers, signature,
                status, ip_address, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
            log_id,
            request.webhook_type,
            request.source,
            request.event_type,
            request.payload,
            request.headers,
            request.signature,
            "pending",
            request.ip_address,
            Utc::now()
        )
        .execute(&self.db_pool)
        .await?;

        info!(
            "Webhook logged: {} from {} - {}",
            request.event_type, request.source, log_id
        );
        Ok(log_id)
    }

    // 更新Webhook处理状态
    pub async fn update_webhook_status(
        &self,
        log_id: i64,
        request: UpdateWebhookLogRequest,
    ) -> Result<(), ServiceError> {
        sqlx::query!(
            r#"
            UPDATE webhook_logs 
            SET status = $2, response_code = $3, response_message = $4, 
                retry_count = COALESCE($5, retry_count), processed_at = $6
            WHERE id = $1
            "#,
            log_id,
            request.status,
            request.response_code,
            request.response_message,
            request.retry_count,
            Utc::now()
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    // 验证PayPal Webhook签名
    pub async fn verify_paypal_signature(
        &self,
        payload: &str,
        signature: &str,
        webhook_secret: &str,
    ) -> Result<bool, ServiceError> {
        let mut mac = HmacSha256::new_from_slice(webhook_secret.as_bytes())
            .map_err(|e| ServiceError::ValidationError(format!("Invalid webhook secret: {}", e)))?;

        mac.update(payload.as_bytes());
        let expected_signature = hex::encode(mac.finalize().into_bytes());

        // PayPal使用SHA256签名
        let provided_signature = signature.strip_prefix("sha256=").unwrap_or(signature);

        Ok(expected_signature.eq_ignore_ascii_case(provided_signature))
    }

    // 处理PayPal Webhook
    pub async fn handle_paypal_webhook(
        &self,
        payload: Value,
        headers: Option<Value>,
        signature: Option<String>,
        ip_address: Option<String>,
    ) -> Result<(), ServiceError> {
        let event_type = payload
            .get("event_type")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        // 记录Webhook
        let log_request = CreateWebhookLogRequest {
            webhook_type: "payment".to_string(),
            source: "paypal".to_string(),
            event_type: event_type.to_string(),
            payload: payload.clone(),
            headers,
            signature: signature.clone(),
            ip_address,
        };

        let log_id = self.log_webhook(log_request).await?;

        // 处理不同类型的事件
        let result = match event_type {
            "PAYMENT.CAPTURE.COMPLETED" => self.handle_payment_completed(&payload).await,
            "PAYMENT.CAPTURE.DENIED" => self.handle_payment_denied(&payload).await,
            "PAYMENT.CAPTURE.REFUNDED" => self.handle_payment_refunded(&payload).await,
            _ => {
                warn!("Unhandled PayPal webhook event: {}", event_type);
                Ok(())
            }
        };

        // 更新处理状态
        let update_request = match result {
            Ok(_) => UpdateWebhookLogRequest {
                status: "success".to_string(),
                response_code: Some(200),
                response_message: Some("Processed successfully".to_string()),
                retry_count: None,
            },
            Err(ref e) => UpdateWebhookLogRequest {
                status: "failed".to_string(),
                response_code: Some(500),
                response_message: Some(e.to_string()),
                retry_count: None,
            },
        };

        self.update_webhook_status(log_id, update_request).await?;
        result
    }

    // 处理PayPal支付完成事件
    pub async fn handle_paypal_payment_completed(
        &self,
        payment_id: &str,
        external_transaction_id: &str,
    ) -> Result<(), ServiceError> {
        info!("Processing PayPal payment completed: {}", payment_id);

        // 使用新的智能支付处理逻辑
        self.payment_service
            .process_payment_completion(payment_id, external_transaction_id)
            .await?;

        // 记录Webhook处理日志
        self.log_webhook_processing(
            "paypal_payment_completed",
            payment_id,
            "success",
            "Payment processed successfully with smart routing",
        )
        .await?;

        Ok(())
    }

    // 处理支付完成事件
    async fn handle_payment_completed(&self, payload: &Value) -> Result<(), ServiceError> {
        let capture_id = payload
            .get("resource")
            .and_then(|r| r.get("id"))
            .and_then(|id| id.as_str())
            .ok_or_else(|| ServiceError::ValidationError("Missing capture ID".to_string()))?;

        // 更新支付交易状态
        sqlx::query!(
            "UPDATE payment_transactions SET status = 'completed', completed_at = $1 WHERE provider_transaction_id = $2",
            Utc::now(),
            capture_id
        )
        .execute(&self.db_pool)
        .await?;

        // 更新订单状态
        sqlx::query!(
            r#"
            UPDATE orders SET status = 'paid', updated_at = $1 
            WHERE id = (SELECT order_id FROM payment_transactions WHERE provider_transaction_id = $2)
            "#,
            Utc::now(),
            capture_id
        )
        .execute(&self.db_pool)
        .await?;

        info!("Payment completed for capture: {}", capture_id);
        Ok(())
    }

    // 处理支付拒绝事件
    async fn handle_payment_denied(&self, payload: &Value) -> Result<(), ServiceError> {
        let capture_id = payload
            .get("resource")
            .and_then(|r| r.get("id"))
            .and_then(|id| id.as_str())
            .ok_or_else(|| ServiceError::ValidationError("Missing capture ID".to_string()))?;

        // 更新支付交易状态
        sqlx::query!(
            "UPDATE payment_transactions SET status = 'failed' WHERE provider_transaction_id = $1",
            capture_id
        )
        .execute(&self.db_pool)
        .await?;

        // 更新订单状态
        sqlx::query!(
            r#"
            UPDATE orders SET status = 'failed', updated_at = $1 
            WHERE id = (SELECT order_id FROM payment_transactions WHERE provider_transaction_id = $2)
            "#,
            Utc::now(),
            capture_id
        )
        .execute(&self.db_pool)
        .await?;

        warn!("Payment denied for capture: {}", capture_id);
        Ok(())
    }

    // 处理退款事件
    async fn handle_payment_refunded(&self, payload: &Value) -> Result<(), ServiceError> {
        let refund_id = payload
            .get("resource")
            .and_then(|r| r.get("id"))
            .and_then(|id| id.as_str())
            .ok_or_else(|| ServiceError::ValidationError("Missing refund ID".to_string()))?;

        let capture_id = payload
            .get("resource")
            .and_then(|r| r.get("links"))
            .and_then(|links| links.as_array())
            .and_then(|arr| {
                arr.iter()
                    .find(|link| link.get("rel").and_then(|r| r.as_str()) == Some("up"))
            })
            .and_then(|link| link.get("href"))
            .and_then(|href| href.as_str())
            .and_then(|url| url.split('/').last())
            .ok_or_else(|| {
                ServiceError::ValidationError("Missing capture ID in refund".to_string())
            })?;

        // 更新订单状态为退款
        sqlx::query!(
            r#"
            UPDATE orders SET status = 'refunded', updated_at = $1 
            WHERE id = (SELECT order_id FROM payment_transactions WHERE provider_transaction_id = $2)
            "#,
            Utc::now(),
            capture_id
        )
        .execute(&self.db_pool)
        .await?;

        info!(
            "Payment refunded: {} for capture: {}",
            refund_id, capture_id
        );
        Ok(())
    }

    // 获取Webhook日志
    pub async fn get_webhook_logs(
        &self,
        webhook_type: Option<String>,
        source: Option<String>,
        status: Option<String>,
        page: Option<u32>,
        page_size: Option<u32>,
    ) -> Result<Vec<WebhookLog>, ServiceError> {
        let page_size = page_size.unwrap_or(50).min(200) as i64;
        let offset = (page.unwrap_or(1) - 1) as i64 * page_size;

        let logs = sqlx::query_as::<_, WebhookLog>(
            r#"
            SELECT * FROM webhook_logs 
            WHERE ($1::text IS NULL OR webhook_type = $1)
              AND ($2::text IS NULL OR source = $2)
              AND ($3::text IS NULL OR status = $3)
            ORDER BY created_at DESC
            LIMIT $4 OFFSET $5
            "#,
        )
        .bind(webhook_type)
        .bind(source)
        .bind(status)
        .bind(page_size)
        .bind(offset)
        .fetch_all(&self.db_pool)
        .await?;

        Ok(logs)
    }

    // 重试失败的Webhook
    pub async fn retry_webhook(&self, log_id: i64) -> Result<(), ServiceError> {
        let webhook = sqlx::query_as::<_, WebhookLog>("SELECT * FROM webhook_logs WHERE id = $1")
            .bind(log_id)
            .fetch_optional(&self.db_pool)
            .await?
            .ok_or_else(|| ServiceError::NotFound("Webhook log not found".to_string()))?;

        if webhook.retry_count >= 3 {
            return Err(ServiceError::ValidationError(
                "Maximum retry count exceeded".to_string(),
            ));
        }

        // 增加重试次数
        sqlx::query!(
            "UPDATE webhook_logs SET retry_count = retry_count + 1 WHERE id = $1",
            log_id
        )
        .execute(&self.db_pool)
        .await?;

        // 重新处理Webhook
        match webhook.source.as_str() {
            "paypal" => {
                self.handle_paypal_webhook(
                    webhook.payload,
                    webhook.headers,
                    webhook.signature,
                    webhook.ip_address,
                )
                .await
            }
            _ => Err(ServiceError::ValidationError(format!(
                "Unsupported webhook source: {}",
                webhook.source
            ))),
        }
    }
}
