//! 配置服务
//!
//! 提供系统配置的读取和写入。
//! 除 server/database/redis 三个连接配置外，所有业务配置均从数据库读取。

use rsws_common::error::RswsError;
use rsws_db::RedisService;
use sqlx::PgPool;
use tracing::warn;

/// PayPal 配置（从 paypal_configs 表读取）
#[derive(Debug, Clone)]
pub struct PayPalDbConfig {
    pub client_id: String,
    pub client_secret: String,
    pub sandbox: bool,
    pub webhook_id: Option<String>,
    pub webhook_secret: Option<String>,
    pub base_url: String,
    pub return_url: String,
    pub cancel_url: String,
    pub brand_name: String,
    pub min_amount: rust_decimal::Decimal,
    pub max_amount: rust_decimal::Decimal,
    pub fee_rate: rust_decimal::Decimal,
}

/// 区块链配置（从 blockchain_configs 表读取）
#[derive(Debug, Clone)]
pub struct BlockchainDbConfig {
    pub network: String,
    pub network_name: String,
    pub api_url: String,
    pub api_key: Option<String>,
    pub usdt_contract: String,
    pub min_confirmations: i32,
    pub min_amount: rust_decimal::Decimal,
    pub max_amount: rust_decimal::Decimal,
    pub fee_rate: rust_decimal::Decimal,
    pub is_active: bool,
}

/// Email 配置（从 email_configs 表读取）
#[derive(Debug, Clone)]
pub struct EmailDbConfig {
    pub provider: String,
    pub host: String,
    pub port: i32,
    pub username: String,
    pub password: String,
    pub use_tls: bool,
    pub from_email: String,
    pub from_name: Option<String>,
    pub reply_to: Option<String>,
}

/// USDT 监听配置（从 usdt_listen_configs 表读取）
#[derive(Debug, Clone)]
pub struct UsdtListenDbConfig {
    pub network: String,
    pub api_url: String,
    pub api_key: Option<String>,
    pub usdt_contract: String,
    pub poll_interval_seconds: u64,
    pub min_confirmations: i32,
    pub is_active: bool,
}

// ==================== Type aliases for complex query results ====================

/// PayPal 配置查询结果行（13 列）
#[allow(clippy::type_complexity)]
type PayPalConfigRow = (
    i32,
    String,
    String,
    bool,
    Option<String>,
    Option<String>,
    String,
    String,
    String,
    String,
    rust_decimal::Decimal,
    rust_decimal::Decimal,
    rust_decimal::Decimal,
);

/// 区块链配置查询结果行（10 列）
#[allow(clippy::type_complexity)]
type BlockchainConfigRow = (
    String,
    String,
    String,
    Option<String>,
    String,
    i32,
    rust_decimal::Decimal,
    rust_decimal::Decimal,
    rust_decimal::Decimal,
    bool,
);

/// Email 配置查询结果行（9 列）
#[allow(clippy::type_complexity)]
type EmailConfigRow = (
    String,
    Option<String>,
    Option<i32>,
    Option<String>,
    Option<String>,
    bool,
    String,
    Option<String>,
    Option<String>,
);

/// USDT 监听配置查询结果行（7 列）
#[allow(clippy::type_complexity)]
type UsdtListenConfigRow = (String, String, Option<String>, String, i32, i32, bool);

/// OSS 存储配置
#[derive(Debug, Clone)]
pub struct OssStorageConfig {
    pub provider: String, // "local" | "s3" | "minio" | "aliyun-oss" | "tencent-cos"
    pub endpoint: String, // S3/MinIO endpoint 或 OSS endpoint
    pub bucket: String,   // bucket 名称
    pub access_key: String, // Access Key ID
    pub secret_key: String, // Secret Access Key
    pub region: String,   // region（可选）
    pub prefix: String,   // 存储路径前缀，默认 "resources/"
    pub custom_domain: Option<String>, // 自定义 CDN 域名（可选）
    pub is_active: bool,  // 是否启用
}

impl OssStorageConfig {
    /// 从 system_configs 构建配置（前缀 storage.）
    pub fn from_map(pairs: &[(String, String, String)]) -> Self {
        let get = |key: &str, pairs: &[(String, String, String)]| -> String {
            pairs
                .iter()
                .find(|(k, _, _)| k == key)
                .map(|(_, v, _)| v.clone())
                .unwrap_or_default()
        };
        let get_bool = |key: &str, pairs: &[(String, String, String)]| -> bool {
            pairs
                .iter()
                .find(|(k, _, _)| k == key)
                .map(|(_, v, _)| v.parse().unwrap_or(false))
                .unwrap_or(false)
        };
        Self {
            provider: get("storage.provider", pairs),
            endpoint: get("storage.endpoint", pairs),
            bucket: get("storage.bucket", pairs),
            access_key: get("storage.access_key", pairs),
            secret_key: get("storage.secret_key", pairs),
            region: get("storage.region", pairs),
            prefix: get("storage.prefix", pairs),
            custom_domain: pairs
                .iter()
                .find(|(k, _, _)| k == "storage.custom_domain")
                .map(|(_, v, _)| Some(v.clone()))
                .unwrap_or(None),
            is_active: get_bool("storage.enabled", pairs),
        }
    }

    /// 是否为本地存储
    pub fn is_local(&self) -> bool {
        self.provider == "local" || self.provider.is_empty()
    }
}

/// 配置服务
pub struct ConfigService {
    pool: PgPool,
    redis: RedisService,
}

impl ConfigService {
    /// 创建配置服务实例
    pub fn new(pool: PgPool, redis: RedisService) -> Self {
        Self { pool, redis }
    }

    /// 获取数据库连接池（供 middleware 直接查询）
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// 获取 Redis 客户端（供 rate_limit 使用）
    pub fn redis_client(&self) -> &RedisService {
        &self.redis
    }

    // ==================== system_configs 通用配置 ====================

    /// 获取配置值
    pub async fn get(&self, key: &str) -> Result<Option<String>, RswsError> {
        let result: Option<(String,)> =
            sqlx::query_as("SELECT config_value FROM system_configs WHERE config_key = $1")
                .bind(key)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| RswsError::internal(format!("Failed to get config: {}", e)))?;

        Ok(result.map(|r| r.0))
    }

    /// 设置配置
    pub async fn set(&self, key: &str, value: &str) -> Result<(), RswsError> {
        sqlx::query(
            r#"
            INSERT INTO system_configs (config_key, config_value, config_type, is_encrypted, created_at, updated_at)
            VALUES ($1, $2, 'string', false, NOW(), NOW())
            ON CONFLICT (config_key) DO UPDATE SET config_value = $2, updated_at = NOW()
            "#,
        )
        .bind(key)
        .bind(value)
        .execute(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to set config: {}", e)))?;

        Ok(())
    }

    /// 获取整型配置值
    pub async fn get_int(&self, key: &str) -> Result<Option<i64>, RswsError> {
        match self.get(key).await? {
            Some(v) => Ok(Some(v.parse().map_err(|_| {
                RswsError::internal(format!("Config '{}' is not a valid integer", key))
            })?)),
            None => Ok(None),
        }
    }

    /// 获取布尔配置值
    pub async fn get_bool(&self, key: &str) -> Result<Option<bool>, RswsError> {
        match self.get(key).await? {
            Some(v) => Ok(Some(v.parse().unwrap_or(false))),
            None => Ok(None),
        }
    }

    // ==================== PayPal 配置 ====================

    /// 从 paypal_configs 表获取活跃的 PayPal 配置
    pub async fn get_paypal_config(&self) -> Result<Option<PayPalDbConfig>, RswsError> {
        let row: Option<PayPalConfigRow> = sqlx::query_as(
            r#"
                SELECT id, client_id, client_secret_encrypted, sandbox,
                       webhook_id, webhook_secret_encrypted,
                       base_url, return_url, cancel_url, brand_name,
                       min_amount, max_amount, fee_rate
                FROM paypal_configs
                WHERE is_active = true
                ORDER BY id DESC
                LIMIT 1
                "#,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to get PayPal config: {}", e)))?;

        Ok(row.map(
            |(
                _,
                client_id,
                client_secret,
                sandbox,
                webhook_id,
                webhook_secret,
                base_url,
                return_url,
                cancel_url,
                brand_name,
                min_amount,
                max_amount,
                fee_rate,
            )| {
                PayPalDbConfig {
                    client_id,
                    client_secret,
                    sandbox,
                    webhook_id,
                    webhook_secret,
                    base_url,
                    return_url,
                    cancel_url,
                    brand_name,
                    min_amount,
                    max_amount,
                    fee_rate,
                }
            },
        ))
    }

    // ==================== 区块链配置 ====================

    /// 从 blockchain_configs 表获取所有活跃的区块链配置
    pub async fn get_blockchain_configs(&self) -> Result<Vec<BlockchainDbConfig>, RswsError> {
        let rows: Vec<BlockchainConfigRow> = sqlx::query_as(
            r#"
                SELECT network, network_name, api_url, api_key_encrypted,
                       usdt_contract, min_confirmations,
                       min_amount, max_amount, fee_rate, is_active
                FROM blockchain_configs
                WHERE is_active = true
                ORDER BY network
                "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to get blockchain configs: {}", e)))?;

        Ok(rows
            .into_iter()
            .map(
                |(
                    network,
                    network_name,
                    api_url,
                    api_key,
                    usdt_contract,
                    min_confirmations,
                    min_amount,
                    max_amount,
                    fee_rate,
                    is_active,
                )| {
                    BlockchainDbConfig {
                        network,
                        network_name,
                        api_url,
                        api_key,
                        usdt_contract,
                        min_confirmations,
                        min_amount,
                        max_amount,
                        fee_rate,
                        is_active,
                    }
                },
            )
            .collect())
    }

    /// 获取指定网络的区块链配置
    pub async fn get_blockchain_config(
        &self,
        network: &str,
    ) -> Result<Option<BlockchainDbConfig>, RswsError> {
        Ok(self
            .get_blockchain_configs()
            .await?
            .into_iter()
            .find(|c| c.network == network))
    }

    // ==================== Email 配置 ====================

    /// 从 email_configs 表获取活跃的邮件配置
    pub async fn get_email_config(&self) -> Result<Option<EmailDbConfig>, RswsError> {
        let row: Option<EmailConfigRow> = sqlx::query_as(
            r#"
                SELECT provider, host, port, username, password_encrypted,
                       use_tls, from_email, from_name, reply_to
                FROM email_configs
                WHERE is_active = true
                ORDER BY id DESC
                LIMIT 1
                "#,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to get email config: {}", e)))?;

        match row {
            Some((
                provider,
                host,
                port,
                username,
                password,
                use_tls,
                from_email,
                from_name,
                reply_to,
            )) => Ok(Some(EmailDbConfig {
                provider,
                host: host.unwrap_or_default(),
                port: port.unwrap_or(465),
                username: username.unwrap_or_default(),
                password: password.unwrap_or_default(),
                use_tls,
                from_email,
                from_name,
                reply_to,
            })),
            None => {
                warn!("No active email config found in database");
                Ok(None)
            }
        }
    }

    // ==================== USDT 监听配置 ====================

    /// 从 usdt_listen_configs 表获取所有活跃的监听配置
    pub async fn get_usdt_listen_configs(&self) -> Result<Vec<UsdtListenDbConfig>, RswsError> {
        let rows: Vec<UsdtListenConfigRow> = sqlx::query_as(
            r#"
                SELECT network, api_url, api_key_encrypted,
                       usdt_contract, poll_interval_seconds,
                       min_confirmations, is_active
                FROM usdt_listen_configs
                WHERE is_active = true
                ORDER BY network
                "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to get USDT listen configs: {}", e)))?;

        Ok(rows
            .into_iter()
            .map(
                |(
                    network,
                    api_url,
                    api_key,
                    usdt_contract,
                    poll_interval_seconds,
                    min_confirmations,
                    is_active,
                )| {
                    UsdtListenDbConfig {
                        network,
                        api_url,
                        api_key,
                        usdt_contract,
                        poll_interval_seconds: poll_interval_seconds as u64,
                        min_confirmations,
                        is_active,
                    }
                },
            )
            .collect())
    }

    /// 获取指定网络的 USDT 监听配置
    pub async fn get_usdt_listen_config(
        &self,
        network: &str,
    ) -> Result<Option<UsdtListenDbConfig>, RswsError> {
        Ok(self
            .get_usdt_listen_configs()
            .await?
            .into_iter()
            .find(|c| c.network == network))
    }

    // ==================== 加密配置 ====================

    /// 从 encryption_configs 表获取活跃的加密密钥
    pub async fn get_encryption_key(&self) -> Result<Option<String>, RswsError> {
        let row: Option<(String,)> = sqlx::query_as(
            r#"
            SELECT encryption_key_encrypted
            FROM encryption_configs
            WHERE is_active = true
            ORDER BY key_version DESC
            LIMIT 1
            "#,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to get encryption key: {}", e)))?;

        Ok(row.map(|r| r.0))
    }

    // ==================== 批量获取配置 ====================

    /// 获取所有 system_configs（按命名空间前缀过滤）
    pub async fn get_configs_by_prefix(
        &self,
        prefix: &str,
    ) -> Result<Vec<(String, String, String)>, RswsError> {
        let rows: Vec<(String, String, String)> = sqlx::query_as(
            r#"
            SELECT config_key, config_value, config_type
            FROM system_configs
            WHERE config_key LIKE $1
            ORDER BY config_key
            "#,
        )
        .bind(format!("{}%", prefix))
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to get configs by prefix: {}", e)))?;

        Ok(rows)
    }

    // ==================== OSS 存储配置 ====================

    /// 获取 OSS 存储配置
    pub async fn get_storage_config(&self) -> Result<OssStorageConfig, RswsError> {
        let pairs = self.get_configs_by_prefix("storage").await?;
        Ok(OssStorageConfig::from_map(&pairs))
    }

    /// 批量保存 OSS 存储配置
    pub async fn save_storage_config(&self, config: &OssStorageConfig) -> Result<(), RswsError> {
        let mut pairs: Vec<(&str, String)> = vec![
            ("storage.provider", config.provider.clone()),
            ("storage.enabled", config.is_active.to_string()),
            ("storage.endpoint", config.endpoint.clone()),
            ("storage.bucket", config.bucket.clone()),
            ("storage.access_key", config.access_key.clone()),
            ("storage.secret_key", config.secret_key.clone()),
            ("storage.region", config.region.clone()),
            ("storage.prefix", {
                if config.prefix.is_empty() {
                    "resources/".to_string()
                } else {
                    config.prefix.clone()
                }
            }),
        ];
        if let Some(ref domain) = config.custom_domain {
            pairs.push(("storage.custom_domain", domain.clone()));
        }
        for (key, value) in pairs {
            self.set(key, &value).await?;
        }
        Ok(())
    }
}
