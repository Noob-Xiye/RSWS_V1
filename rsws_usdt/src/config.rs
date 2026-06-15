//! USDT 监听配置

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// USDT 监听配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsdtConfig {
    /// 网络类型: "tron" 或 "ethereum"
    pub network: String,

    /// API URL (TronGrid 或 Etherscan)
    pub api_url: String,

    /// API Key (可选，用于提高速率限制)
    pub api_key: Option<String>,

    /// USDT 合约地址
    pub usdt_contract: String,

    /// 轮询间隔 (秒)
    pub poll_interval_seconds: u64,

    /// 最小确认数
    pub min_confirmations: i32,

    /// 是否启用
    pub is_active: bool,
}

impl UsdtConfig {
    /// 创建 Tron 网络默认配置
    pub fn tron_default() -> Self {
        Self {
            network: "tron".to_string(),
            api_url: "https://api.trongrid.io".to_string(),
            api_key: None,
            usdt_contract: "TR7NHmqjeNQHG7uHypHpP6QqQqQqQqQqQq".to_string(), // USDT TRC20 合约
            poll_interval_seconds: 10,
            min_confirmations: 3,
            is_active: true,
        }
    }

    /// 创建 Ethereum 网络默认配置
    pub fn ethereum_default() -> Self {
        Self {
            network: "ethereum".to_string(),
            api_url: "https://api.etherscan.io".to_string(),
            api_key: None,
            usdt_contract: "0xdAC17F958D2ee523a2206206994597C13D831ec7".to_string(), // USDT ERC20 合约
            poll_interval_seconds: 15,
            min_confirmations: 12,
            is_active: true,
        }
    }
}

/// 收款地址配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletAddress {
    /// 地址
    pub address: String,

    /// 网络类型
    pub network: String,

    /// 名称 (备注)
    pub name: Option<String>,

    /// 是否启用
    pub is_active: bool,

    /// 累计收款金额
    pub total_received: Decimal,
}

/// 监听器状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListenerStatus {
    /// 网络类型
    pub network: String,

    /// 是否运行中
    pub is_running: bool,

    /// 最后检查时间
    pub last_check_at: Option<chrono::DateTime<chrono::Utc>>,

    /// 最后处理的区块号
    pub last_block_number: Option<u64>,

    /// 已处理交易数
    pub processed_transactions: u64,

    /// 错误数
    pub error_count: u64,
}
