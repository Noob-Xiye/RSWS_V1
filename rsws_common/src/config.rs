//! 静态配置
//!
//! 仅保留启动时必需的配置（server/database/redis/encryption）。
//! 所有业务配置（PayPal/区块链/Email/USDT监听等）从数据库读取，
//! 通过 ConfigService 提供。

use config::{Config, ConfigError};
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
        .add_source(config::File::with_name("config.toml"))
        .build()?;

    config.try_deserialize::<AppConfig>()
}
