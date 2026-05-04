//! 认证服务

use rsws_common::error::RswsError;
use rsws_common::error_code::ErrorCode;
use rsws_common::signature::SignatureService;
use rsws_db::ApiKeyRepository;
use std::sync::Arc;

/// 认证服务
pub struct AuthService {
    api_key_repo: Arc<ApiKeyRepository>,
}

impl AuthService {
    /// 创建认证服务实例
    pub fn new(api_key_repo: Arc<ApiKeyRepository>) -> Self {
        Self { api_key_repo }
    }

    /// 验证 API Key 签名
    pub async fn verify_signature(
        &self,
        api_key: &str,
        signature: &str,
        method: &str,
        path: &str,
        timestamp: u64,
        nonce: &str,
        body: &str,
    ) -> Result<i64, RswsError> {
        // 获取 API Key 记录
        let api_key_record = self.api_key_repo
            .get_by_api_key(api_key)
            .await?
            .ok_or_else(|| RswsError::business(ErrorCode::AUTH_INVALID_API_KEY))?;

        // 检查是否过期
        if let Some(expires_at) = api_key_record.expires_at {
            if expires_at < chrono::Utc::now() {
                return Err(RswsError::business(ErrorCode::AUTH_API_KEY_EXPIRED));
            }
        }

        // 验证签名
        let valid = SignatureService::verify(
            &api_key_record.api_secret,
            method,
            path,
            timestamp,
            nonce,
            body,
            signature,
        )?;

        if !valid {
            return Err(RswsError::business(ErrorCode::AUTH_INVALID_SIGNATURE));
        }

        // 检查时间戳是否在有效范围内
        if !SignatureService::is_timestamp_valid(timestamp, 300) {
            return Err(RswsError::business(ErrorCode::AUTH_TIMESTAMP_EXPIRED));
        }

        // 更新最后使用时间
        self.api_key_repo.update_last_used(api_key_record.id).await?;

        Ok(api_key_record.user_id)
    }
}
