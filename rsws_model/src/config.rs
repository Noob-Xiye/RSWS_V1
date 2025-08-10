use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SystemConfig {
    pub id: i32,
    pub config_key: String,
    pub config_value: String,
    pub config_type: String,
    pub description: Option<String>,
    pub is_encrypted: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EmailConfig {
    pub id: i32,
    pub provider: String,
    pub host: Option<String>,
    pub port: Option<i32>,
    pub username: Option<String>,
    pub password_encrypted: Option<String>,
    pub use_tls: bool,
    pub from_email: Option<String>,
    pub from_name: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BlockchainPluginConfig {
    pub id: i32,
    pub plugin_name: String,
    pub network: String,
    pub rpc_url: Option<String>,
    pub api_key_encrypted: Option<String>,
    pub contract_address: Option<String>,
    pub config_json: Option<serde_json::Value>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DownloadPluginConfig {
    pub id: i32,
    pub plugin_name: String,
    pub storage_type: String,
    pub storage_config: Option<serde_json::Value>,
    pub max_file_size: Option<i64>,
    pub allowed_extensions: Option<Vec<String>>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// 动态配置值的枚举类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConfigValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Json(serde_json::Value),
}

// 在现有配置结构体中添加
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfig {
    pub level: String,
    pub enable_database: bool,
    pub enable_file: bool,
    pub file_path: Option<String>,
    pub max_file_size: Option<u64>,
    pub retention_days: Option<u32>,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            enable_database: true,
            enable_file: false,
            file_path: None,
            max_file_size: Some(10 * 1024 * 1024), // 10MB
            retention_days: Some(30),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateLogConfigRequest {
    pub level: Option<String>,
    pub enable_database_logging: Option<bool>,
    pub enable_file_logging: Option<bool>,
    pub log_file_path: Option<String>,
    pub max_file_size: Option<i64>,
    pub retention_days: Option<i32>,
    pub enable_error_logging: Option<bool>,
    pub enable_operation_logging: Option<bool>,
    pub enable_payment_logging: Option<bool>,
}
