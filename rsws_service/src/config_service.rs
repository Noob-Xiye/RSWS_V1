use crate::error::ServiceError;
use rsws_common::encryption::EncryptionService;
use rsws_model::config::*;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct ConfigService {
    db_pool: PgPool,
    encryption: EncryptionService,
    // 内存缓存，提高性能
    cache: Arc<RwLock<HashMap<String, ConfigValue>>>,
}

impl ConfigService {
    pub fn new(db_pool: PgPool, encryption: EncryptionService) -> Self {
        Self {
            db_pool,
            encryption,
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    // 获取系统配置
    pub async fn get_system_config(&self, key: &str) -> Result<Option<ConfigValue>, ServiceError> {
        // 先检查缓存
        {
            let cache = self.cache.read().await;
            if let Some(value) = cache.get(key) {
                return Ok(Some(value.clone()));
            }
        }

        // 从数据库获取
        let config =
            sqlx::query_as::<_, SystemConfig>("SELECT * FROM system_configs WHERE config_key = $1")
                .bind(key)
                .fetch_optional(&self.db_pool)
                .await?;

        if let Some(config) = config {
            let value = self.parse_config_value(&config).await?;

            // 更新缓存
            {
                let mut cache = self.cache.write().await;
                cache.insert(key.to_string(), value.clone());
            }

            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    // 设置系统配置
    pub async fn set_system_config(
        &self,
        key: &str,
        value: ConfigValue,
        description: Option<String>,
        user_id: Option<i32>,
    ) -> Result<(), ServiceError> {
        let (config_value, config_type, is_encrypted) = self.serialize_config_value(&value).await?;

        // 检查是否已存在
        let existing =
            sqlx::query_scalar::<_, i32>("SELECT id FROM system_configs WHERE config_key = $1")
                .bind(key)
                .fetch_optional(&self.db_pool)
                .await?;

        if let Some(id) = existing {
            // 更新现有配置
            sqlx::query(
                "UPDATE system_configs SET config_value = $1, config_type = $2, 
                 is_encrypted = $3, description = $4, updated_at = CURRENT_TIMESTAMP 
                 WHERE id = $5",
            )
            .bind(&config_value)
            .bind(&config_type)
            .bind(is_encrypted)
            .bind(&description)
            .bind(id)
            .execute(&self.db_pool)
            .await?;
        } else {
            // 插入新配置
            sqlx::query(
                "INSERT INTO system_configs (config_key, config_value, config_type, 
                 is_encrypted, description) VALUES ($1, $2, $3, $4, $5)",
            )
            .bind(key)
            .bind(&config_value)
            .bind(&config_type)
            .bind(is_encrypted)
            .bind(&description)
            .execute(&self.db_pool)
            .await?;
        }

        // 更新缓存
        {
            let mut cache = self.cache.write().await;
            cache.insert(key.to_string(), value);
        }

        Ok(())
    }

    // 获取JWT配置
    pub async fn get_jwt_config(&self) -> Result<JwtConfig, ServiceError> {
        let secret = self
            .get_system_config("jwt.secret")
            .await?
            .ok_or_else(|| ServiceError::ConfigNotFound("jwt.secret".to_string()))?;

        let expires_in = self
            .get_system_config("jwt.expires_in")
            .await?
            .unwrap_or(ConfigValue::Number(3600.0));

        let refresh_expires_in = self
            .get_system_config("jwt.refresh_expires_in")
            .await?
            .unwrap_or(ConfigValue::Number(86400.0));

        let algorithm = self
            .get_system_config("jwt.algorithm")
            .await?
            .unwrap_or(ConfigValue::String("HS256".to_string()));

        Ok(JwtConfig {
            secret: match secret {
                ConfigValue::String(s) => s,
                _ => {
                    return Err(ServiceError::InvalidConfigType(
                        "jwt.secret must be string".to_string(),
                    ))
                }
            },
            expires_in: match expires_in {
                ConfigValue::Number(n) => n as i64,
                _ => 3600,
            },
            refresh_expires_in: match refresh_expires_in {
                ConfigValue::Number(n) => n as i64,
                _ => 86400,
            },
            algorithm: match algorithm {
                ConfigValue::String(s) => s,
                _ => "HS256".to_string(),
            },
        })
    }

    // 获取活跃的邮件配置
    pub async fn get_active_email_config(&self) -> Result<Option<EmailConfig>, ServiceError> {
        let config = sqlx::query_as::<_, EmailConfig>(
            "SELECT * FROM email_configs WHERE is_active = true ORDER BY id DESC LIMIT 1",
        )
        .fetch_optional(&self.db_pool)
        .await?;

        Ok(config)
    }

    // PayPal配置管理
    pub async fn get_paypal_config(&self) -> Result<Option<PayPalConfig>, ServiceError> {
        let config = sqlx::query_as::<_, PayPalConfig>(
            "SELECT * FROM paypal_configs WHERE is_active = true ORDER BY id DESC LIMIT 1",
        )
        .fetch_optional(&self.db_pool)
        .await?;

        if let Some(mut config) = config {
            // 解密敏感信息
            config.client_secret_encrypted =
                self.encryption.decrypt(&config.client_secret_encrypted)?;
            if let Some(ref webhook_secret) = config.webhook_secret_encrypted {
                config.webhook_secret_encrypted = Some(self.encryption.decrypt(webhook_secret)?);
            }
            Ok(Some(config))
        } else {
            Ok(None)
        }
    }

    pub async fn update_paypal_config(
        &self,
        request: UpdatePayPalConfigRequest,
        admin_id: i32,
    ) -> Result<(), ServiceError> {
        // 加密敏感信息
        let client_secret_encrypted = self.encryption.encrypt(&request.client_secret)?;
        let webhook_secret_encrypted = if let Some(ref secret) = request.webhook_secret {
            Some(self.encryption.encrypt(secret)?)
        } else {
            None
        };

        // 先禁用现有配置
        sqlx::query("UPDATE paypal_configs SET is_active = false")
            .execute(&self.db_pool)
            .await?;

        // 插入新配置
        sqlx::query(
            r#"
            INSERT INTO paypal_configs (
                client_id, client_secret_encrypted, sandbox, webhook_id, webhook_secret_encrypted,
                base_url, return_url, cancel_url, brand_name, min_amount, max_amount, fee_rate, is_active
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            "#
        )
        .bind(&request.client_id)
        .bind(&client_secret_encrypted)
        .bind(request.sandbox)
        .bind(&request.webhook_id)
        .bind(&webhook_secret_encrypted)
        .bind(if request.sandbox { "https://api.sandbox.paypal.com" } else { "https://api.paypal.com" })
        .bind(&request.return_url)
        .bind(&request.cancel_url)
        .bind(&request.brand_name)
        .bind(request.min_amount)
        .bind(request.max_amount)
        .bind(request.fee_rate)
        .bind(request.is_active)
        .execute(&self.db_pool)
        .await?;

        self.clear_cache().await;
        Ok(())
    }

    // 区块链配置管理
    pub async fn get_blockchain_config(
        &self,
        network: &str,
    ) -> Result<Option<BlockchainConfig>, ServiceError> {
        let config = sqlx::query_as::<_, BlockchainConfig>(
            "SELECT * FROM blockchain_configs WHERE network = $1 AND is_active = true",
        )
        .bind(network)
        .fetch_optional(&self.db_pool)
        .await?;

        if let Some(mut config) = config {
            // 解密API密钥
            if let Some(ref api_key) = config.api_key_encrypted {
                config.api_key_encrypted = Some(self.encryption.decrypt(api_key)?);
            }
            Ok(Some(config))
        } else {
            Ok(None)
        }
    }

    pub async fn update_blockchain_config(
        &self,
        network: &str,
        request: UpdateBlockchainConfigRequest,
        admin_id: i32,
    ) -> Result<(), ServiceError> {
        // 加密API密钥
        let api_key_encrypted = if let Some(ref api_key) = request.api_key {
            Some(self.encryption.encrypt(api_key)?)
        } else {
            None
        };

        let wallet_addresses_json = serde_json::to_value(&request.wallet_addresses)?;

        sqlx::query(
            r#"
            INSERT INTO blockchain_configs (
                network, network_name, api_url, api_key_encrypted, usdt_contract,
                wallet_addresses, min_confirmations, min_amount, max_amount, fee_rate, is_active
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            ON CONFLICT (network) DO UPDATE SET
                network_name = EXCLUDED.network_name,
                api_url = EXCLUDED.api_url,
                api_key_encrypted = EXCLUDED.api_key_encrypted,
                usdt_contract = EXCLUDED.usdt_contract,
                wallet_addresses = EXCLUDED.wallet_addresses,
                min_confirmations = EXCLUDED.min_confirmations,
                min_amount = EXCLUDED.min_amount,
                max_amount = EXCLUDED.max_amount,
                fee_rate = EXCLUDED.fee_rate,
                is_active = EXCLUDED.is_active,
                updated_at = NOW()
            "#,
        )
        .bind(network)
        .bind(&request.network_name)
        .bind(&request.api_url)
        .bind(&api_key_encrypted)
        .bind(&request.usdt_contract)
        .bind(&wallet_addresses_json)
        .bind(request.min_confirmations)
        .bind(request.min_amount)
        .bind(request.max_amount)
        .bind(request.fee_rate)
        .bind(request.is_active)
        .execute(&self.db_pool)
        .await?;

        self.clear_cache().await;
        Ok(())
    }

    // 获取所有活跃的支付方式
    pub async fn get_active_payment_methods(
        &self,
    ) -> Result<Vec<PaymentMethodConfig>, ServiceError> {
        let methods = sqlx::query_as::<_, PaymentMethodConfig>(
            "SELECT * FROM payment_method_configs WHERE is_active = true ORDER BY sort_order, id",
        )
        .fetch_all(&self.db_pool)
        .await?;

        Ok(methods)
    }

    pub async fn update_payment_method(
        &self,
        method_id: &str,
        request: UpdatePaymentMethodRequest,
        admin_id: i32,
    ) -> Result<(), ServiceError> {
        sqlx::query(
            r#"
            UPDATE payment_method_configs SET
                method_name = $2,
                icon_url = $3,
                description = $4,
                sort_order = $5,
                is_active = $6,
                updated_at = NOW()
            WHERE method_id = $1
            "#,
        )
        .bind(method_id)
        .bind(&request.method_name)
        .bind(&request.icon_url)
        .bind(&request.description)
        .bind(request.sort_order)
        .bind(request.is_active)
        .execute(&self.db_pool)
        .await?;

        self.clear_cache().await;
        Ok(())
    }

    // 获取加密配置
    pub async fn get_encryption_config(
        &self,
        config_name: &str,
    ) -> Result<Option<String>, ServiceError> {
        let config = sqlx::query_scalar::<_, String>(
            "SELECT encryption_key_encrypted FROM encryption_configs WHERE config_name = $1 AND is_active = true"
        )
        .bind(config_name)
        .fetch_optional(&self.db_pool)
        .await?;

        if let Some(encrypted_key) = config {
            // 使用主密钥解密加密配置
            let decrypted_key = self.encryption.decrypt(&encrypted_key)?;
            Ok(Some(decrypted_key))
        } else {
            Ok(None)
        }
    }

    // 更新加密配置
    pub async fn update_encryption_config(
        &self,
        config_name: &str,
        new_key: &str,
        admin_id: i32,
    ) -> Result<(), ServiceError> {
        let encrypted_key = self.encryption.encrypt(new_key)?;

        sqlx::query(
            r#"
            INSERT INTO encryption_configs (config_name, encryption_key_encrypted, key_version)
            VALUES ($1, $2, (SELECT COALESCE(MAX(key_version), 0) + 1 FROM encryption_configs WHERE config_name = $1))
            ON CONFLICT (config_name) 
            DO UPDATE SET 
                encryption_key_encrypted = EXCLUDED.encryption_key_encrypted,
                key_version = EXCLUDED.key_version,
                updated_at = NOW()
            "#
        )
        .bind(config_name)
        .bind(&encrypted_key)
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    // 获取日志配置
    pub async fn get_log_config(&self) -> Result<LogConfig, ServiceError> {
        let level = self
            .get_system_config("log.level")
            .await?
            .and_then(|v| match v {
                ConfigValue::String(s) => Some(s),
                _ => None,
            })
            .unwrap_or_else(|| "info".to_string());

        let enable_database_logging = self
            .get_system_config("log.enable_database_logging")
            .await?
            .and_then(|v| match v {
                ConfigValue::Boolean(b) => Some(b),
                _ => None,
            })
            .unwrap_or(true);

        let enable_file_logging = self
            .get_system_config("log.enable_file_logging")
            .await?
            .and_then(|v| match v {
                ConfigValue::Boolean(b) => Some(b),
                _ => None,
            })
            .unwrap_or(false);

        let log_file_path = self
            .get_system_config("log.file_path")
            .await?
            .and_then(|v| match v {
                ConfigValue::String(s) => Some(s),
                _ => None,
            });

        let max_file_size =
            self.get_system_config("log.max_file_size")
                .await?
                .and_then(|v| match v {
                    ConfigValue::Number(n) => Some(n as i64),
                    _ => None,
                });

        let retention_days = self
            .get_system_config("log.retention_days")
            .await?
            .and_then(|v| match v {
                ConfigValue::Number(n) => Some(n as i32),
                _ => None,
            });

        let enable_error_logging = self
            .get_system_config("log.enable_error_logging")
            .await?
            .and_then(|v| match v {
                ConfigValue::Boolean(b) => Some(b),
                _ => None,
            })
            .unwrap_or(true);

        let enable_operation_logging = self
            .get_system_config("log.enable_operation_logging")
            .await?
            .and_then(|v| match v {
                ConfigValue::Boolean(b) => Some(b),
                _ => None,
            })
            .unwrap_or(true);

        let enable_payment_logging = self
            .get_system_config("log.enable_payment_logging")
            .await?
            .and_then(|v| match v {
                ConfigValue::Boolean(b) => Some(b),
                _ => None,
            })
            .unwrap_or(true);

        Ok(LogConfig {
            level,
            enable_database_logging,
            enable_file_logging,
            log_file_path,
            max_file_size,
            retention_days,
            enable_error_logging,
            enable_operation_logging,
            enable_payment_logging,
        })
    }

    // 更新日志配置
    pub async fn update_log_config(
        &self,
        request: UpdateLogConfigRequest,
        admin_id: i32,
    ) -> Result<(), ServiceError> {
        if let Some(level) = request.level {
            self.set_system_config(
                "log.level",
                ConfigValue::String(level),
                Some("Log level configuration".to_string()),
                Some(admin_id),
            )
            .await?;
        }

        if let Some(enable_database_logging) = request.enable_database_logging {
            self.set_system_config(
                "log.enable_database_logging",
                ConfigValue::Boolean(enable_database_logging),
                Some("Enable database logging".to_string()),
                Some(admin_id),
            )
            .await?;
        }

        if let Some(enable_file_logging) = request.enable_file_logging {
            self.set_system_config(
                "log.enable_file_logging",
                ConfigValue::Boolean(enable_file_logging),
                Some("Enable file logging".to_string()),
                Some(admin_id),
            )
            .await?;
        }

        if let Some(log_file_path) = request.log_file_path {
            self.set_system_config(
                "log.file_path",
                ConfigValue::String(log_file_path),
                Some("Log file path".to_string()),
                Some(admin_id),
            )
            .await?;
        }

        if let Some(max_file_size) = request.max_file_size {
            self.set_system_config(
                "log.max_file_size",
                ConfigValue::Number(max_file_size as f64),
                Some("Maximum log file size in bytes".to_string()),
                Some(admin_id),
            )
            .await?;
        }

        if let Some(retention_days) = request.retention_days {
            self.set_system_config(
                "log.retention_days",
                ConfigValue::Number(retention_days as f64),
                Some("Log retention period in days".to_string()),
                Some(admin_id),
            )
            .await?;
        }

        if let Some(enable_error_logging) = request.enable_error_logging {
            self.set_system_config(
                "log.enable_error_logging",
                ConfigValue::Boolean(enable_error_logging),
                Some("Enable error logging".to_string()),
                Some(admin_id),
            )
            .await?;
        }

        if let Some(enable_operation_logging) = request.enable_operation_logging {
            self.set_system_config(
                "log.enable_operation_logging",
                ConfigValue::Boolean(enable_operation_logging),
                Some("Enable operation logging".to_string()),
                Some(admin_id),
            )
            .await?;
        }
    }
}
