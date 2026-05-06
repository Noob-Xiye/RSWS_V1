//! 区块链服务

use rsws_common::error::RswsError;
use rsws_common::config::USDTConfig;
use rsws_db::WalletRepository;
use serde_json::Value;
use std::sync::Arc;
use tracing::{info, warn};
use reqwest::Client;

/// 区块链服务
pub struct BlockchainService {
    config: USDTConfig,
    client: Client,
    wallet_repo: Arc<WalletRepository>,
}

impl BlockchainService {
    /// 创建区块链服务实例
    pub fn new(config: USDTConfig, wallet_repo: WalletRepository) -> Self {
        Self {
            client: Client::new(),
            config,
            wallet_repo: Arc::new(wallet_repo),
        }
    }

    /// 获取 TRC20 地址（优先从数据库读取，回退到配置文件）
    pub async fn get_trc20_address(&self) -> String {
        match self.wallet_repo.get_platform_wallet("tron").await {
            Ok(Some(wallet)) => {
                info!("Using TRC20 address from DB: {}", &wallet.address);
                wallet.address
            }
            Ok(None) => {
                warn!("No TRC20 wallet found in DB, using config fallback");
                self.config.trc20_address.clone()
            }
            Err(e) => {
                warn!("Failed to get TRC20 wallet from DB: {}, using config fallback", e);
                self.config.trc20_address.clone()
            }
        }
    }

    /// 获取 ERC20 地址（优先从数据库读取，回退到配置文件）
    pub async fn get_erc20_address(&self) -> String {
        match self.wallet_repo.get_platform_wallet("ethereum").await {
            Ok(Some(wallet)) => {
                info!("Using ERC20 address from DB: {}", &wallet.address);
                wallet.address
            }
            Ok(None) => {
                warn!("No ERC20 wallet found in DB, using config fallback");
                self.config.erc20_address.clone()
            }
            Err(e) => {
                warn!("Failed to get ERC20 wallet from DB: {}, using config fallback", e);
                self.config.erc20_address.clone()
            }
        }
    }

    /// 获取确认数要求
    pub fn get_confirmations_required(&self) -> u32 {
        self.config.confirmations_required
    }

    /// 检查 TRON 交易状态（TronGrid API）
    pub async fn check_tron_transaction(&self, tx_hash: &str) -> Result<Value, RswsError> {
        info!("Checking TRON transaction: {}", tx_hash);

        let url = format!(
            "https://api.trongrid.io/v1/transactions/{}",
            tx_hash
        );

        let mut request = self.client.get(&url);
        if let Some(api_key) = &self.config.trongrid_api_key {
            request = request.header("TRON-PRO-API-KEY", api_key);
        }

        match request.send().await {
            Ok(resp) => {
                if !resp.status().is_success() {
                    warn!("TronGrid API error: status={}", resp.status());
                    return Ok(serde_json::json!({
                        "hash": tx_hash,
                        "network": "tron",
                        "status": "error",
                        "error": format!("API status: {}", resp.status())
                    }));
                }
                match resp.json::<Value>().await {
                    Ok(json) => {
                        let confirmed = json.get("data")
                            .and_then(|d| d.get(0))
                            .and_then(|tx| tx.get("confirmed"))
                            .and_then(|c| c.as_bool())
                            .unwrap_or(false);
                        let confirmations = if confirmed { self.config.confirmations_required } else { 0 };
                        Ok(serde_json::json!({
                            "hash": tx_hash,
                            "network": "tron",
                            "status": if confirmed { "confirmed" } else { "pending" },
                            "confirmed": confirmed,
                            "confirmations": confirmations,
                            "required_confirmations": self.config.confirmations_required,
                            "raw": json
                        }))
                    }
                    Err(e) => {
                        warn!("Failed to parse TronGrid response: {}", e);
                        Err(RswsError::internal(format!("TronGrid parse error: {}", e)))
                    }
                }
            }
            Err(e) => {
                warn!("TronGrid request failed: {}", e);
                Err(RswsError::internal(format!("TronGrid request failed: {}", e)))
            }
        }
    }

    /// 检查 ETH/ERC20 交易状态（Etherscan API）
    pub async fn check_eth_transaction(&self, tx_hash: &str) -> Result<Value, RswsError> {
        info!("Checking ETH transaction: {}", tx_hash);

        let api_key = self.config.etherscan_api_key
            .as_deref()
            .unwrap_or("");

        let url = format!(
            "https://api.etherscan.io/api?module=transaction&action=gettxreceiptstatus&txhash={}&apikey={}",
            tx_hash, api_key
        );

        match self.client.get(&url).send().await {
            Ok(resp) => {
                if !resp.status().is_success() {
                    warn!("Etherscan API error: status={}", resp.status());
                    return Ok(serde_json::json!({
                        "hash": tx_hash,
                        "network": "ethereum",
                        "status": "error",
                        "error": format!("API status: {}", resp.status())
                    }));
                }
                match resp.json::<Value>().await {
                    Ok(json) => {
                        let status = json.get("result")
                            .and_then(|r| r.get("status"))
                            .and_then(|s| s.as_str())
                            .unwrap_or("0");
                        let is_success = status == "1";
                        let confirmations = if is_success { self.config.confirmations_required } else { 0 };
                        Ok(serde_json::json!({
                            "hash": tx_hash,
                            "network": "ethereum",
                            "status": if is_success { "confirmed" } else { "pending" },
                            "confirmed": is_success,
                            "confirmations": confirmations,
                            "required_confirmations": self.config.confirmations_required,
                            "raw": json
                        }))
                    }
                    Err(e) => {
                        warn!("Failed to parse Etherscan response: {}", e);
                        Err(RswsError::internal(format!("Etherscan parse error: {}", e)))
                    }
                }
            }
            Err(e) => {
                warn!("Etherscan request failed: {}", e);
                Err(RswsError::internal(format!("Etherscan request failed: {}", e)))
            }
        }
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