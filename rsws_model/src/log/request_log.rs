use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RequestLog {
    pub id: i64,
    pub request_id: String,
    pub method: String,
    pub path: String,
    pub query_params: Option<serde_json::Value>,
    pub headers: Option<serde_json::Value>,
    pub body_size: Option<i32>,
    pub user_id: Option<i64>,
    pub admin_id: Option<i64>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub response_status: Option<i32>,
    pub response_size: Option<i32>,
    pub duration_ms: Option<i32>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRequestLogRequest {
    pub request_id: String,
    pub method: String,
    pub path: String,
    pub query_params: Option<serde_json::Value>,
    pub headers: Option<serde_json::Value>,
    pub body_size: Option<i32>,
    pub user_id: Option<i64>,
    pub admin_id: Option<i64>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRequestLogRequest {
    pub response_status: i32,
    pub response_size: Option<i32>,
    pub duration_ms: i32,
}