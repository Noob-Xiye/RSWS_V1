//! 用户支付配置服务

use rsws_common::error::RswsError;
use sqlx::PgPool;

/// 用户支付配置服务
pub struct UserPaymentService {
    pool: PgPool,
}

impl UserPaymentService {
    /// 创建用户支付配置服务实例
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 获取用户支付配置
    pub async fn get_user_configs(
        &self,
        user_id: i64,
    ) -> Result<Vec<rsws_model::payment::UserPaymentConfig>, RswsError> {
        let configs = sqlx::query_as::<_, rsws_model::payment::UserPaymentConfig>(
            "SELECT id, user_id, payment_method, account_address, account_name, is_active, created_at, updated_at FROM user_payment_configs WHERE user_id = $1 AND is_active = true",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to get user payment configs: {}", e)))?;

        Ok(configs)
    }
}
