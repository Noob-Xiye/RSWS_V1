//! 静态配置
//!
//! 仅保留启动时必需的配置（server/database/redis/encryption）。
//! 所有业务配置（PayPal/区块链/Email/USDT监听等）从数据库读取，
//! 通过 ConfigService 提供。
//!
//! 优先级：环境变量 > config.toml
//! 环境变量前缀：RSWS_，分隔符：_
//! 例如：RSWS_DATABASE_URL 覆盖 database.url

use config::{Config, ConfigError, Environment};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub cors_origins: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RedisConfig {
    pub url: String,
    pub pool_size: usize,
}

#[derive(Debug, Deserialize, Clone)]
pub struct EncryptionConfig {
    pub key: String,
}

/// 应用静态配置（仅 config.toml 中的连接配置）
#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub encryption: EncryptionConfig,
}

pub fn load_config() -> Result<AppConfig, ConfigError> {
    let config = Config::builder()
        // 1. 从 config.toml 读取基础配置
        .add_source(config::File::with_name("config.toml"))
        // 2. 环境变量覆盖（RSWS_ 前缀，支持 RSWS_DATABASE_URL 覆盖 database.url 等）
        .add_source(
            Environment::with_prefix("RSWS")
                .prefix_separator("_")
                .separator("_")
        )
        .build()?;

    config.try_deserialize::<AppConfig>()
}
