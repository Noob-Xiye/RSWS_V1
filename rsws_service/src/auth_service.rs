//! 认证服务
//!
//! 新设计（Cregis 单密钥方案）：
//! - api_key 作为签名密钥，前端持有用于签名
//! - 验证签名：通过 user_id 查找 api_key，重算签名对比
//! - 不再需要 api_secret

use rsws_common::error::RswsError;
use rsws_common::error_code::ErrorCode;
use rsws_db::ApiKeyRepository;
use std::collections::HashMap;
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

    /// 验证 API Key 签名（Cregis 方案）
    ///
    /// 新流程：
    /// 1. 通过 user_id 查找 api_key
    /// 2. 用同样算法重算签名（api_key 拼在排序参数前面 → MD5）
    /// 3. 对比签名，一致则通过
    pub async fn verify_signature(
        &self,
        user_id: i64,
        params: &HashMap<String, String>,
        sign: &str,
    ) -> Result<i64, RswsError> {
        // 1) 查找 api_key
        let api_key_record = self
            .api_key_repo
            .get_active_key_by_user_id(user_id)
            .await?
            .ok_or_else(|| RswsError::business(ErrorCode::AUTH_INVALID_API_KEY))?;

        // 检查是否过期
        if let Some(expires_at) = api_key_record.expires_at {
            if expires_at < chrono::Utc::now() {
                return Err(RswsError::business(ErrorCode::AUTH_API_KEY_EXPIRED));
            }
        }

        // 2) 重算签名（Cregis: api_key 拼在排序参数前面）
        let computed_sign =
            rsws_common::signature::compute_cregis_signature(params, &api_key_record.api_key);

        // 3) 对比签名
        if computed_sign != sign {
            return Err(RswsError::business(ErrorCode::AUTH_INVALID_SIGNATURE));
        }

        // 4) 更新最后使用时间
        self.api_key_repo
            .update_last_used(api_key_record.id)
            .await?;

        Ok(api_key_record.user_id)
    }
}
