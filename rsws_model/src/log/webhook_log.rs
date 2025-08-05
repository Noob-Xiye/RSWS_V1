use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WebhookLog {
    pub id: i64,
    pub webhook_type: String,
    pub source: String,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub headers: Option<serde_json::Value>,
    pub signature: Option<String>,
    pub status: String,
    pub response_code: Option<i32>,
    pub response_message: Option<String>,
    pub processed_at: Option<DateTime<Utc>>,
    pub retry_count: i32,
    pub ip_address: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWebhookLogRequest {
    pub webhook_type: String,
    pub source: String,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub headers: Option<serde_json::Value>,
    pub signature: Option<String>,
    pub ip_address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateWebhookLogRequest {
    pub status: String,
    pub response_code: Option<i32>,
    pub response_message: Option<String>,
    pub retry_count: Option<i32>,
}
