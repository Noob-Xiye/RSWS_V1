//! USDT Transaction Monitoring Service
//!
//! 内置 epusdt 能力，提供 USDT (TRC20/ERC20) 交易监听服务。
//!
//! # 功能
//!
//! - 监听 TronGrid/Etherscan API 检测 USDT 转账
//! - 匹配订单金额，自动确认支付
//! - 幂等处理，防止重复确认
//! - 支持多收款地址轮询
//!
//! # 使用
//!
//! ```rust,no_run
//! use rsws_usdt::UsdtListener;
//! use rsws_usdt::UsdtConfig;
//! use sqlx::PgPool;
//!
//! #[tokio::main]
//! async fn main() {
//!     // 示例：创建 USDT 监听器
//!     // 实际使用时需要配置数据库连接和配置
//!     // let pool = PgPool::connect("...").await.unwrap();
//!     // let config = UsdtConfig::default();
//!     // let listener = UsdtListener::new(pool, config, None);
//!     // listener.start().await;
//! }
//! ```

pub mod config;
pub mod ethereum;
pub mod listener;
pub mod matcher;
pub mod processor;
pub mod tron;

pub use config::UsdtConfig;
pub use listener::UsdtListener;
pub use processor::TransactionProcessor;

use thiserror::Error;

/// USDT 监听服务错误
#[derive(Error, Debug)]
pub enum UsdtError {
    #[error("API request failed: {0}")]
    ApiError(String),

    #[error("Transaction not found")]
    TransactionNotFound,

    #[error("Order not found")]
    OrderNotFound,

    #[error("Invalid amount format")]
    InvalidAmount,

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Config error: {0}")]
    ConfigError(String),

    #[error("Encryption error: {0}")]
    EncryptionError(String),
}

impl From<reqwest::Error> for UsdtError {
    fn from(e: reqwest::Error) -> Self {
        UsdtError::ApiError(e.to_string())
    }
}

impl From<sqlx::Error> for UsdtError {
    fn from(e: sqlx::Error) -> Self {
        UsdtError::DatabaseError(e.to_string())
    }
}
