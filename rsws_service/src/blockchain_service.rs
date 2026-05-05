//! 区块链服务

use rsws_common::error::RswsError;
use rsws_common::config::USDTConfig;
use serde_json::Value;
use tracing::info;

/// 区块链服务
pub struct BlockchainService {
    config: USDTConfig,
}

impl BlockchainService {
    /// 创建区块链服务实例
    pub fn new(config: USDTConfig) -> Self {
        Self { config }
    }

    /// 获取 TRC20 地址（用于接收用户付款）
    pub fn get_trc20_address(&self) -> String {
        self.config.trc20_address.clone()
    }

    /// 获取 ERC20 地址（用于接收用户付款）
    pub fn get_erc20_address(&self) -> String {
        self.config.erc20_address.clone()
    }

    /// 获取确认数要求
    pub fn get_confirmations_required(&self) -> u32 {
        self.config.confirmations_required
    }

    /// 检查 TRON 交易状态（Mock）
    pub async fn check_tron_transaction(&self, tx_hash: &str) -> Result<Value, RswsError> {
        info!("Checking TRON transaction: {}", tx_hash);

        // TODO: 实现实际的 TRON 节点查询
        // 使用 TronGrid API 或直接连接 TRON 节点

        Ok(serde_json::json!({
            "hash": tx_hash,
            "network": "tron",
            "status": "pending",
            "confirmations": 0,
            "required_confirmations": self.config.confirmations_required
        }))
    }

    /// 检查 ETH/ERC20 交易状态（Mock）
    pub async fn check_eth_transaction(&self, tx_hash: &str) -> Result<Value, RswsError> {
        info!("Checking ETH transaction: {}", tx_hash);

        // TODO: 实现实际的 Ethereum 节点查询
        // 使用 Etherscan API 或直接连接 ETH 节点

        Ok(serde_json::json!({
            "hash": tx_hash,
            "network": "ethereum",
            "status": "pending",
            "confirmations": 0,
            "required_confirmations": self.config.confirmations_required
        }))
    }

    /// 验证 TRC20 地址格式
    pub fn validate_trc20_address(&self, address: &str) -> bool {
        // TRON 地址以 T 开头，长度 34
        address.starts_with('T') && address.len() == 34
    }

    /// 验证 ERC20 地址格式
    pub fn validate_erc20_address(&self, address: &str) -> bool {
        // ETH 地址以 0x 开头，长度 42
        address.starts_with("0x") && address.len() == 42
    }
}