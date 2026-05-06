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

#[derive(Debug, Deserialize, Clone)]
pub struct PayPalConfig {
    pub client_id: String,
    pub client_secret: String,
    pub mode: String, // "sandbox" or "live"
    pub return_url: String,
    pub cancel_url: String,
    #[serde(default)]
    pub webhook_id: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct USDTConfig {
    pub trc20_address: String,
    pub erc20_address: String,
    pub trc20_private_key: Option<String>,
    pub erc20_private_key: Option<String>,
    pub confirmations_required: u32,
    #[serde(default)]
    pub trongrid_api_key: Option<String>,
    #[serde(default)]
    pub etherscan_api_key: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub encryption: EncryptionConfig,
    pub paypal: Option<PayPalConfig>,
    pub usdt: Option<USDTConfig>,
}

pub fn load_config() -> Result<AppConfig, ConfigError> {
    let config = Config::builder()
        .add_source(config::File::with_name("config.toml"))
        .build()?;

    config.try_deserialize::<AppConfig>()
}

impl AppConfig {
    /// 获取 PayPal 配置（带默认值）
    pub fn paypal(&self) -> PayPalConfig {
        self.paypal.clone().unwrap_or(PayPalConfig {
            client_id: String::new(),
            client_secret: String::new(),
            mode: "sandbox".to_string(),
            return_url: "http://localhost:3000/payment/success".to_string(),
            cancel_url: "http://localhost:3000/payment/cancel".to_string(),
            webhook_id: None,
        })
    }

    /// 获取 USDT 配置（带默认值）
    pub fn usdt(&self) -> USDTConfig {
        self.usdt.clone().unwrap_or(USDTConfig {
            trc20_address: String::new(),
            erc20_address: String::new(),
            trc20_private_key: None,
            erc20_private_key: None,
            confirmations_required: 3,
            trongrid_api_key: None,
            etherscan_api_key: None,
        })
    }
}