//! USDT 交易监听服务

use crate::{
    config::{UsdtConfig, WalletAddress, ListenerStatus},
    processor::{TransactionProcessor, UsdtTransaction},
    tron::TronClient,
    ethereum::EthereumClient,
    UsdtError,
};
use chrono::Utc;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::time::{interval, Duration};
use tracing::{info, error};

/// USDT 监听服务
pub struct UsdtListener {
    db_pool: PgPool,
    tron_client: Option<TronClient>,
    ethereum_client: Option<EthereumClient>,
    processor: Arc<TransactionProcessor>,
}

impl UsdtListener {
    pub fn new(
        db_pool: PgPool,
        tron_config: Option<UsdtConfig>,
        ethereum_config: Option<UsdtConfig>,
    ) -> Self {
        let tron_client = tron_config.as_ref().map(|c| TronClient::new(c));
        let ethereum_client = ethereum_config.as_ref().map(|c| EthereumClient::new(c));
        let processor = TransactionProcessor::new(db_pool.clone());

        Self {
            db_pool,
            tron_client,
            ethereum_client,
            processor: Arc::new(processor),
        }
    }

    /// 启动监听服务
    pub async fn start(&self) {
        info!("Starting USDT listener service");

        if let Some(ref client) = self.tron_client {
            let db_pool = self.db_pool.clone();
            let processor = self.processor.clone();
            let client = client.clone();
            tokio::spawn(async move {
                Self::listen_tron(db_pool, client, processor).await;
            });
            info!("Tron listener started");
        }

        if let Some(ref client) = self.ethereum_client {
            let db_pool = self.db_pool.clone();
            let processor = self.processor.clone();
            let client = client.clone();
            tokio::spawn(async move {
                Self::listen_ethereum(db_pool, client, processor).await;
            });
            info!("Ethereum listener started");
        }
    }

    /// Tron 网络监听任务
    async fn listen_tron(
        db_pool: PgPool,
        client: TronClient,
        processor: Arc<TransactionProcessor>,
    ) {
        let mut interval = interval(Duration::from_secs(10));
        let mut _last_block: Option<u64> = None;

        loop {
            interval.tick().await;

            let wallets = match Self::get_active_wallets(&db_pool, "tron").await {
                Ok(w) => w,
                Err(e) => {
                    error!("Failed to get Tron wallets: {}", e);
                    continue;
                }
            };

            if wallets.is_empty() {
                continue;
            }

            let latest_block = match client.get_latest_block_number().await {
                Ok(b) => b,
                Err(e) => {
                    error!("Failed to get latest Tron block: {}", e);
                    continue;
                }
            };

            for wallet in wallets {
                match client.get_transactions(&wallet.address, 20).await {
                    Ok(transactions) => {
                        for raw_tx in transactions {
                            let confirmations = client.calculate_confirmations(
                                raw_tx.block_number,
                                latest_block,
                            );

                            if !client.is_confirmed(confirmations) {
                                continue;
                            }

                            let tx = UsdtTransaction {
                                id: rsws_common::snowflake::next_id(),
                                tx_hash: raw_tx.tx_id,
                                network: "tron".to_string(),
                                from_address: raw_tx.from,
                                to_address: raw_tx.to,
                                amount: raw_tx.amount,
                                block_number: raw_tx.block_number as i64,
                                confirmations: confirmations as i32,
                                status: "pending".to_string(),
                                order_id: None,
                                processed_at: None,
                                created_at: Utc::now(),
                            };

                            match processor.process_transaction(tx).await {
                                Ok(result) => info!("Tron transaction processed: matched={}", result),
                                Err(e) => error!("Failed to process Tron transaction: {}", e),
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to get Tron transactions for {}: {}", wallet.address, e);
                    }
                }
            }

            _last_block = Some(latest_block);
        }
    }

    /// Ethereum 网络监听任务
    async fn listen_ethereum(
        db_pool: PgPool,
        client: EthereumClient,
        processor: Arc<TransactionProcessor>,
    ) {
        let mut interval = interval(Duration::from_secs(15));

        loop {
            interval.tick().await;

            // 从数据库读取以太坊收款地址
            let wallets = match Self::get_active_wallets(&db_pool, "ethereum").await {
                Ok(w) => w,
                Err(e) => {
                    error!("Failed to get Ethereum wallets: {}", e);
                    continue;
                }
            };

            if wallets.is_empty() {
                continue;
            }

            for wallet in wallets {
                match client.get_transactions(&wallet.address, 20).await {
                    Ok(transactions) => {
                        for raw_tx in transactions {
                            if !client.is_confirmed(raw_tx.confirmations) {
                                continue;
                            }

                            let tx = UsdtTransaction {
                                id: rsws_common::snowflake::next_id(),
                                tx_hash: raw_tx.tx_hash,
                                network: "ethereum".to_string(),
                                from_address: raw_tx.from,
                                to_address: raw_tx.to,
                                amount: raw_tx.amount,
                                block_number: raw_tx.block_number as i64,
                                confirmations: raw_tx.confirmations as i32,
                                status: "pending".to_string(),
                                order_id: None,
                                processed_at: None,
                                created_at: Utc::now(),
                            };

                            match processor.process_transaction(tx).await {
                                Ok(result) => info!("Ethereum transaction processed: matched={}", result),
                                Err(e) => error!("Failed to process Ethereum transaction: {}", e),
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to get Ethereum transactions for {}: {}", wallet.address, e);
                    }
                }
            }
        }
    }

    async fn get_active_wallets(
        db_pool: &PgPool,
        network: &str,
    ) -> Result<Vec<WalletAddress>, UsdtError> {
        let rows = sqlx::query_as::<_, (String, String, Option<String>, bool, rust_decimal::Decimal)>(
            "SELECT address, network, name, is_active, total_received FROM usdt_wallets WHERE network = $1 AND is_active = true"
        )
        .bind(network)
        .fetch_all(db_pool)
        .await
        .map_err(|e| UsdtError::DatabaseError(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|(address, network, name, is_active, total_received)| WalletAddress {
                address,
                network,
                name,
                is_active,
                total_received,
            })
            .collect())
    }

    pub async fn get_status(&self) -> Vec<ListenerStatus> {
        let mut statuses = Vec::new();

        if self.tron_client.is_some() {
            statuses.push(ListenerStatus {
                network: "tron".to_string(),
                is_running: true,
                last_check_at: None,
                last_block_number: None,
                processed_transactions: 0,
                error_count: 0,
            });
        }

        if self.ethereum_client.is_some() {
            statuses.push(ListenerStatus {
                network: "ethereum".to_string(),
                is_running: true,
                last_check_at: None,
                last_block_number: None,
                processed_transactions: 0,
                error_count: 0,
            });
        }

        statuses
    }
}
