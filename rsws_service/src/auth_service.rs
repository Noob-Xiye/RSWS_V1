use std::sync::Arc;
use sqlx::PgPool;
use chrono::{Utc, Duration};
use rand::Rng;
use base64::{Engine as _, engine::general_purpose};
use rsws_model::auth::*;
use rsws_common::{snowflake, signature::SignatureService};
use rsws_common::error::ServiceError;
use rsws_db::redis::api_key::ApiKeyRedisService;
use std::net::IpAddr;
use std::collections::BTreeMap;

pub struct AuthService {
    db_pool: PgPool,
    redis_service: Arc<ApiKeyRedisService>,
}

impl AuthService {
    pub fn new(db_pool: PgPool, redis_service: Arc<ApiKeyRedisService>) -> Self {
        Self {
            db_pool,
            redis_service,
        }
    }

    // 用户登录
    pub async fn login(
        &self,
        request: LoginRequest,
        ip_address: Option<IpAddr>,
        user_agent: Option<String>,
    ) -> Result<LoginResponse, ServiceError> {
        // 验证用户凭据
        let user = self.validate_user_credentials(&request.username, &request.password).await?;
        
        // 生成会话信息
        let session_id = snowflake::next_id();
        let session_token = self.generate_session_token();
        let (api_key, api_secret) = self.generate_api_credentials();
        
        // 设置过期时间（7天）
        let expires_at = Utc::now() + Duration::days(7);
        
        // 存储会话到数据库
        let user_session = sqlx::query_as::<_, UserSession>(
            r#"
            INSERT INTO user_sessions 
            (id, user_id, session_token, api_key, api_secret, device_info, ip_address, user_agent, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#
        )
        .bind(session_id)
        .bind(user.id)
        .bind(&session_token)
        .bind(&api_key)
        .bind(&api_secret)
        .bind(request.device_info.map(|d| serde_json::to_value(d).unwrap()))
        .bind(ip_address)
        .bind(&user_agent)
        .bind(expires_at)
        .fetch_one(&self.db_pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(format!("Failed to create session: {}", e)))?;
        
        // 存储到Redis缓存
        let session_cache = serde_json::json!({
            "user_id": user.id,
            "session_id": session_id,
            "permissions": user.permissions,
            "expires_at": expires_at
        });
        
        self.redis_service.store_session(
            &api_key,
            &session_cache.to_string(),
            7 * 24 * 3600, // 7天TTL
        ).await?;
        
        Ok(LoginResponse {
            session_token,
            api_key,
            api_secret,
            expires_at,
            user_info: UserInfo {
                id: user.id,
                username: user.username,
                email: user.email,
                nickname: user.nickname,
                avatar: user.avatar,
                permissions: user.permissions,
            },
        })
    }

    // 验证API请求签名
    pub async fn validate_api_request(
        &self,
        method: &str,
        path: &str,
        api_key: &str,
        timestamp: i64,
        nonce: &str,
        signature: &str,
        body: Option<&str>,
        query_params: Option<&BTreeMap<String, String>>,
    ) -> Result<SignatureValidation, ServiceError> {
        // 从Redis获取会话信息
        let session_data = match self.redis_service.get_session(api_key).await? {
            Some(data) => data,
            None => {
                // Redis中没有，从数据库查询
                match self.get_session_from_db(api_key).await? {
                    Some(session) => {
                        // 重新缓存到Redis
                        let session_cache = serde_json::json!({
                            "user_id": session.user_id,
                            "session_id": session.id,
                            "api_secret": session.api_secret,
                            "expires_at": session.expires_at
                        });
                        
                        self.redis_service.store_session(
                            api_key,
                            &session_cache.to_string(),
                            3600, // 1小时TTL
                        ).await?;
                        
                        session_cache.to_string()
                    },
                    None => {
                        return Ok(SignatureValidation {
                            is_valid: false,
                            user_session: None,
                            error_message: Some("Invalid API key".to_string()),
                        });
                    }
                }
            }
        };
        
        let session_info: serde_json::Value = serde_json::from_str(&session_data)
            .map_err(|_| ServiceError::InternalError("Invalid session data".to_string()))?;
        
        let api_secret = session_info["api_secret"].as_str()
            .ok_or_else(|| ServiceError::InternalError("Missing API secret".to_string()))?;
        
        // 验证签名
        let is_valid = SignatureService::verify_signature(
            api_secret,
            method,
            path,
            timestamp,
            nonce,
            signature,
            body,
            query_params,
        ).map_err(|e| ServiceError::ValidationError(format!("Signature validation failed: {}", e)))?;
        
        if is_valid {
            // 更新最后活动时间
            self.update_session_activity(api_key).await?;
            
            // 获取完整会话信息
            let user_session = self.get_session_from_db(api_key).await?;
            
            Ok(SignatureValidation {
                is_valid: true,
                user_session,
                error_message: None,
            })
        } else {
            Ok(SignatureValidation {
                is_valid: false,
                user_session: None,
                error_message: Some("Invalid signature".to_string()),
            })
        }
    }

    // 登出
    pub async fn logout(&self, api_key: &str) -> Result<(), ServiceError> {
        // 从数据库删除会话
        sqlx::query("UPDATE user_sessions SET is_active = false WHERE api_key = $1")
            .bind(api_key)
            .execute(&self.db_pool)
            .await
            .map_err(|e| ServiceError::DatabaseError(format!("Failed to logout: {}", e)))?;
        
        // 从Redis删除缓存
        self.redis_service.delete_session(api_key).await?;
        
        Ok(())
    }

    // 生成会话令牌
    fn generate_session_token(&self) -> String {
        let mut rng = rand::thread_rng();
        let token_bytes: [u8; 32] = rng.gen();
        format!("st_{}", general_purpose::URL_SAFE_NO_PAD.encode(token_bytes))
    }

    // 生成API凭据
    fn generate_api_credentials(&self) -> (String, String) {
        let mut rng = rand::thread_rng();
        
        let api_key_bytes: [u8; 24] = rng.gen();
        let api_key = format!("ak_{}", general_purpose::URL_SAFE_NO_PAD.encode(api_key_bytes));
        
        let api_secret_bytes: [u8; 32] = rng.gen();
        let api_secret = format!("sk_{}", general_purpose::URL_SAFE_NO_PAD.encode(api_secret_bytes));
        
        (api_key, api_secret)
    }

    // 从数据库获取会话
    async fn get_session_from_db(&self, api_key: &str) -> Result<Option<UserSession>, ServiceError> {
        let session = sqlx::query_as::<_, UserSession>(
            "SELECT * FROM user_sessions WHERE api_key = $1 AND is_active = true AND expires_at > NOW()"
        )
        .bind(api_key)
        .fetch_optional(&self.db_pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(format!("Failed to get session: {}", e)))?;
        
        Ok(session)
    }

    // 更新会话活动时间
    async fn update_session_activity(&self, api_key: &str) -> Result<(), ServiceError> {
        sqlx::query("UPDATE user_sessions SET last_activity = NOW() WHERE api_key = $1")
            .bind(api_key)
            .execute(&self.db_pool)
            .await
            .map_err(|e| ServiceError::DatabaseError(format!("Failed to update activity: {}", e)))?;
        
        Ok(())
    }

    // 验证用户凭据（需要实现）
    async fn validate_user_credentials(&self, username: &str, password: &str) -> Result<User, ServiceError> {
        // TODO: 实现用户验证逻辑
        todo!("Implement user validation")
    }
}