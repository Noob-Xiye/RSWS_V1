//! TronGrid API 封装

use crate::{UsdtConfig, UsdtError};
use reqwest::Client;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// TronGrid API 客户端
#[derive(Clone)]
pub struct TronClient {
    pub client: Client,
    pub api_url: String,
    pub api_key: Option<String>,
    pub usdt_contract: String,
    pub min_confirmations: i32,
}

/// Tron 交易记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TronTransaction {
    /// 交易 ID
    pub tx_id: String,

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

/// TronGrid API 响应
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TronGridResponse {
    success: bool,
    #[serde(default)]
    meta: Option<serde_json::Value>,
    #[serde(default)]
    data: Vec<TronTxData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TronTxData {
    #[serde(rename = "transaction_id")]
    tx_id: String,
    block_timestamp: i64,
    #[serde(default)]
    block_number: Option<u64>,
    from: String,
    to: String,
    value: String,
    #[serde(default)]
    contract_ret: Option<String>,
}

impl TronClient {
    /// 创建新客户端
    pub fn new(config: &UsdtConfig) -> Self {
        Self {
            client: Client::new(),
            api_url: config.api_url.clone(),
            api_key: config.api_key.clone(),
            usdt_contract: config.usdt_contract.clone(),
            min_confirmations: config.min_confirmations,
        }
    }

    /// 获取地址的 USDT 交易列表
    ///
    /// 查询指定地址接收的 USDT (TRC20) 转账记录
    pub async fn get_transactions(
        &self,
        address: &str,
        limit: u32,
    ) -> Result<Vec<TronTransaction>, UsdtError> {
        let url = format!(
            "{}/v1/accounts/{}/transactions/trc20?contract_address={}&limit={}&order_by=block_timestamp,desc",
            self.api_url, address, self.usdt_contract, limit
        );

        let mut request = self.client.get(&url);

        if let Some(ref api_key) = self.api_key {
            request = request.header("TRON-PRO-API-KEY", api_key);
        }

        let response = request
            .send()
            .await?
            .json::<TronGridResponse>()
            .await?;

        if !response.success {
            return Err(UsdtError::ApiError("TronGrid API returned error".to_string()));
        }

        // 转换数据
        let transactions = response
            .data
            .into_iter()
            .filter_map(|tx| {
                // USDT TRC20 精度为 6 位小数
                let amount = Decimal::from_str(&tx.value).ok()?;
                let amount = amount / Decimal::from(1_000_000); // 转换为 USDT 单位

                Some(TronTransaction {
                    tx_id: tx.tx_id,
                    block_number: tx.block_number?,
                    from: tx.from,
                    to: tx.to,
                    amount,
                    confirmations: 0, // 需要单独查询确认数
                    timestamp: tx.block_timestamp as u64,
                })
            })
            .collect();

        Ok(transactions)
    }

    /// 获取当前区块高度
    pub async fn get_latest_block_number(&self) -> Result<u64, UsdtError> {
        let url = format!("{}/wallet/getnowblock", self.api_url);

        let mut request = self.client.post(&url);

        if let Some(ref api_key) = self.api_key {
            request = request.header("TRON-PRO-API-KEY", api_key);
        }

        #[derive(Deserialize)]
        struct BlockResponse {
            #[serde(rename = "block_header")]
            block_header: BlockHeader,
        }

        #[derive(Deserialize)]
        struct BlockHeader {
            raw_data: RawData,
        }

        #[derive(Deserialize)]
        struct RawData {
            number: u64,
        }

        let response = request
            .send()
            .await?
            .json::<BlockResponse>()
            .await?;

        Ok(response.block_header.raw_data.number)
    }

    /// 计算确认数
    pub fn calculate_confirmations(
        &self,
        tx_block: u64,
        latest_block: u64,
    ) -> u32 {
        if latest_block >= tx_block {
            (latest_block - tx_block) as u32
        } else {
            0
        }
    }

    /// 检查确认数是否足够
    pub fn is_confirmed(&self, confirmations: u32) -> bool {
        confirmations >= self.min_confirmations as u32
    }
}
