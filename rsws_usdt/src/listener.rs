//! USDT 交易监听服务

use crate::{
    config::{UsdtConfig, WalletAddress, ListenerStatus},
    processor::TransactionProcessor,
    tron::TronClient,
    ethereum::EthereumClient,
    UsdtError,
};
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
    /// 创建新监听服务
    pub fn new(
        db_pool: PgPool,
        tron_config: Option<UsdtConfig>,
        ethereum_config: Option<UsdtConfig>,
    ) -> Self {
        let tron_client = tron_config.as_ref().map(|c| TronClient::new(c));
        let ethereum_client = ethereum_config.as_ref().map(|c| EthereumClient::new(c));

        // 使用精确匹配策略
        let processor = TransactionProcessor::new(
            db_pool.clone(),
            crate::matcher::MatchStrategy::Exact,
        );

        Self {
            db_pool,
            tron_client,
            ethereum_client,
            processor: Arc::new(processor),
        }
    }

    /// 启动监听服务
    ///
    /// 生成两个后台任务，分别监听 Tron 和 Ethereum 网络
    pub async fn start(&self) {
        info!("Starting USDT listener service");

        // 启动 Tron 监听任务
        if let Some(ref client) = self.tron_client {
            let db_pool = self.db_pool.clone();
            let processor = self.processor.clone();
            let client = client.clone();

            tokio::spawn(async move {
                Self::listen_tron(db_pool, client, processor).await;
            });

            info!("Tron listener started");
        }

        // 启动 Ethereum 监听任务
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

            // 获取收款地址列表
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

            // 获取最新区块高度
            let latest_block = match client.get_latest_block_number().await {
                Ok(b) => b,
                Err(e) => {
                    error!("Failed to get latest Tron block: {}", e);
                    continue;
                }
            };

            // 检查每个地址的交易
            for wallet in wallets {
                match client.get_transactions(&wallet.address, 20).await {
                    Ok(transactions) => {
                        for tx in transactions {
                            // 计算确认数
                            let confirmations = client.calculate_confirmations(
                                tx.block_number,
                                latest_block,
                            );

                            // 检查确认数是否足够
                            if !client.is_confirmed(confirmations) {
                                continue;
                            }

                            // 处理交易
                            match processor.process_transaction(
                                &tx.tx_id,
                                "tron",
                                &tx.from,
                                &tx.to,
                                tx.amount,
                                tx.block_number as i64,
                                confirmations as i32,
                            ).await {
                                Ok(result) => {
                                    info!("Tron transaction processed: {:?}", result);
                                }
                                Err(e) => {
                                    error!("Failed to process Tron transaction: {}", e);
                                }
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

            // 获取收款地址列表
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

            // 检查每个地址的交易
            for wallet in wallets {
                match client.get_transactions(&wallet.address, 20).await {
                    Ok(transactions) => {
                        for tx in transactions {
                            // 检查确认数是否足够
                            if !client.is_confirmed(tx.confirmations) {
                                continue;
                            }

                            // 处理交易
                            match processor.process_transaction(
                                &tx.tx_hash,
                                "ethereum",
                                &tx.from,
                                &tx.to,
                                tx.amount,
                                tx.block_number as i64,
                                tx.confirmations as i32,
                            ).await {
                                Ok(result) => {
                                    info!("Ethereum transaction processed: {:?}", result);
                                }
                                Err(e) => {
                                    error!("Failed to process Ethereum transaction: {}", e);
                                }
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

    /// 获取活跃的收款地址
    async fn get_active_wallets(
        db_pool: &PgPool,
        network: &str,
    ) -> Result<Vec<WalletAddress>, UsdtError> {
        let rows = sqlx::query_as::<_, (String, String, Option<String>, bool, rust_decimal::Decimal)>(
            "SELECT address, network, name, is_active, total_received FROM usdt_wallets WHERE network = $1 AND is_active = true"
        )
        .bind(network)
        .fetch_all(db_pool)
        .await?;

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

    /// 获取监听状态
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

// 为 TronClient 和 EthereumClient 实现 Clone
impl Clone for TronClient {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            api_url: self.api_url.clone(),
            api_key: self.api_key.clone(),
            usdt_contract: self.usdt_contract.clone(),
            min_confirmations: self.min_confirmations,
        }
    }
}

impl Clone for EthereumClient {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            api_url: self.api_url.clone(),
            api_key: self.api_key.clone(),
            usdt_contract: self.usdt_contract.clone(),
            min_confirmations: self.min_confirmations,
        }
    }
}
