use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ErrorLog {
    pub id: i64,
    pub error_type: String,
    pub error_message: String,
    pub stack_trace: Option<String>,
    pub module: String,
    pub function_name: Option<String>,
    pub user_id: Option<i64>,
    pub admin_id: Option<i64>,
    pub request_id: Option<String>,
    pub context: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateErrorLogRequest {
    pub error_type: String,
    pub error_message: String,
    pub stack_trace: Option<String>,
    pub module: String,
    pub function_name: Option<String>,
    pub user_id: Option<i64>,
    pub admin_id: Option<i64>,
    pub request_id: Option<String>,
    pub context: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}
