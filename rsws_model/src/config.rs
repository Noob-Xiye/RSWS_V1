//! 配置模型
//!
//! 系统配置、邮件配置、区块链配置等

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 系统配置
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SystemConfig {
    pub id: i64,
    pub config_key: String,
    pub config_value: String,
    pub config_type: String,
    pub description: Option<String>,
    pub is_encrypted: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 邮件配置
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EmailConfig {
    pub id: i64,
    pub provider: String,
    pub host: Option<String>,
    pub port: Option<i32>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub use_tls: bool,
    pub from_email: Option<String>,
    pub from_name: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 区块链插件配置
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BlockchainPluginConfig {
    pub id: i64,
    pub plugin_name: String,
    pub network: String,
    pub rpc_url: Option<String>,
    pub api_key: Option<String>,
    pub contract_address: Option<String>,
    pub config_json: Option<serde_json::Value>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 下载插件配置
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DownloadPluginConfig {
    pub id: i64,
    pub plugin_name: String,
    pub storage_type: String,
    pub storage_config: Option<serde_json::Value>,
    pub max_file_size: Option<i64>,
    pub allowed_extensions: Option<Vec<String>>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 动态配置值的枚举类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConfigValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Json(serde_json::Value),
}

/// API Key 认证配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyConfig {
    /// 会话有效期 (天)
    pub session_expire_days: i32,
    /// 签名有效期 (秒，防重放)
    pub signature_expire_seconds: i32,
    /// 每用户最大会话数
    pub max_sessions_per_user: i32,
    /// 是否启用速率限制
    pub enable_rate_limit: bool,
    /// 默认速率限制 (次/分钟)
    pub default_rate_limit: i32,
}

impl Default for ApiKeyConfig {
    fn default() -> Self {
        Self {
            session_expire_days: 7,
            signature_expire_seconds: 300,
            max_sessions_per_user: 5,
            enable_rate_limit: true,
            default_rate_limit: 100,
        }
    }
}

/// 日志配置
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

/// 更新日志配置请求
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

// ==================== 单元测试 ====================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_key_config_default() {
        let config = ApiKeyConfig::default();
        assert_eq!(config.session_expire_days, 7);
        assert_eq!(config.signature_expire_seconds, 300);
        assert!(config.enable_rate_limit);
    }

    #[test]
    fn test_log_config_default() {
        let config = LogConfig::default();
        assert_eq!(config.level, "info");
        assert!(config.enable_database);
        assert!(!config.enable_file);
    }

    #[test]
    fn test_config_value_string() {
        let value = ConfigValue::String("test".to_string());
        let json = serde_json::to_string(&value).unwrap();
        assert_eq!(json, "\"test\"");
    }

    #[test]
    fn test_config_value_number() {
        let value = ConfigValue::Number(42.5);
        let json = serde_json::to_string(&value).unwrap();
        assert_eq!(json, "42.5");
    }

    #[test]
    fn test_config_value_boolean() {
        let value = ConfigValue::Boolean(true);
        let json = serde_json::to_string(&value).unwrap();
        assert_eq!(json, "true");
    }
}
