//! 钱包仓储层
//!
//! 提供 USDT 收款钱包的数据库操作

use chrono::{DateTime, Utc};
use rsws_common::error::RswsError;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::PgPool;

/// USDT 钱包
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UsdtWallet {
    pub id: i64,
    pub address: String,
    pub network: String,
    pub name: Option<String>,
    pub is_active: bool,
    pub total_received: Option<f64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// USDT 钱包仓储
#[derive(Clone)]
pub struct WalletRepository {
    pool: PgPool,
}

impl WalletRepository {
    /// 创建钱包仓储实例
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 根据网络获取平台收款钱包（返回第一个激活的）
    pub async fn get_platform_wallet(&self, network: &str) -> Result<Option<UsdtWallet>, RswsError> {
        let wallet = sqlx::query_as::<_, UsdtWallet>(
            r#"
            SELECT id, address, network, name, is_active, total_received, created_at, updated_at 
            FROM usdt_wallets 
            WHERE network = $1 AND is_active = true 
            ORDER BY created_at ASC 
            LIMIT 1
            "#,
        )
        .bind(network)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to get wallet: {}", e)))?;

        Ok(wallet)
    }

    /// 根据地址查询钱包
    pub async fn get_by_address(&self, address: &str) -> Result<Option<UsdtWallet>, RswsError> {
        let wallet = sqlx::query_as::<_, UsdtWallet>(
            r#"
            SELECT id, address, network, name, is_active, total_received, created_at, updated_at 
            FROM usdt_wallets 
            WHERE address = $1
            "#,
        )
        .bind(address)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to get wallet by address: {}", e)))?;

        Ok(wallet)
    }

    /// 列出所有钱包
    pub async fn list_all(&self) -> Result<Vec<UsdtWallet>, RswsError> {
        let wallets = sqlx::query_as::<_, UsdtWallet>(
            r#"SELECT id, address, network, name, is_active, total_received, created_at, updated_at FROM usdt_wallets ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to list wallets: {}", e)))?;

        Ok(wallets)
    }

    /// 创建或更新钱包
    pub async fn upsert(&self, network: &str, address: &str, name: Option<&str>) -> Result<UsdtWallet, RswsError> {
        let wallet = sqlx::query_as::<_, UsdtWallet>(
            r#"
            INSERT INTO usdt_wallets (address, network, name, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, true, NOW(), NOW())
            ON CONFLICT (id) DO UPDATE SET
                address = EXCLUDED.address,
                name = EXCLUDED.name,
                is_active = true,
                updated_at = NOW()
            RETURNING id, address, network, name, is_active, total_received, created_at, updated_at
            "#,
        )
        .bind(address)
        .bind(network)
        .bind(name)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to upsert wallet: {}", e)))?;

        Ok(wallet)
    }
}
