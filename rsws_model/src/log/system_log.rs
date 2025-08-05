use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warn => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR"),
            LogLevel::Fatal => write!(f, "FATAL"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SystemLog {
    pub id: i64,
    pub log_level: String,
    pub module: String,
    pub message: String,
    pub context: Option<serde_json::Value>,
    pub user_id: Option<i64>,
    pub admin_id: Option<i64>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub request_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSystemLogRequest {
    pub log_level: LogLevel,
    pub module: String,
    pub message: String,
    pub context: Option<serde_json::Value>,
    pub user_id: Option<i64>,
    pub admin_id: Option<i64>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub request_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogQueryRequest {
    pub log_level: Option<LogLevel>,
    pub module: Option<String>,
    pub user_id: Option<i64>,
    pub admin_id: Option<i64>,
    pub request_id: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}
