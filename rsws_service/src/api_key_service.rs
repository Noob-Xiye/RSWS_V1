//! API Key 服务

use std::sync::Arc;
use rsws_db::ApiKeyRepository;
use rsws_model::api_key::{ApiKey, CreateApiKeyRequest, ApiKeyResponse};
use rsws_common::error::RswsError;

/// API Key 服务
pub struct ApiKeyService {
    repository: Arc<ApiKeyRepository>,
}

impl ApiKeyService {
    /// 创建 API Key 服务实例
    pub fn new(repository: Arc<ApiKeyRepository>) -> Self {
        Self { repository }
    }

    /// 创建 API Key
    pub async fn create(
        &self,
        user_id: i64,
        request: CreateApiKeyRequest,
    ) -> Result<ApiKeyResponse, RswsError> {
        let (api_key, api_secret) = self.repository
            .create(user_id, &request)
            .await?;

        Ok(ApiKeyResponse {
            id: api_key.id,
            name: api_key.name,
            api_key: api_key.api_key,
            api_secret: Some(api_secret),
            permissions: request.permissions,
            rate_limit: api_key.rate_limit,
            last_used_at: api_key.last_used_at,
            expires_at: api_key.expires_at,
            is_active: api_key.is_active,
            created_at: api_key.created_at,
        })
    }

    /// 验证 API Key
    pub async fn validate(
        &self,
        api_key: &str,
        api_secret: &str,
    ) -> Result<Option<ApiKey>, RswsError> {
        self.repository.validate(api_key, api_secret).await
    }

    /// 获取用户的 API Keys
    pub async fn get_user_keys(&self, user_id: i64) -> Result<Vec<ApiKey>, RswsError> {
        self.repository.get_user_api_keys(user_id).await
    }

    /// 删除 API Key
    pub async fn delete(&self, api_key_id: i64, user_id: i64) -> Result<bool, RswsError> {
        self.repository.delete(api_key_id, user_id).await
    }
}
