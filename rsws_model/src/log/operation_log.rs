use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// 重用现有的 admin_operation_logs 表结构
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OperationLog {
    pub id: i64,
    pub admin_id: i64,
    pub operation_type: String,
    pub operation_target: Option<String>,
    pub target_id: Option<String>,
    pub operation_content: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOperationLogRequest {
    pub admin_id: i64,
    pub operation_type: String,
    pub operation_target: Option<String>,
    pub target_id: Option<String>,
    pub operation_content: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogQueryRequest {
    pub user_id: Option<i64>,
    pub admin_id: Option<i64>,
    pub operation_type: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}
