//! 认证服务
//!
//! 新设计（Cregis 单密钥方案）：
//! - api_key 作为签名密钥，前端持有用于签名
//! - 验证签名：通过 user_id 查找 api_key，重算签名对比
//! - 不再需要 api_secret
//! - 纯 Redis 实现，无数据库依赖

use rsws_common::error::RswsError;
use rsws_common::error_code::ErrorCode;
use std::collections::HashMap;
use std::sync::Arc;

use crate::api_key_service::ApiKeyService;

/// 认证服务（使用 Redis-only ApiKeyService）
pub struct AuthService {
    api_key_service: Arc<ApiKeyService>,
}

impl AuthService {
    /// 创建认证服务实例
    pub fn new(redis: Arc<rsws_db::RedisService>) -> Self {
        Self {
            api_key_service: Arc::new(ApiKeyService::new(redis)),
        }
    }

    /// 验证 API Key 签名（Cregis 方案）
    ///
    /// 新流程：
    /// 1. 通过 user_id 从 Redis 查找 api_key
    /// 2. 用同样算法重算签名（api_key 拼在排序参数前面 → MD5）
    /// 3. 对比签名，一致则通过
    pub async fn verify_signature(
        &self,
        user_id: i64,
        params: &HashMap<String, String>,
        sign: &str,
    ) -> Result<i64, RswsError> {
        match self
            .api_key_service
            .validate_signature_by_user_id(user_id, params, sign)
            .await?
        {
            Some(_api_key_record) => Ok(user_id),
            None => Err(RswsError::business(ErrorCode::AUTH_INVALID_SIGNATURE)),
        }
    }
}
