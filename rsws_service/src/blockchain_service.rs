use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use rsws_common::error::ServiceError;
use rsws_model::payment::*;
use rust_decimal::Decimal;
use std::collections::HashMap;
use qrcode::{QrCode, render::svg};
use base64;

pub struct BlockchainService {
    client: Client,
    network: String, // "tron" or "ethereum"
    api_key: String,
    api_url: String,
    usdt_contract: String,
    wallet_addresses: Vec<String>,
    min_confirmations: i32,
    current_wallet_index: std::sync::atomic::AtomicUsize,
}

#[derive(Serialize, Deserialize)]
struct TronTransaction {
    txid: String,
    block_number: u64,
    from: String,
    to: String,
    value: String,
    confirmations: u32,
}

#[derive(Serialize, Deserialize)]
struct TronApiResponse {
    success: bool,
    data: Vec<TronTransaction>,
}

impl BlockchainService {
    pub fn new(
        network: String,
        api_key: String,
        api_url: String,
        usdt_contract: String,
        wallet_addresses: Vec<String>,
        min_confirmations: i32,
    ) -> Self {
        Self {
            client: Client::new(),
            network,
            api_key,
            api_url,
            usdt_contract,
            wallet_addresses,
            min_confirmations,
            current_wallet_index: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    fn get_next_wallet(&self) -> String {
        let index = self.current_wallet_index
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst) % self.wallet_addresses.len();
        self.wallet_addresses[index].clone()
    }

    async fn check_transaction(&self, address: &str, amount: &Decimal) -> Result<Option<BlockchainTransaction>, ServiceError> {
        match self.network.as_str() {
            "tron" => self.check_tron_transaction(address, amount).await,
            "ethereum" => self.check_ethereum_transaction(address, amount).await,
            _ => Err(ServiceError::BadRequest("Unsupported network".to_string())),
        }
    }

    async fn check_tron_transaction(&self, address: &str, amount: &Decimal) -> Result<Option<BlockchainTransaction>, ServiceError> {
        // TRON API调用逻辑
        let url = format!(
            "{}/v1/accounts/{}/transactions/trc20?contract_address={}&limit=20",
            self.api_url, address, self.usdt_contract
        );

        let response = self.client
            .get(&url)
            .header("TRON-PRO-API-KEY", &self.api_key)
            .send()
            .await?
            .json::<TronApiResponse>()
            .await?;

        if response.success {
            for tx in response.data {
                let tx_amount = Decimal::from_str_exact(&tx.value)
                    .map_err(|_| ServiceError::BadRequest("Invalid amount format".to_string()))?;
                
                if tx.to.eq_ignore_ascii_case(address) && 
                   tx_amount == *amount && 
                   tx.confirmations >= self.min_confirmations as u32 {
                    return Ok(Some(BlockchainTransaction {
                        txid: tx.txid,
                        block_number: tx.block_number,
                        from: tx.from,
                        to: tx.to,
                        value: tx.value,
                        confirmations: tx.confirmations,
                        network: "tron".to_string(),
                    }));
                }
            }
        }
        Ok(None)
    }

    async fn check_ethereum_transaction(&self, address: &str, amount: &Decimal) -> Result<Option<BlockchainTransaction>, ServiceError> {
        // Ethereum API调用逻辑
        let url = format!(
            "{}?module=account&action=tokentx&contractaddress={}&address={}&page=1&offset=20&sort=desc&apikey={}",
            self.api_url, self.usdt_contract, address, self.api_key
        );

        let response = self.client
            .get(&url)
            .send()
            .await?
            .json::<EthereumApiResponse>()
            .await?;

        if response.status == "1" {
            for tx in response.result {
                // USDT在Ethereum上有6位小数
                let tx_amount = Decimal::from_str_exact(&tx.value)
                    .map_err(|_| ServiceError::BadRequest("Invalid amount format".to_string()))?
                    / Decimal::new(1000000, 0); // 转换为USDT单位
                
                if tx.to.eq_ignore_ascii_case(address) && 
                   tx_amount == *amount && 
                   tx.confirmations.parse::<i32>().unwrap_or(0) >= self.min_confirmations {
                    return Ok(Some(BlockchainTransaction {
                        txid: tx.hash,
                        block_number: tx.block_number.parse().unwrap_or(0),
                        from: tx.from,
                        to: tx.to,
                        value: tx.value,
                        confirmations: tx.confirmations.parse().unwrap_or(0) as u32,
                        network: "ethereum".to_string(),
                    }));
                }
            }
        }
        Ok(None)
    }

    fn generate_qr_code(&self, address: &str, amount: &Decimal) -> Result<String, ServiceError> {
        let uri = match self.network.as_str() {
            "tron" => format!("tron:{}?amount={}", address, amount),
            "ethereum" => format!("ethereum:{}@1?value={}", address, amount),
            _ => return Err(ServiceError::BadRequest("Unsupported network".to_string())),
        };

        let code = QrCode::new(&uri)
            .map_err(|_| ServiceError::BadRequest("Failed to generate QR code".to_string()))?;
        
        let svg = code.render::<svg::Color>()
            .min_dimensions(200, 200)
            .build();
        
        let encoded = base64::encode(svg.as_bytes());
        Ok(format!("data:image/svg+xml;base64,{}", encoded))
    }
}

// Ethereum API响应结构
#[derive(Serialize, Deserialize)]
struct EthereumApiResponse {
    status: String,
    message: String,
    result: Vec<EthereumTransaction>,
}

#[derive(Serialize, Deserialize)]
struct EthereumTransaction {
    hash: String,
    block_number: String,
    from: String,
    to: String,
    value: String,
    confirmations: String,
}

// 通用区块链交易结构
#[derive(Serialize, Deserialize)]
struct BlockchainTransaction {
    txid: String,
    block_number: u64,
    from: String,
    to: String,
    value: String,
    confirmations: u32,
    network: String,
}