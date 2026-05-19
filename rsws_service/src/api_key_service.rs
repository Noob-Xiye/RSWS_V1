//! API Key service (Pure Redis, no DB)
//!
//! Cregis scheme:
//! - api_key is signing key, held by frontend for signing, NOT transmitted
//! - Backend looks up api_key by user_id for verification
//! - No database needed

use rsws_common::error::RswsError;
use rsws_common::utils::generate_api_key;
use rsws_db::RedisService;
use rsws_model::api_key::{ApiKey, ApiKeyResponse, CreateApiKeyRequest};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// CachedApiKey stored in Redis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedApiKey {
    pub user_id: i64,
    pub api_key_id: i64,
    pub api_key: String,
    pub permissions: Vec<String>,
    pub rate_limit: i32,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// ApiKeyService (Pure Redis)
pub struct ApiKeyService {
    redis: Arc<RedisService>,
}

impl ApiKeyService {
    /// Create ApiKeyService (Redis only, no DB)
    pub fn new(redis: Arc<RedisService>) -> Self {
        Self { redis }
    }

    /// Redis key format: apikey:user:{user_id}
    fn redis_key(user_id: i64) -> String {
        format!("apikey:user:{}", user_id)
    }

    /// Default session TTL (seconds): 7 days
    const DEFAULT_SESSION_TTL: u64 = 7 * 24 * 3600;

    /// Get session TTL (from Redis config, default 7 days)
    async fn session_ttl(&self) -> u64 {
        self.redis
            .get("config:api_key.session_expire_days")
            .await
            .ok()
            .flatten()
            .and_then(|v| v.parse::<u64>().ok())
            .map(|days| days * 24 * 3600)
            .unwrap_or(Self::DEFAULT_SESSION_TTL)
    }

    /// Validate API Key by user_id (for signature verification)
    pub async fn validate_by_user_id(&self, user_id: i64) -> Result<Option<ApiKey>, RswsError> {
        if let Some(cached) = self
            .redis
            .get_json::<CachedApiKey>(&Self::redis_key(user_id))
            .await?
        {
            // Check expiry
            if let Some(expires) = cached.expires_at {
                if expires < chrono::Utc::now() {
                    let _ = self.redis.del(&Self::redis_key(user_id)).await;
                    return Ok(None);
                }
            }
            // Reconstruct ApiKey from Redis cache
            return Ok(Some(ApiKey {
                id: cached.api_key_id,
                user_id: cached.user_id,
                api_key: cached.api_key,
                name: String::new(),
                permissions: serde_json::to_value(&cached.permissions).unwrap_or_default(),
                rate_limit: cached.rate_limit,
                last_used_at: None,
                expires_at: cached.expires_at,
                is_active: true,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            }));
        }

        // Not found in Redis = not logged in or expired
        Ok(None)
    }

    /// Create API Key (Redis only, no DB write)
    pub async fn create(
        &self,
        user_id: i64,
        request: CreateApiKeyRequest,
    ) -> Result<ApiKeyResponse, RswsError> {
        // Generate api_key (as signing key)
        let api_key = generate_api_key();

        // Calculate expiry
        let expires_at = request
            .expires_in_days
            .map(|days| chrono::Utc::now() + chrono::Duration::days(days as i64));

        // Build CachedApiKey
        let cached = CachedApiKey {
            user_id,
            api_key_id: chrono::Utc::now().timestamp_millis(),
            api_key: api_key.clone(),
            permissions: request.permissions.clone(),
            rate_limit: request.rate_limit.unwrap_or(1000),
            expires_at,
        };

        // Write to Redis
        let ttl = self.session_ttl().await;
        self.redis
            .set_json(&Self::redis_key(user_id), &cached, ttl)
            .await?;

        Ok(ApiKeyResponse {
            id: cached.api_key_id,
            name: request.name,
            api_key,
            permissions: request.permissions,
            rate_limit: cached.rate_limit,
            last_used_at: None,
            expires_at,
            is_active: true,
            created_at: chrono::Utc::now(),
        })
    }

    /// Get user's API Keys (from Redis)
    pub async fn get_user_keys(&self, user_id: i64) -> Result<Vec<ApiKey>, RswsError> {
        if let Some(cached) = self
            .redis
            .get_json::<CachedApiKey>(&Self::redis_key(user_id))
            .await?
        {
            let api_key = ApiKey {
                id: cached.api_key_id,
                user_id: cached.user_id,
                api_key: cached.api_key,
                name: String::new(),
                permissions: serde_json::to_value(&cached.permissions).unwrap_or_default(),
                rate_limit: cached.rate_limit,
                last_used_at: None,
                expires_at: cached.expires_at,
                is_active: true,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            };
            return Ok(vec![api_key]);
        }
        Ok(vec![])
    }

    /// Delete API Key (from Redis)
    pub async fn delete(&self, _api_key_id: i64, user_id: i64) -> Result<bool, RswsError> {
        self.redis.del(&Self::redis_key(user_id)).await?;
        Ok(true)
    }

    /// Update last used time (not needed in Redis scheme)
    pub async fn update_last_used(&self, _api_key_id: i64) -> Result<(), RswsError> {
        Ok(())
    }

    /// Deactivate API Key (delete Redis key)
    pub async fn deactivate_key(&self, user_id: i64) -> Result<bool, RswsError> {
        self.redis.del(&Self::redis_key(user_id)).await?;
        Ok(true)
    }

    /// On password change, delete user's API Key (delete Redis key)
    pub async fn on_password_change(&self, user_id: i64) -> Result<(), RswsError> {
        let _ = self.redis.del(&Self::redis_key(user_id)).await;
        Ok(())
    }

    /// Validate signature (Cregis scheme: lookup api_key by user_id)
    pub async fn validate_signature_by_user_id(
        &self,
        user_id: i64,
        params: &HashMap<String, String>,
        sign: &str,
    ) -> Result<Option<ApiKey>, RswsError> {
        let api_key_record = self.validate_by_user_id(user_id).await?;
        let api_key_record = match api_key_record {
            Some(r) => r,
            None => return Ok(None),
        };

        let api_key = &api_key_record.api_key;

        // Recompute signature (Cregis: api_key prepended to sorted params)
        let computed_sign = rsws_common::signature::compute_cregis_signature(params, api_key);

        // Compare signatures
        if computed_sign == sign {
            Ok(Some(api_key_record))
        } else {
            tracing::warn!(
                "Signature mismatch for user_id: {}. Expected: {}, Got: {}",
                user_id,
                computed_sign,
                sign
            );
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_api_key() {
        let api_key = ApiKeyService::generate_api_key();
        assert!(api_key.starts_with("ak_"));
        assert!(api_key.len() > 10);
    }

    #[test]
    fn test_generate_api_key_unique() {
        let key1 = ApiKeyService::generate_api_key();
        let key2 = ApiKeyService::generate_api_key();
        assert_ne!(key1, key2);
    }
}
