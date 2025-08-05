use sqlx::PgPool;
use chrono::{DateTime, Utc, Duration};
use rand::Rng;
use sha2::{Sha256, Digest};
use base64::{Engine as _, engine::general_purpose};
use rsws_model::api_key::*;
use rsws_common::error::DbError;

pub struct ApiKeyRepository {
    pool: PgPool,
}

impl ApiKeyRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // 生成API Key和Secret
    fn generate_api_credentials() -> (String, String) {
        let mut rng = rand::thread_rng();
        
        // 生成32字节的随机数据作为API Key
        let api_key_bytes: [u8; 32] = rng.gen();
        let api_key = format!("ak_{}", general_purpose::URL_SAFE_NO_PAD.encode(&api_key_bytes));
        
        // 生成64字节的随机数据作为API Secret
        let secret_bytes: [u8; 64] = rng.gen();
        let api_secret = format!("sk_{}", general_purpose::URL_SAFE_NO_PAD.encode(&secret_bytes));
        
        (api_key, api_secret)
    }

    // 创建API Key
    pub async fn create_api_key(
        &self,
        user_id: i32,
        request: &CreateApiKeyRequest,
    ) -> Result<ApiKey, DbError> {
        let (api_key, api_secret) = Self::generate_api_credentials();
        
        // 计算过期时间
        let expires_at = request.expires_in_days.map(|days| {
            Utc::now() + Duration::days(days as i64)
        });
        
        let permissions_json = serde_json::to_value(&request.permissions)
            .map_err(|e| DbError::SerializationError(format!("Failed to serialize permissions: {}", e)))?;
        
        let api_key_record = sqlx::query_as::<_, ApiKey>(
            r#"
            INSERT INTO api_keys (user_id, api_key, api_secret, name, permissions, rate_limit, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#
        )
        .bind(user_id)
        .bind(&api_key)
        .bind(&api_secret)
        .bind(&request.name)
        .bind(&permissions_json)
        .bind(request.rate_limit.unwrap_or(1000))
        .bind(expires_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::QueryError(format!("Failed to create API key: {}", e)))?;
        
        Ok(api_key_record)
    }

    // 根据API Key获取记录
    pub async fn get_by_api_key(&self, api_key: &str) -> Result<Option<ApiKey>, DbError> {
        let api_key_record = sqlx::query_as::<_, ApiKey>(
            "SELECT * FROM api_keys WHERE api_key = $1 AND is_active = true"
        )
        .bind(api_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::QueryError(format!("Failed to get API key: {}", e)))?;
        
        Ok(api_key_record)
    }

    // 验证API Key和Secret
    pub async fn validate_credentials(
        &self,
        api_key: &str,
        api_secret: &str,
    ) -> Result<Option<ApiKey>, DbError> {
        let api_key_record = sqlx::query_as::<_, ApiKey>(
            r#"
            SELECT * FROM api_keys 
            WHERE api_key = $1 AND api_secret = $2 AND is_active = true
            AND (expires_at IS NULL OR expires_at > NOW())
            "#
        )
        .bind(api_key)
        .bind(api_secret)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::QueryError(format!("Failed to validate credentials: {}", e)))?;
        
        Ok(api_key_record)
    }

    // 更新最后使用时间
    pub async fn update_last_used(&self, api_key_id: i32) -> Result<(), DbError> {
        sqlx::query(
            "UPDATE api_keys SET last_used_at = NOW() WHERE id = $1"
        )
        .bind(api_key_id)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::QueryError(format!("Failed to update last used: {}", e)))?;
        
        Ok(())
    }

    // 获取用户的所有API Key
    pub async fn get_user_api_keys(&self, user_id: i32) -> Result<Vec<ApiKey>, DbError> {
        let api_keys = sqlx::query_as::<_, ApiKey>(
            "SELECT * FROM api_keys WHERE user_id = $1 ORDER BY created_at DESC"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::QueryError(format!("Failed to get user API keys: {}", e)))?;
        
        Ok(api_keys)
    }

    // 删除API Key
    pub async fn delete_api_key(&self, api_key_id: i32, user_id: i32) -> Result<bool, DbError> {
        let result = sqlx::query(
            "DELETE FROM api_keys WHERE id = $1 AND user_id = $2"
        )
        .bind(api_key_id)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::QueryError(format!("Failed to delete API key: {}", e)))?;
        
        Ok(result.rows_affected() > 0)
    }

    // 记录API Key使用日志
    pub async fn log_usage(
        &self,
        api_key_id: i32,
        ip_address: Option<std::net::IpAddr>,
        user_agent: Option<&str>,
        endpoint: Option<&str>,
        method: Option<&str>,
        status_code: Option<i32>,
        response_time_ms: Option<i32>,
    ) -> Result<(), DbError> {
        sqlx::query(
            r#"
            INSERT INTO api_key_usage_logs 
            (api_key_id, ip_address, user_agent, endpoint, method, status_code, response_time_ms)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#
        )
        .bind(api_key_id)
        .bind(ip_address)
        .bind(user_agent)
        .bind(endpoint)
        .bind(method)
        .bind(status_code)
        .bind(response_time_ms)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::QueryError(format!("Failed to log usage: {}", e)))?;
        
        Ok(())
    }
}