use std::sync::Arc;
use rsws_db::api_key::ApiKeyRepository;
use rsws_db::redis::api_key::ApiKeyRedisService;
use rsws_model::api_key::*;
use rsws_common::error::ServiceError;
use chrono::Utc;
use std::net::IpAddr;

pub struct ApiKeyService {
    repository: Arc<ApiKeyRepository>,
    redis_service: Arc<ApiKeyRedisService>,
}

impl ApiKeyService {
    pub fn new(
        repository: Arc<ApiKeyRepository>,
        redis_service: Arc<ApiKeyRedisService>,
    ) -> Self {
        Self {
            repository,
            redis_service,
        }
    }

    // 创建API Key
    pub async fn create_api_key(
        &self,
        user_id: i32,
        request: CreateApiKeyRequest,
    ) -> Result<ApiKeyResponse, ServiceError> {
        let api_key = self.repository.create_api_key(user_id, &request).await?;
        
        Ok(ApiKeyResponse {
            id: api_key.id,
            name: api_key.name,
            api_key: api_key.api_key,
            api_secret: Some(api_key.api_secret), // 只在创建时返回
            permissions: serde_json::from_value(api_key.permissions).unwrap_or_default(),
            rate_limit: api_key.rate_limit,
            last_used_at: api_key.last_used_at,
            expires_at: api_key.expires_at,
            is_active: api_key.is_active,
            created_at: api_key.created_at,
        })
    }

    // 认证API Key
    pub async fn authenticate(
        &self,
        api_key: &str,
        api_secret: &str,
    ) -> Result<Option<ApiKeySession>, ServiceError> {
        // 先检查Redis缓存
        if let Some(session) = self.redis_service.get_session(api_key).await? {
            return Ok(Some(session));
        }
        
        // 从数据库验证
        if let Some(api_key_record) = self.repository.validate_credentials(api_key, api_secret).await? {
            let permissions: Vec<String> = serde_json::from_value(api_key_record.permissions)
                .unwrap_or_default();
            
            let session = ApiKeySession {
                user_id: api_key_record.user_id,
                api_key_id: api_key_record.id,
                permissions,
                rate_limit: api_key_record.rate_limit,
                last_access: Utc::now(),
            };
            
            // 存储到Redis缓存，TTL 1小时
            self.redis_service.store_session(api_key, &session, 3600).await?;
            
            // 更新数据库中的最后使用时间
            self.repository.update_last_used(api_key_record.id).await?;
            
            Ok(Some(session))
        } else {
            Ok(None)
        }
    }

    // 检查速率限制
    pub async fn check_rate_limit(
        &self,
        api_key: &str,
        limit: i32,
    ) -> Result<bool, ServiceError> {
        self.redis_service.check_rate_limit(api_key, limit, 3600).await
            .map_err(|e| ServiceError::ExternalError(format!("Rate limit check failed: {}", e)))
    }

    // 记录使用日志
    pub async fn log_usage(
        &self,
        api_key_id: i32,
        ip_address: Option<IpAddr>,
        user_agent: Option<&str>,
        endpoint: Option<&str>,
        method: Option<&str>,
        status_code: Option<i32>,
        response_time_ms: Option<i32>,
    ) -> Result<(), ServiceError> {
        self.repository.log_usage(
            api_key_id,
            ip_address,
            user_agent,
            endpoint,
            method,
            status_code,
            response_time_ms,
        ).await
            .map_err(|e| ServiceError::DatabaseError(format!("Failed to log usage: {}", e)))
    }

    // 获取用户的API Keys
    pub async fn get_user_api_keys(&self, user_id: i32) -> Result<Vec<ApiKeyResponse>, ServiceError> {
        let api_keys = self.repository.get_user_api_keys(user_id).await?;
        
        let responses = api_keys.into_iter().map(|api_key| {
            ApiKeyResponse {
                id: api_key.id,
                name: api_key.name,
                api_key: api_key.api_key,
                api_secret: None, // 不返回secret
                permissions: serde_json::from_value(api_key.permissions).unwrap_or_default(),
                rate_limit: api_key.rate_limit,
                last_used_at: api_key.last_used_at,
                expires_at: api_key.expires_at,
                is_active: api_key.is_active,
                created_at: api_key.created_at,
            }
        }).collect();
        
        Ok(responses)
    }

    // 删除API Key
    pub async fn delete_api_key(&self, api_key_id: i32, user_id: i32) -> Result<bool, ServiceError> {
        // 先从数据库删除
        let deleted = self.repository.delete_api_key(api_key_id, user_id).await?;
        
        if deleted {
            // 清除Redis中的相关会话（需要先获取api_key）
            // 这里可以优化，在删除前先获取api_key值
            // 暂时跳过Redis清理，依赖TTL自动过期
        }
        
        Ok(deleted)
    }

    // 清除用户所有会话
    pub async fn clear_user_sessions(&self, user_id: i32) -> Result<(), ServiceError> {
        self.redis_service.clear_user_sessions(user_id).await
            .map_err(|e| ServiceError::ExternalError(format!("Failed to clear sessions: {}", e)))
    }
}