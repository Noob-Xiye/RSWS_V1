//! Etherscan API 封装

use crate::{UsdtConfig, UsdtError};
use reqwest::Client;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Etherscan API 客户端
#[derive(Clone)]
pub struct EthereumClient {
    pub client: Client,
    pub api_url: String,
    pub api_key: Option<String>,
    pub usdt_contract: String,
    pub min_confirmations: i32,
}

/// Ethereum 交易记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthereumTransaction {
    /// 交易 Hash
    pub tx_hash: String,

    /// 区块号
    pub block_number: u64,

    /// 发送地址
    pub from: String,

    /// 接收地址
    pub to: String,

    /// 金额 (USDT，已转换)
    pub amount: Decimal,

    /// 确认数
    pub confirmations: u32,

    /// 时间戳
    pub timestamp: u64,
}

/// Etherscan API 响应
#[derive(Debug, Clone, Serialize, Deserialize)]
struct EtherscanResponse {
    status: String,
    message: String,
    #[serde(default)]
    result: Vec<EtherscanTx>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EtherscanTx {
    hash: String,
    #[serde(rename = "blockNumber")]
    block_number: String,
    #[serde(rename = "timeStamp")]
    time_stamp: String,
    from: String,
    to: String,
    value: String,
    #[serde(default)]
    confirmations: String,
    #[serde(rename = "contractAddress")]
    contract_address: Option<String>,
}

impl EthereumClient {
    /// 创建新客户端
    pub fn new(config: &UsdtConfig) -> Self {
        // 强制使用 HTTPS 防止 API Key 和交易数据在传输中被截获
        let api_url = if config.api_url.starts_with("https://") {
            config.api_url.clone()
        } else {
            tracing::warn!(
                "[Security] Etherscan API URL should use HTTPS. Got: {}. Forcing HTTPS.",
                config.api_url
            );
            config.api_url.replace("http://", "https://")
        };

        Self {
            client: Client::new(),
            api_url,
            api_key: config.api_key.clone(),
            usdt_contract: config.usdt_contract.clone(),
            min_confirmations: config.min_confirmations,
        }
    }

    /// 获取地址的 USDT 交易列表
    ///
    /// 查询指定地址接收的 USDT (ERC20) 转账记录
    pub async fn get_transactions(
        &self,
        address: &str,
        limit: u32,
    ) -> Result<Vec<EthereumTransaction>, UsdtError> {
        let api_key = self.api_key.as_deref().unwrap_or("YourApiKeyToken");

        let url = format!(
            "{}?module=account&action=tokentx&contractaddress={}&address={}&page=1&offset={}&sort=desc&apikey={}",
            self.api_url, self.usdt_contract, address, limit, api_key
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await?
            .json::<EtherscanResponse>()
            .await?;

        if response.status != "1" {
            return Err(UsdtError::ApiError(format!(
                "Etherscan API error: {}",
                response.message
            )));
        }

        // 转换数据
        let transactions = response
            .result
            .into_iter()
            .filter_map(|tx| {
                // USDT ERC20 精度为 6 位小数
                let value = Decimal::from_str(&tx.value).ok()?;
                let amount = value / Decimal::from(1_000_000); // 转换为 USDT 单位

                let block_number = u64::from_str(&tx.block_number).ok()?;
                let timestamp = u64::from_str(&tx.time_stamp).ok()?;
                let confirmations = u32::from_str(&tx.confirmations).unwrap_or(0);

                Some(EthereumTransaction {
                    tx_hash: tx.hash,
                    block_number,
                    from: tx.from,
                    to: tx.to,
                    amount,
                    confirmations,
                    timestamp,
                })
            })
            .collect();

        Ok(transactions)
    }

    /// 获取当前区块高度
    pub async fn get_latest_block_number(&self) -> Result<u64, UsdtError> {
        let api_key = self.api_key.as_deref().unwrap_or("YourApiKeyToken");

        let url = format!(
            "{}?module=proxy&action=eth_blockNumber&apikey={}",
            self.api_url, api_key
        );

        #[derive(Deserialize)]
        struct BlockResponse {
            result: String,
        }

        let response = self
            .client
            .get(&url)
            .send()
            .await?
            .json::<BlockResponse>()
            .await?;

        // 结果是十六进制字符串
        let block_number = u64::from_str_radix(response.result.trim_start_matches("0x"), 16)
            .map_err(|_| UsdtError::ApiError("Invalid block number format".to_string()))?;

        Ok(block_number)
    }

    /// 检查确认数是否足够
    pub fn is_confirmed(&self, confirmations: u32) -> bool {
        confirmations >= self.min_confirmations as u32
    }
}
