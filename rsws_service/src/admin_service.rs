//! 管理员服务
//!
//! 管理员 CRUD + Admin API Key 管理
//! 认证统一走 API Key,不使用 JWT
//! Admin API Key 凭证缓存到 Redis(快速验证 + 强制下线)

use std::collections::HashMap;
use std::sync::Arc;
use rsws_common::error::RswsError;
use rsws_common::ErrorCode;
use rsws_db::admin::AdminRepository;
use rsws_db::RedisService;
use rsws_model::user_models::admin::*;
use serde::{Serialize, Deserialize};

/// Redis 中缓存的管理员 API Key 会话信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedAdminApiKey {
    pub admin_id: i64,
    pub key_id: i64,
    pub api_secret: String,
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

    /// Redis key 格式
    fn redis_key(api_key: &str) -> String {
        format!("admin_apikey:{}", api_key)
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
        let admin = self.admin_repo
            .verify_admin_credentials(email, password)
            .await?
            .ok_or_else(|| RswsError::unauthorized("Invalid email or password"))?;

        if !admin.is_active {
            return Err(RswsError::forbidden("Account is disabled"));
        }

        // 更新登录信息
        self.admin_repo.update_admin_login(admin.id, ip_address).await?;

        // 记录操作日志
        let _ = self.admin_repo.log_admin_operation(
            admin.id,
            "login",
            None,
            None,
            None,
            ip_address,
            None,
        ).await;

        let permissions: Vec<String> = serde_json::from_value(admin.permissions.clone())
            .unwrap_or_default();

        Ok(AdminInfo {
            id: admin.id,
            email: admin.email,
            username: admin.username,
            avatar_url: admin.avatar_url,
            role: admin.role,
            permissions,
        })
    }

    /// 通过 ID 获取管理员信息
    pub async fn get_admin_info(&self, id: i64) -> Result<AdminInfo, RswsError> {
        let admin = self.admin_repo.get_admin_by_id(id).await?
            .ok_or_else(|| RswsError::not_found("Admin not found"))?;

        let permissions: Vec<String> = serde_json::from_value(admin.permissions.clone())
            .unwrap_or_default();

        Ok(AdminInfo {
            id: admin.id,
            email: admin.email,
            username: admin.username,
            avatar_url: admin.avatar_url,
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

        let admin = self.admin_repo.create_admin(email, password, username, role).await?;

        if let Some(creator_id) = creator_id {
            let _ = self.admin_repo.log_admin_operation(
                creator_id,
                "create",
                Some("admin"),
                Some(&admin.id.to_string()),
                Some(&format!("Created admin: {}", admin.username)),
                ip_address,
                None,
            ).await;
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

        // 如果改了密码,清除所有 API Key 缓存
        if request.password.is_some() {
            self.invalidate_admin_keys(updated.id).await;
            // 同时禁用其所有 API Key
            self.admin_repo.deactivate_admin_api_keys(updated.id).await?;
        }

        let _ = self.admin_repo.log_admin_operation(
            updater_id,
            "update",
            Some("admin"),
            Some(&updated.id.to_string()),
            Some(&format!("Updated admin: {}", updated.username)),
            ip_address,
            None,
        ).await;

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

        let admin_infos = admins.into_iter().map(|admin| {
            let permissions: Vec<String> = serde_json::from_value(admin.permissions.clone())
                .unwrap_or_default();
            AdminInfo {
                id: admin.id,
                email: admin.email,
                username: admin.username,
                avatar_url: admin.avatar_url,
                role: admin.role,
                permissions,
            }
        }).collect();

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

        // 禁用管理员所有 API Key
        self.admin_repo.deactivate_admin_api_keys(admin_id).await?;

        // 清除 Redis 缓存
        self.invalidate_admin_keys(admin_id).await;

        let _ = self.admin_repo.log_admin_operation(
            operator_id,
            "deactivate",
            Some("admin"),
            Some(&admin_id.to_string()),
            None,
            ip_address,
            None,
        ).await;

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
        let _ = self.admin_repo.log_admin_operation(
            operator_id,
            "activate",
            Some("admin"),
            Some(&admin_id.to_string()),
            None,
            ip_address,
            None,
        ).await;

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
        let (record, secret) = self.admin_repo
            .create_admin_api_key(admin_id, name, permissions, rate_limit, expires_in_days)
            .await?;

        // 创建后写入 Redis 缓存
        if let Some(ref redis) = self.redis {
            let perms: Vec<String> = serde_json::from_value(record.permissions.clone())
                .unwrap_or_default();
            let cached = CachedAdminApiKey {
                admin_id: record.admin_id,
                key_id: record.id,
                api_secret: secret.clone(),
                role: String::new(), // role 会在 validate 时从 DB 补充
                permissions: perms,
                rate_limit: record.rate_limit,
                expires_at: record.expires_at,
            };
            let _ = redis.set_json(&Self::redis_key(&record.api_key), &cached, Self::DEFAULT_SESSION_TTL).await;
        }

        let perms: Vec<String> = serde_json::from_value(record.permissions.clone())
            .unwrap_or_default();

        Ok(AdminApiKeyResponse {
            id: record.id,
            name: record.name,
            api_key: record.api_key,
            api_secret: Some(secret),
            permissions: perms,
            rate_limit: record.rate_limit,
            last_used_at: record.last_used_at,
            expires_at: record.expires_at,
            is_active: record.is_active,
            created_at: record.created_at,
        })
    }

    /// 获取管理员的 API Key 列表
    pub async fn list_api_keys(&self, admin_id: i64) -> Result<Vec<AdminApiKey>, RswsError> {
        self.admin_repo.get_admin_api_keys(admin_id).await
    }

    /// 删除管理员 API Key
    pub async fn delete_api_key(&self, key_id: i64, admin_id: i64) -> Result<bool, RswsError> {
        let deleted = self.admin_repo.delete_admin_api_key(key_id, admin_id).await?;

        // 删除后清除 Redis(需要知道 api_key 值,这里简化处理)
        // 更好的做法是 delete 返回被删记录

        Ok(deleted)
    }

    /// 切换管理员 API Key 状态
    pub async fn toggle_api_key_status(
        &self,
        key_id: i64,
        admin_id: i64,
        is_active: bool,
    ) -> Result<(), RswsError> {
        let updated = self.admin_repo.update_api_key_status(key_id, admin_id, is_active).await?;
        
        if !updated {
            return Err(RswsError::from(ErrorCode::NOT_FOUND));
        }

        // 如果禁用，清除 Redis 缓存
        if !is_active {
            // TODO: 先查询 api_key 值，再删除 Redis 缓存
            // 这里简化处理，依赖缓存 TTL 过期
        }

        Ok(())
    }

    /// 验证管理员 API Key(供中间件调用,带 Redis 缓存)
    pub async fn validate_admin_api_key(
        &self,
        api_key: &str,
        api_secret: &str,
    ) -> Result<Option<(AdminApiKey, Admin)>, RswsError> {
        // 1) 先查 Redis
        if let Some(ref redis) = self.redis {
            if let Some(cached) = redis.get_json::<CachedAdminApiKey>(&Self::redis_key(api_key)).await? {
                // Redis 命中:验证 secret
                if cached.api_secret == api_secret {
                    // 检查是否过期
                    if let Some(expires) = cached.expires_at {
                        if expires < chrono::Utc::now() {
                            let _ = redis.del(&Self::redis_key(api_key)).await;
                            return Ok(None);
                        }
                    }

                    // 需要补查 admin 信息
                    let admin = self.admin_repo.get_admin_by_id(cached.admin_id).await?;
                    if let Some(admin) = admin {
                        if !admin.is_active {
                            let _ = redis.del(&Self::redis_key(api_key)).await;
                            return Ok(None);
                        }

                        // 构造 AdminApiKey(简化)
                        let key_record = AdminApiKey {
                            id: cached.key_id,
                            admin_id: cached.admin_id,
                            name: String::new(),
                            api_key: api_key.to_string(),
                            api_secret_encrypted: String::new(), // Redis 中不需要
                            permissions: serde_json::to_value(&cached.permissions).unwrap_or_default(),
                            rate_limit: cached.rate_limit,
                            last_used_at: None,
                            expires_at: cached.expires_at,
                            is_active: true,
                            created_at: chrono::Utc::now(),
                            updated_at: chrono::Utc::now(),
                        };

                        return Ok(Some((key_record, admin)));
                    }
                }
                // secret 不匹配
                let _ = redis.del(&Self::redis_key(api_key)).await;
            }
        }

        // 2) Redis miss → 查 DB
        let result = self.admin_repo.validate_admin_api_key(api_key, api_secret).await?;

        // 3) DB 验证通过 → 写入 Redis
        if let Some((ref key_record, ref admin)) = result {
            if let Some(ref redis) = self.redis {
                let permissions: Vec<String> = serde_json::from_value(key_record.permissions.clone())
                    .unwrap_or_default();
                let cached = CachedAdminApiKey {
                    admin_id: admin.id,
                    key_id: key_record.id,
                    api_secret: api_secret.to_string(),
                    role: admin.role.clone(),
                    permissions,
                    rate_limit: key_record.rate_limit,
                    expires_at: key_record.expires_at,
                };
                let _ = redis.set_json(&Self::redis_key(api_key), &cached, Self::DEFAULT_SESSION_TTL).await;
            }
        }

        Ok(result)
    }

    /// 清除管理员所有 Redis 缓存的 API Key
    async fn invalidate_admin_keys(&self, admin_id: i64) {
        if let Some(ref redis) = self.redis {
            // 获取管理员所有 API Key,逐个删 Redis
            match self.admin_repo.get_admin_api_keys(admin_id).await {
                Ok(keys) => {
                    for key in keys {
                        let _ = redis.del(&Self::redis_key(&key.api_key)).await;
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to get admin API keys for cache invalidation: {}", e);
                }
            }
        }
    }

    /// 验证管理员签名认证（符合 Cregis 方案）
    pub async fn validate_admin_api_key_signature(
        &self,
        api_key: &str,
        params: &HashMap<String, String>,
        sign: &str,
    ) -> Result<Option<(AdminApiKey, Admin)>, RswsError> {
        // 1) 获取 api_secret（先查 Redis 缓存）
        let api_secret = if let Some(ref redis) = self.redis {
            if let Some(cached) = redis.get_json::<CachedAdminApiKey>(&Self::redis_key(api_key)).await? {
                Some(cached.api_secret)
            } else {
                None
            }
        } else {
            None
        };

        let api_secret = match api_secret {
            Some(s) => s,
            None => {
                // Redis miss，从 DB 获取
                let record = self.admin_repo.get_admin_api_key_by_key(api_key).await?;
                match record {
                    Some(r) => {
                        // 写入 Redis 缓存
                        if let Some(ref redis) = self.redis {
                            let permissions: Vec<String> = serde_json::from_value(r.permissions.clone())
                                .unwrap_or_default();
                            let cached = CachedAdminApiKey {
                                admin_id: r.admin_id,
                                key_id: r.id,
                                api_secret: r.api_secret_encrypted.clone(),
                                role: String::new(),
                                permissions,
                                rate_limit: r.rate_limit,
                                expires_at: r.expires_at,
                            };
                            let _ = redis.set_json(&Self::redis_key(api_key), &cached, Self::DEFAULT_SESSION_TTL).await;
                        }
                        r.api_secret_encrypted
                    }
                    None => return Ok(None),
                }
            }
        };

        // 2) 重算签名
        let computed_sign = compute_signature(params, &api_secret);

        // 3) 对比签名
        if computed_sign == sign {
            // 签名正确，获取完整的记录
            let result = self.admin_repo.validate_admin_api_key(api_key, &api_secret).await?;
            Ok(result)
        } else {
            tracing::warn!(
                "Admin signature mismatch for api_key: {}. Expected: {}, Got: {}",
                api_key, computed_sign, sign
            );
            Ok(None)
        }
    }
}

/// 计算签名（符合 Cregis 方案）
fn compute_signature(params: &HashMap<String, String>, api_secret: &str) -> String {
    // 1. 获取所有 key（排除 sign），排序
    let mut keys: Vec<&String> = params.keys()
        .filter(|k| (*k).as_str() != "sign")
        .collect();
    keys.sort();

    // 2. 按 ASCII 顺序拼接 key + value
    let param_str: String = keys.iter()
        .map(|k| format!("{}{}", k, params[*k]))
        .collect();

    // 3. 拼接 api_secret（拼在前面，与 Cregis 方案一致）
    let sign_str = format!("{}{}", api_secret, param_str);

    // 4. MD5 + 小写 hex
    format!("{:x}", md5::compute(sign_str.as_bytes()))
}
