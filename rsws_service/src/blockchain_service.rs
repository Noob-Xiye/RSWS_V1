//! 区块链服务
//!
//! 所有配置均从数据库读取（blockchain_configs + usdt_listen_configs 表）

use crate::config_service::BlockchainDbConfig;
use reqwest::Client;
use rsws_common::error::RswsError;
use rsws_db::WalletRepository;
use serde_json::Value;
use std::sync::Arc;
use tracing::{info, warn};

/// 区块链服务
pub struct BlockchainService {
    client: Client,
    wallet_repo: Arc<WalletRepository>,
}

impl BlockchainService {
    /// 创建区块链服务实例
    pub fn new(wallet_repo: WalletRepository) -> Self {
        Self {
            client: Client::new(),
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
                warn!("No TRC20 wallet found in DB");
                String::new()
            }
            Err(e) => {
                warn!("Failed to get TRC20 wallet from DB: {}", e);
                String::new()
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
                warn!("No ERC20 wallet found in DB");
                String::new()
            }
            Err(e) => {
                warn!("Failed to get ERC20 wallet from DB: {}", e);
                String::new()
            }
        }
    }

    /// 使用传入的区块链配置检查 TRON 交易状态
    pub async fn check_tron_transaction_with_config(
        &self,
        tx_hash: &str,
        config: &BlockchainDbConfig,
    ) -> Result<Value, RswsError> {
        info!("Checking TRON transaction: {}", tx_hash);

        let url = format!(
            "{}/v1/transactions/{}",
            config.api_url.trim_end_matches('/'),
            tx_hash
        );

        let mut request = self.client.get(&url);
        if let Some(api_key) = &config.api_key {
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
                        let confirmed = json
                            .get("data")
                            .and_then(|d| d.get(0))
                            .and_then(|tx| tx.get("confirmed"))
                            .and_then(|c| c.as_bool())
                            .unwrap_or(false);
                        let confirmations = if confirmed {
                            config.min_confirmations
                        } else {
                            0
                        };
                        Ok(serde_json::json!({
                            "hash": tx_hash,
                            "network": "tron",
                            "status": if confirmed { "confirmed" } else { "pending" },
                            "confirmed": confirmed,
                            "confirmations": confirmations,
                            "required_confirmations": config.min_confirmations,
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
                Err(RswsError::internal(format!(
                    "TronGrid request failed: {}",
                    e
                )))
            }
        }
    }

    /// 使用传入的区块链配置检查 ETH/ERC20 交易状态
    pub async fn check_eth_transaction_with_config(
        &self,
        tx_hash: &str,
        config: &BlockchainDbConfig,
    ) -> Result<Value, RswsError> {
        info!("Checking ETH transaction: {}", tx_hash);

        let api_key = config.api_key.as_deref().unwrap_or("");

        let url = format!(
            "{}/api?module=transaction&action=gettxreceiptstatus&txhash={}&apikey={}",
            config.api_url.trim_end_matches('/'),
            tx_hash,
            api_key
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
                        let status = json
                            .get("result")
                            .and_then(|r| r.get("status"))
                            .and_then(|s| s.as_str())
                            .unwrap_or("0");
                        let is_success = status == "1";
                        let confirmations = if is_success {
                            config.min_confirmations
                        } else {
                            0
                        };
                        Ok(serde_json::json!({
                            "hash": tx_hash,
                            "network": "ethereum",
                            "status": if is_success { "confirmed" } else { "pending" },
                            "confirmed": is_success,
                            "confirmations": confirmations,
                            "required_confirmations": config.min_confirmations,
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
                Err(RswsError::internal(format!(
                    "Etherscan request failed: {}",
                    e
                )))
            }
        }
    }

    /// 验证 TRC20 地址格式
    pub fn validate_trc20_address(&self, address: &str) -> bool {
        address.starts_with('T') && address.len() == 34
    }

    /// 验证 ERC20 地址格式
    pub fn validate_erc20_address(&self, address: &str) -> bool {
        address.starts_with("0x") && address.len() == 42
    }

    /// 列出所有 USDT 钱包
    pub async fn list_usdt_wallets(&self) -> Result<Vec<rsws_db::wallet::UsdtWallet>, RswsError> {
        self.wallet_repo.list_all().await
    }

    /// 更新或创建 USDT 钱包
    pub async fn upsert_usdt_wallet(
        &self,
        network: &str,
        address: &str,
        name: Option<&str>,
    ) -> Result<rsws_db::wallet::UsdtWallet, RswsError> {
        self.wallet_repo.upsert(network, address, name).await
    }
}
