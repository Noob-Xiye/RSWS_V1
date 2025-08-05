use config::{Config, ConfigError};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub cors_origins: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
}

#[derive(Debug, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    pub pool_size: usize,
}

#[derive(Debug, Deserialize)]
pub struct EncryptionConfig {
    pub key: String,
}

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub encryption: EncryptionConfig, // 新增加密配置
}

pub fn load_config() -> Result<AppConfig, ConfigError> {
    let config = Config::builder()
        .add_source(config::File::with_name("config.toml"))
        .build()?;

    config.try_deserialize::<AppConfig>()
}
