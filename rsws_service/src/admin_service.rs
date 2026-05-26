//! 管理员服务
//!
//! 管理员 CRUD + Admin API Key 管理
//! 认证统一走 API Key,不使用 JWT
//! Admin API Key 凭证缓存到 Redis(快速验证 + 强制下线)

use rsws_common::error::RswsError;
use rsws_db::admin::AdminRepository;
use rsws_db::RedisService;
use rsws_model::user_models::admin::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Redis 中缓存的管理员 API Key 会话信息（单密钥方案）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedAdminApiKey {
    pub admin_id: i64,
    pub key_id: i64,
    pub api_key: String,
    pub role: String,
    pub permissions: Vec<String>,
    pub rate_limit: Option<i32>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// 管理员服务
pub struct AdminService {
    admin_repo: Arc<AdminRepository>,
    redis: Option<RedisService>,
}

impl AdminService {
    /// 创建管理员服务(无 Redis)
    pub fn new(admin_repo: AdminRepository) -> Self {
        Self {
            admin_repo: Arc::new(admin_repo),
            redis: None,
        }
    }

    /// 创建管理员服务(带 Redis 缓存)
    pub fn with_redis(admin_repo: AdminRepository, redis: RedisService) -> Self {
        Self {
            admin_repo: Arc::new(admin_repo),
            redis: Some(redis),
        }
    }

    /// Redis key 格式（按 admin_id 索引，和用户侧一致）
    fn redis_key(admin_id: i64) -> String {
        format!("admin_apikey:{}", admin_id)
    }

    /// 默认会话 TTL(秒)= 7 天
    const DEFAULT_SESSION_TTL: u64 = 7 * 24 * 3600;

    /// 管理员登录(验证密码,返回管理员信息)
    /// 注意:登录后客户端需自行使用 Admin API Key 调用受保护接口
    pub async fn login(
        &self,
        email: &str,
        password: &str,
        ip_address: Option<&str>,
    ) -> Result<AdminInfo, RswsError> {
        let admin = self
            .admin_repo
            .verify_admin_credentials(email, password)
            .await?
            .ok_or_else(|| RswsError::unauthorized("Invalid email or password"))?;

        if !admin.is_active {
            return Err(RswsError::forbidden("Account is disabled"));
        }

        // 更新登录信息
        self.admin_repo
            .update_admin_login(admin.id, ip_address)
            .await?;

        // 记录操作日志
        let _ = self
            .admin_repo
            .log_admin_operation(admin.id, "login", None, None, None, ip_address, None)
            .await;

        let permissions: Vec<String> =
            serde_json::from_value(admin.permissions.clone()).unwrap_or_default();

        Ok(AdminInfo {
            id: admin.id,
            email: admin.email,
            username: admin.username,
            nickname: admin.nickname,
            avatar_url: admin.avatar_url,
            is_active: admin.is_active,
            role: admin.role,
            permissions,
        })
    }

    /// 通过 ID 获取管理员信息
    pub async fn get_admin_info(&self, id: i64) -> Result<AdminInfo, RswsError> {
        let admin = self
            .admin_repo
            .get_admin_by_id(id)
            .await?
            .ok_or_else(|| RswsError::not_found("Admin not found"))?;

        let permissions: Vec<String> =
            serde_json::from_value(admin.permissions.clone()).unwrap_or_default();

        Ok(AdminInfo {
            id: admin.id,
            email: admin.email,
            username: admin.username,
            nickname: admin.nickname,
            avatar_url: admin.avatar_url,
            is_active: admin.is_active,
            role: admin.role,
            permissions,
        })
    }

    /// 创建管理员
    pub async fn create_admin(
        &self,
        email: &str,
        password: &str,
        username: &str,
        role: &str,
        creator_id: Option<i64>,
        ip_address: Option<&str>,
    ) -> Result<Admin, RswsError> {
        if self.admin_repo.email_exists(email).await? {
            return Err(RswsError::conflict("Email already exists"));
        }

        let admin = self
            .admin_repo
            .create_admin(email, password, username, role)
            .await?;

        if let Some(creator_id) = creator_id {
            let _ = self
                .admin_repo
                .log_admin_operation(
                    creator_id,
                    "create",
                    Some("admin"),
                    Some(&admin.id.to_string()),
                    Some(&format!("Created admin: {}", admin.username)),
                    ip_address,
                    None,
                )
                .await;
        }

        Ok(admin)
    }

    /// 更新管理员信息
    pub async fn update_admin(
        &self,
        request: UpdateAdminRequest,
        updater_id: i64,
        ip_address: Option<&str>,
    ) -> Result<Admin, RswsError> {
        let updated = self.admin_repo.update_admin(&request).await?;

        // 如果是禁用管理员,清除其 Redis 缓存
        if request.is_active == Some(false) {
            self.invalidate_admin_keys(updated.id).await;
        }

        // 如果改了密码,清除所有 API Key（Redis-only）
        if request.password.is_some() {
            self.invalidate_admin_keys(updated.id).await;
        }

        let _ = self
            .admin_repo
            .log_admin_operation(
                updater_id,
                "update",
                Some("admin"),
                Some(&updated.id.to_string()),
                Some(&format!("Updated admin: {}", updated.username)),
                ip_address,
                None,
            )
            .await;

        Ok(updated)
    }

    /// 获取管理员列表
    pub async fn list_admins(
        &self,
        page: i64,
        page_size: i64,
        role: Option<&str>,
    ) -> Result<(Vec<AdminInfo>, i64), RswsError> {
        let admins = self.admin_repo.get_admins(page, page_size, role).await?;
        let total = self.admin_repo.get_admins_count(role).await?;

        let admin_infos = admins
            .into_iter()
            .map(|admin| {
                let permissions: Vec<String> =
                    serde_json::from_value(admin.permissions.clone()).unwrap_or_default();
                AdminInfo {
                    id: admin.id,
                    email: admin.email,
                    username: admin.username,
                    nickname: admin.nickname,
                    avatar_url: admin.avatar_url,
                    is_active: admin.is_active,
                    role: admin.role,
                    permissions,
                }
            })
            .collect();

        Ok((admin_infos, total))
    }

    /// 停用管理员
    pub async fn deactivate_admin(
        &self,
        admin_id: i64,
        operator_id: i64,
        ip_address: Option<&str>,
    ) -> Result<(), RswsError> {
        let request = UpdateAdminRequest {
            id: admin_id,
            email: None,
            password: None,
            username: None,
            avatar_url: None,
            is_active: Some(false),
            role: None,
            permissions: None,
        };
        self.admin_repo.update_admin(&request).await?;

        // 禁用管理员所有 API Key（Redis-only: 清除 Redis 缓存即可）
        self.invalidate_admin_keys(admin_id).await;

        let _ = self
            .admin_repo
            .log_admin_operation(
                operator_id,
                "deactivate",
                Some("admin"),
                Some(&admin_id.to_string()),
                None,
                ip_address,
                None,
            )
            .await;

        Ok(())
    }

    /// 激活管理员
    pub async fn activate_admin(
        &self,
        admin_id: i64,
        operator_id: i64,
        ip_address: Option<&str>,
    ) -> Result<(), RswsError> {
        let request = UpdateAdminRequest {
            id: admin_id,
            email: None,
            password: None,
            username: None,
            avatar_url: None,
            is_active: Some(true),
            role: None,
            permissions: None,
        };
        self.admin_repo.update_admin(&request).await?;

        // 记录操作日志
        let _ = self
            .admin_repo
            .log_admin_operation(
                operator_id,
                "activate",
                Some("admin"),
                Some(&admin_id.to_string()),
                None,
                ip_address,
                None,
            )
            .await;

        Ok(())
    }

    /// 重置管理员密码
    pub async fn reset_password(
        &self,
        admin_id: i64,
        new_password: &str,
        operator_id: i64,
        ip_address: Option<&str>,
    ) -> Result<(), RswsError> {
        let request = UpdateAdminRequest {
            id: admin_id,
            email: None,
            password: Some(new_password.to_string()),
            username: None,
            avatar_url: None,
            is_active: None,
            role: None,
            permissions: None,
        };

        // update_admin 会处理：哈希密码、禁用 API Key、记录日志
        self.update_admin(request, operator_id, ip_address).await?;

        Ok(())
    }

    // ==================== Admin API Key 管理 ====================

    /// 创建管理员 API Key
    pub async fn create_api_key(
        &self,
        admin_id: i64,
        name: &str,
        permissions: Vec<String>,
        rate_limit: Option<i32>,
        expires_in_days: Option<i32>,
    ) -> Result<AdminApiKeyResponse, RswsError> {
        // 生成新的 API Key（不存 DB，只存 Redis）
        let api_key = rsws_common::utils::generate_api_key();
        let key_id = chrono::Utc::now().timestamp_millis(); // 用时间戳作为唯一 ID
        let expires_at =
            expires_in_days.map(|days| chrono::Utc::now() + chrono::Duration::days(days as i64));

        // 写入 Redis（覆盖旧 Key）
        if let Some(ref redis) = self.redis {
            let cached = CachedAdminApiKey {
                admin_id,
                key_id,
                api_key: api_key.clone(),
                role: String::new(),
                permissions: permissions.clone(),
                rate_limit: Some(rate_limit.unwrap_or(1000)),
                expires_at,
            };
            let ttl = expires_in_days
                .map(|d| d as u64 * 86400)
                .unwrap_or(Self::DEFAULT_SESSION_TTL);
            let _ = redis
                .set_json(&Self::redis_key(admin_id), &cached, ttl)
                .await;
        } else {
            return Err(RswsError::internal("Redis not configured"));
        }

        // 构造响应（id 是临时生成的，不存 DB）
        Ok(AdminApiKeyResponse {
            id: key_id,
            name: name.to_string(),
            api_key,
            permissions,
            rate_limit: Some(rate_limit.unwrap_or(1000)),
            last_used_at: None,
            expires_at,
            is_active: true,
            created_at: chrono::Utc::now(),
        })
    }

    /// 获取管理员的当前 API Key（从 Redis 读取）
    pub async fn list_api_keys(&self, admin_id: i64) -> Result<Vec<AdminApiKey>, RswsError> {
        if let Some(ref redis) = self.redis {
            if let Some(cached) = redis
                .get_json::<CachedAdminApiKey>(&Self::redis_key(admin_id))
                .await?
            {
                let key = AdminApiKey {
                    id: cached.key_id,
                    admin_id: cached.admin_id,
                    name: "current".to_string(),
                    api_key: cached.api_key,
                    permissions: serde_json::json!(cached.permissions),
                    rate_limit: cached.rate_limit,
                    last_used_at: None,
                    expires_at: cached.expires_at,
                    is_active: true,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                };
                return Ok(vec![key]);
            }
        }
        Ok(vec![]) // Redis 中无 Key，返回空列表
    }

    /// 删除管理员的 API Key（从 Redis 删除）
    pub async fn delete_api_key(&self, _key_id: i64, admin_id: i64) -> Result<bool, RswsError> {
        if let Some(ref redis) = self.redis {
            let result = redis.del(&Self::redis_key(admin_id)).await;
            return Ok(result.is_ok());
        }
        Ok(false)
    }

    /// 切换管理员 API Key 状态（Redis 中暂不支持，返回成功）
    pub async fn toggle_api_key_status(
        &self,
        _key_id: i64,
        admin_id: i64,
        is_active: bool,
    ) -> Result<(), RswsError> {
        // Redis 方案中不存储历史 Key，当前 Key 总是活跃的
        // 如果需要禁用，直接删除 Redis 中的 Key
        if !is_active {
            self.invalidate_admin_keys(admin_id).await;
        }
        Ok(())
    }

    /// 验证管理员 API Key 是否存在且活跃（从 Redis 查找）
    pub async fn validate_admin_api_key(
        &self,
        api_key: &str,
    ) -> Result<Option<(AdminApiKey, Admin)>, RswsError> {
        if let Some(ref redis) = self.redis {
            // 扫描所有 admin_apikey:* 键
            let pattern = "admin_apikey:*".to_string();
            if let Ok(keys) = redis.scan_keys(&pattern, 100).await {
                for key in keys {
                    if let Some(cached) = redis.get_json::<CachedAdminApiKey>(&key).await? {
                        if cached.api_key == api_key {
                            // 找到匹配的 Key，获取 admin 信息
                            let admin = self
                                .admin_repo
                                .get_admin_by_id(cached.admin_id)
                                .await?
                                .ok_or_else(|| RswsError::not_found("Admin not found"))?;

                            let ak = AdminApiKey {
                                id: cached.key_id,
                                admin_id: cached.admin_id,
                                name: "current".to_string(),
                                api_key: cached.api_key,
                                permissions: serde_json::json!(cached.permissions),
                                rate_limit: cached.rate_limit,
                                last_used_at: None,
                                expires_at: cached.expires_at,
                                is_active: true,
                                created_at: chrono::Utc::now(),
                                updated_at: chrono::Utc::now(),
                            };
                            return Ok(Some((ak, admin)));
                        }
                    }
                }
            }
        }
        Ok(None) // 未找到
    }

    /// 获取管理员 API Key（仅从 Redis 读取）
    ///
    /// 返回 (api_key, key_id) 元组，用于签名验证
    /// 如果 Redis 中不存在，返回错误（不回退 DB）
    async fn get_cached_admin_key(
        &self,
        admin_id: i64,
    ) -> Result<(Option<String>, Option<i64>), RswsError> {
        // 只从 Redis 读取
        if let Some(ref redis) = self.redis {
            if let Some(cached) = redis
                .get_json::<CachedAdminApiKey>(&Self::redis_key(admin_id))
                .await?
            {
                // 检查是否过期
                if let Some(expires) = cached.expires_at {
                    if expires < chrono::Utc::now() {
                        let _ = redis.del(&Self::redis_key(admin_id)).await;
                        return Err(RswsError::unauthorized("API Key expired"));
                    }
                }
                return Ok((Some(cached.api_key), Some(cached.key_id)));
            } else {
                return Err(RswsError::unauthorized(
                    "API Key not found in Redis. Please login again.",
                ));
            }
        }

        Err(RswsError::internal("Redis not configured"))
    }

    /// 清除管理员所有 Redis 缓存的 API Key
    async fn invalidate_admin_keys(&self, admin_id: i64) {
        if let Some(ref redis) = self.redis {
            // 按 admin_id 索引，直接删除一个 key
            let _ = redis.del(&Self::redis_key(admin_id)).await;
        }
    }

    /// 验证管理员签名认证（单密钥 Cregis 方案）
    ///
    /// 前端传 admin_id (=user_id) + timestamp + nonce + sign
    /// 后端通过 admin_id 查找 api_key，用 api_key 重算签名验签
    /// 验证管理员签名（Cregis 方案：通过 admin_id 查找 api_key 验签）
    ///
    /// 返回 Ok(Some(api_key_id)) 验签通过
    /// 返回 Ok(None) 验签失败（无活跃 key 或签名不匹配）
    pub async fn validate_signature_by_admin_id(
        &self,
        admin_id: i64,
        params: &HashMap<String, String>,
        sign: &str,
    ) -> Result<Option<i64>, RswsError> {
        // 1) 获取 api_key（通过 admin_id 走缓存+DB）
        let (api_key, key_id) = self.get_cached_admin_key(admin_id).await?;
        let (api_key, key_id) = match (api_key, key_id) {
            (Some(k), Some(id)) => (k, id),
            _ => return Ok(None),
        };

        // 2) 重算签名（Cregis: api_key 拼在排序参数前面）
        let computed_sign = rsws_common::signature::compute_cregis_signature(params, &api_key);

        // 3) 对比签名
        if computed_sign == sign {
            // Redis-only: 不再更新 DB last_used
            Ok(Some(key_id))
        } else {
            tracing::warn!(
                "Admin signature mismatch for admin_id: {}. Expected: {}, Got: {}",
                admin_id,
                computed_sign,
                sign
            );
            Ok(None)
        }
    }
}
