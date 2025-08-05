use redis::{AsyncCommands, RedisResult};
use serde_json;
use chrono::{DateTime, Utc, Duration};
use rsws_model::api_key::{ApiKeySession, Permission};
use rsws_common::error::DbError;
use std::collections::HashMap;

pub struct ApiKeyRedisService {
    redis_client: redis::Client,
}

impl ApiKeyRedisService {
    pub fn new(redis_url: &str) -> Result<Self, DbError> {
        let redis_client = redis::Client::open(redis_url)
            .map_err(|e| DbError::ConnectionError(format!("Redis connection failed: {}", e)))?;
        
        Ok(Self { redis_client })
    }

    // 存储API Key会话
    pub async fn store_session(
        &self,
        api_key: &str,
        session: &ApiKeySession,
        ttl_seconds: u64,
    ) -> Result<(), DbError> {
        let mut conn = self.redis_client.get_async_connection().await
            .map_err(|e| DbError::ConnectionError(format!("Redis connection failed: {}", e)))?;
        
        let session_key = format!("api_session:{}", api_key);
        let session_json = serde_json::to_string(session)
            .map_err(|e| DbError::SerializationError(format!("Failed to serialize session: {}", e)))?;
        
        conn.setex(&session_key, ttl_seconds, session_json).await
            .map_err(|e| DbError::OperationError(format!("Failed to store session: {}", e)))?;
        
        Ok(())
    }

    // 获取API Key会话
    pub async fn get_session(&self, api_key: &str) -> Result<Option<ApiKeySession>, DbError> {
        let mut conn = self.redis_client.get_async_connection().await
            .map_err(|e| DbError::ConnectionError(format!("Redis connection failed: {}", e)))?;
        
        let session_key = format!("api_session:{}", api_key);
        let session_json: Option<String> = conn.get(&session_key).await
            .map_err(|e| DbError::OperationError(format!("Failed to get session: {}", e)))?;
        
        match session_json {
            Some(json) => {
                let session: ApiKeySession = serde_json::from_str(&json)
                    .map_err(|e| DbError::SerializationError(format!("Failed to deserialize session: {}", e)))?;
                Ok(Some(session))
            },
            None => Ok(None),
        }
    }

    // 删除API Key会话
    pub async fn delete_session(&self, api_key: &str) -> Result<(), DbError> {
        let mut conn = self.redis_client.get_async_connection().await
            .map_err(|e| DbError::ConnectionError(format!("Redis connection failed: {}", e)))?;
        
        let session_key = format!("api_session:{}", api_key);
        conn.del(&session_key).await
            .map_err(|e| DbError::OperationError(format!("Failed to delete session: {}", e)))?;
        
        Ok(())
    }

    // 更新会话最后访问时间
    pub async fn update_last_access(&self, api_key: &str) -> Result<(), DbError> {
        if let Some(mut session) = self.get_session(api_key).await? {
            session.last_access = Utc::now();
            self.store_session(api_key, &session, 3600).await?; // 1小时TTL
        }
        Ok(())
    }

    // 检查和更新速率限制
    pub async fn check_rate_limit(
        &self,
        api_key: &str,
        limit: i32,
        window_seconds: u64,
    ) -> Result<bool, DbError> {
        let mut conn = self.redis_client.get_async_connection().await
            .map_err(|e| DbError::ConnectionError(format!("Redis connection failed: {}", e)))?;
        
        let rate_key = format!("rate_limit:{}", api_key);
        let current_count: i32 = conn.get(&rate_key).await.unwrap_or(0);
        
        if current_count >= limit {
            return Ok(false); // 超出限制
        }
        
        // 增加计数
        let new_count: i32 = conn.incr(&rate_key, 1).await
            .map_err(|e| DbError::OperationError(format!("Failed to increment rate limit: {}", e)))?;
        
        // 如果是第一次访问，设置过期时间
        if new_count == 1 {
            conn.expire(&rate_key, window_seconds as usize).await
                .map_err(|e| DbError::OperationError(format!("Failed to set rate limit expiry: {}", e)))?;
        }
        
        Ok(true)
    }

    // 获取当前速率限制状态
    pub async fn get_rate_limit_status(
        &self,
        api_key: &str,
    ) -> Result<(i32, i32), DbError> { // (current_count, remaining_ttl)
        let mut conn = self.redis_client.get_async_connection().await
            .map_err(|e| DbError::ConnectionError(format!("Redis connection failed: {}", e)))?;
        
        let rate_key = format!("rate_limit:{}", api_key);
        let current_count: i32 = conn.get(&rate_key).await.unwrap_or(0);
        let ttl: i32 = conn.ttl(&rate_key).await.unwrap_or(-1);
        
        Ok((current_count, ttl))
    }

    // 清除用户的所有会话
    pub async fn clear_user_sessions(&self, user_id: i32) -> Result<(), DbError> {
        let mut conn = self.redis_client.get_async_connection().await
            .map_err(|e| DbError::ConnectionError(format!("Redis connection failed: {}", e)))?;
        
        // 扫描所有api_session:*键
        let pattern = "api_session:*";
        let keys: Vec<String> = conn.keys(pattern).await
            .map_err(|e| DbError::OperationError(format!("Failed to scan keys: {}", e)))?;
        
        for key in keys {
            if let Ok(Some(session_json)) = conn.get::<_, Option<String>>(&key).await {
                if let Ok(session) = serde_json::from_str::<ApiKeySession>(&session_json) {
                    if session.user_id == user_id {
                        let _: () = conn.del(&key).await
                            .map_err(|e| DbError::OperationError(format!("Failed to delete session: {}", e)))?;
                    }
                }
            }
        }
        
        Ok(())
    }
}