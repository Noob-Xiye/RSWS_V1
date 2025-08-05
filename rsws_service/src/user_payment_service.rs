use rsws_common::error::ServiceError;
use rsws_common::snowflake;
use rsws_model::payment::{UserPaymentConfig, CreateUserPaymentConfigRequest};
use chrono::Utc;
use sqlx::PgPool;
use std::sync::Arc;
use tracing::info;

pub struct UserPaymentService {
    db_pool: Arc<PgPool>,
}

impl UserPaymentService {
    pub fn new(db_pool: Arc<PgPool>) -> Self {
        Self { db_pool }
    }

    // 创建用户收款配置 - 支持多种支付方式并存
    pub async fn create_user_payment_config(
        &self,
        user_id: i64,
        request: CreateUserPaymentConfigRequest,
    ) -> Result<i64, ServiceError> {
        // 验证支付方式
        self.validate_payment_method(&request.payment_method)?;
        
        // 验证账户地址格式
        self.validate_account_address(&request.payment_method, &request.account_address)?;
        
        // 检查是否已存在相同支付方式和地址的配置
        let existing_config = sqlx::query!(
            r#"
            SELECT id FROM user_payment_configs 
            WHERE user_id = $1 AND payment_method = $2 AND account_address = $3 AND is_active = true
            "#,
            user_id,
            request.payment_method,
            request.account_address
        )
        .fetch_optional(&*self.db_pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        if existing_config.is_some() {
            return Err(ServiceError::BusinessError("该收款配置已存在".to_string()));
        }

        // 创建新配置（不删除旧配置，允许多个配置并存）
        let config_id = snowflake::next_id();
        sqlx::query!(
            r#"
            INSERT INTO user_payment_configs 
            (id, user_id, payment_method, account_address, account_name, is_active)
            VALUES ($1, $2, $3, $4, $5, true)
            "#,
            config_id,
            user_id,
            request.payment_method,
            request.account_address,
            request.account_name
        )
        .execute(&*self.db_pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        info!(
            "User payment config created: {} for user {} ({})", 
            config_id, user_id, request.payment_method
        );
        Ok(config_id)
    }

    // 获取用户所有收款配置（按支付方式分组）
    pub async fn get_user_payment_configs(
        &self,
        user_id: i64,
    ) -> Result<Vec<UserPaymentConfig>, ServiceError> {
        let configs = sqlx::query_as!(
            UserPaymentConfig,
            r#"
            SELECT id, user_id, payment_method, account_address, 
                   account_name, is_active, created_at, updated_at
            FROM user_payment_configs 
            WHERE user_id = $1 AND is_active = true
            ORDER BY payment_method, created_at DESC
            "#,
            user_id
        )
        .fetch_all(&*self.db_pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        Ok(configs)
    }

    // 获取用户特定支付方式的配置列表
    pub async fn get_user_payment_configs_by_method(
        &self,
        user_id: i64,
        payment_method: &str,
    ) -> Result<Vec<UserPaymentConfig>, ServiceError> {
        let configs = sqlx::query_as!(
            UserPaymentConfig,
            r#"
            SELECT id, user_id, payment_method, account_address, 
                   account_name, is_active, created_at, updated_at
            FROM user_payment_configs 
            WHERE user_id = $1 AND payment_method = $2 AND is_active = true
            ORDER BY created_at DESC
            "#,
            user_id,
            payment_method
        )
        .fetch_all(&*self.db_pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        Ok(configs)
    }

    // 获取用户默认收款配置（每种支付方式的最新配置）
    pub async fn get_user_default_payment_config(
        &self,
        user_id: i64,
        payment_method: &str,
    ) -> Result<Option<UserPaymentConfig>, ServiceError> {
        let config = sqlx::query_as!(
            UserPaymentConfig,
            r#"
            SELECT id, user_id, payment_method, account_address, 
                   account_name, is_active, created_at, updated_at
            FROM user_payment_configs 
            WHERE user_id = $1 AND payment_method = $2 AND is_active = true
            ORDER BY created_at DESC
            LIMIT 1
            "#,
            user_id,
            payment_method
        )
        .fetch_optional(&*self.db_pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        Ok(config)
    }

    // 设置默认收款配置
    pub async fn set_default_payment_config(
        &self,
        user_id: i64,
        config_id: i64,
    ) -> Result<(), ServiceError> {
        // 验证配置是否属于该用户
        let config = sqlx::query!(
            "SELECT payment_method FROM user_payment_configs WHERE id = $1 AND user_id = $2 AND is_active = true",
            config_id,
            user_id
        )
        .fetch_optional(&*self.db_pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?
        .ok_or_else(|| ServiceError::NotFound("收款配置不存在".to_string()))?;

        // 更新配置的创建时间，使其成为最新的（默认）配置
        sqlx::query!(
            "UPDATE user_payment_configs SET updated_at = $1 WHERE id = $2",
            Utc::now(),
            config_id
        )
        .execute(&*self.db_pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        info!("Set default payment config: {} for user {}", config_id, user_id);
        Ok(())
    }

    // 删除用户收款配置
    pub async fn delete_user_payment_config(
        &self,
        user_id: i64,
        config_id: i64,
    ) -> Result<(), ServiceError> {
        let result = sqlx::query!(
            "UPDATE user_payment_configs SET is_active = false WHERE id = $1 AND user_id = $2",
            config_id,
            user_id
        )
        .execute(&*self.db_pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(ServiceError::NotFound("收款配置不存在".to_string()));
        }

        info!("User payment config deleted: {} for user {}", config_id, user_id);
        Ok(())
    }

    // 获取用户收款配置统计
    pub async fn get_user_payment_config_stats(
        &self,
        user_id: i64,
    ) -> Result<std::collections::HashMap<String, i64>, ServiceError> {
        let stats = sqlx::query!(
            r#"
            SELECT payment_method, COUNT(*) as count
            FROM user_payment_configs 
            WHERE user_id = $1 AND is_active = true
            GROUP BY payment_method
            "#,
            user_id
        )
        .fetch_all(&*self.db_pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        let mut result = std::collections::HashMap::new();
        for stat in stats {
            result.insert(stat.payment_method, stat.count.unwrap_or(0));
        }

        Ok(result)
    }

    // 验证支付方式
    fn validate_payment_method(&self, payment_method: &str) -> Result<(), ServiceError> {
        match payment_method {
            "paypal" | "usdt_tron" | "usdt_eth" => Ok(()),
            _ => Err(ServiceError::BusinessError("不支持的支付方式".to_string())),
        }
    }

    // 验证账户地址格式
    fn validate_account_address(
        &self,
        payment_method: &str,
        account_address: &str,
    ) -> Result<(), ServiceError> {
        match payment_method {
            "paypal" => {
                // 简单的邮箱格式验证
                if !account_address.contains('@') || !account_address.contains('.') {
                    return Err(ServiceError::BusinessError("PayPal邮箱格式不正确".to_string()));
                }
            },
            "usdt_tron" => {
                // TRON地址验证（以T开头，34位字符）
                if !account_address.starts_with('T') || account_address.len() != 34 {
                    return Err(ServiceError::BusinessError("TRON地址格式不正确".to_string()));
                }
            },
            "usdt_eth" => {
                // 以太坊地址验证（以0x开头，42位字符）
                if !account_address.starts_with("0x") || account_address.len() != 42 {
                    return Err(ServiceError::BusinessError("以太坊地址格式不正确".to_string()));
                }
            },
            _ => return Err(ServiceError::BusinessError("不支持的支付方式".to_string())),
        }
        Ok(())
    }
}
