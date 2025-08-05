use rsws_db::admin::AdminRepository;
use rsws_model::user::admin::*;
use rsws_model::user::role::*;
use rsws_common::error::ServiceError;
use chrono::{Duration, Utc};
use bcrypt::verify;
use jsonwebtoken::{encode, Header, EncodingKey, Algorithm};
use std::net::IpAddr;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct AdminClaims {
    sub: String,        // 主题 (admin ID)
    exp: usize,         // 过期时间
    iat: usize,         // 签发时间
    role: String,       // 角色
    permissions: Vec<String>, // 权限
}

pub struct AdminService {
    admin_repo: AdminRepository,
    jwt_secret: String,
    jwt_expiry: i64,    // 过期时间（分钟）
}

impl AdminService {
    pub fn new(admin_repo: AdminRepository, jwt_secret: String, jwt_expiry: i64) -> Self {
        Self {
            admin_repo,
            jwt_secret,
            jwt_expiry,
        }
    }
    
    // 创建管理员
    pub async fn create_admin(
        &self,
        request: CreateAdminRequest,
        creator_id: Option<i64>,
        ip: Option<IpAddr>,
        user_agent: Option<&str>,
    ) -> Result<Admin, ServiceError> {
        // 检查邮箱是否已存在
        if self.admin_repo.email_exists(&request.email).await? {
            return Err(ServiceError::AlreadyExists("Email already exists".to_string()));
        }
        
        // 创建管理员
        let admin = self.admin_repo.create_admin(&request).await?;
        
        // 记录操作日志
        if let Some(creator_id) = creator_id {
            let _ = self.admin_repo.log_admin_operation(
                creator_id,
                "create",
                Some("admin"),
                Some(&admin.id.to_string()),
                Some(&format!("Created admin: {}", admin.username)),
                ip.map(|ip| ip.to_string()).as_deref(),
                user_agent,
            ).await;
        }
        
        Ok(admin)
    }
    
    // 管理员登录
    // 移除 bcrypt::verify 导入，使用 PasswordService
    use rsws_common::password::PasswordService;
    
    impl AdminService {
        // 更新管理员登录逻辑
        pub async fn login(
            &self,
            request: AdminLoginRequest,
            ip_address: Option<IpAddr>,
        ) -> Result<AdminLoginResponse, ServiceError> {
            // 验证管理员凭据（使用Argon2）
            let admin = self.admin_repo
                .verify_admin_credentials(&request.email, &request.password)
                .await?
                .ok_or_else(|| ServiceError::AuthError("邮箱或密码错误".to_string()))?;
            
            // 获取管理员
            let admin = self.admin_repo.get_admin_by_email(&request.email).await?
                .ok_or_else(|| ServiceError::NotFound("Admin not found".to_string()))?;
            
            // 检查账号是否激活
            if !admin.is_active {
                return Err(ServiceError::Unauthorized("Account is disabled".to_string()));
            }
            
            // 更新登录信息
            self.admin_repo.update_admin_login(
                admin.id,
                ip.map(|ip| ip.to_string()).as_deref(),
            ).await?;
            
            // 记录登录日志
            let _ = self.admin_repo.log_admin_operation(
                admin.id,
                "login",
                None,
                None,
                None,
                ip.map(|ip| ip.to_string()).as_deref(),
                user_agent,
            ).await;
            
            // 生成JWT
            let now = Utc::now();
            let expires_at = now + Duration::minutes(self.jwt_expiry);
            let claims = AdminClaims {
                sub: admin.id.to_string(),
                exp: expires_at.timestamp() as usize,
                iat: now.timestamp() as usize,
                role: admin.role.clone(),
                permissions: admin.permissions.clone(),
            };
            
            let token = encode(
                &Header::new(Algorithm::HS256),
                &claims,
                &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
            ).map_err(|e| ServiceError::InternalError(format!("Failed to generate token: {}", e)))?;
            
            // 返回登录响应
            Ok(AdminLoginResponse {
                admin: AdminInfo {
                    id: admin.id,
                    email: admin.email,
                    username: admin.username,
                    avatar_url: admin.avatar_url,
                    role: admin.role,
                    permissions: admin.permissions,
                },
                token,
                expires_at,
            })
        }
        
        // 获取管理员信息
        pub async fn get_admin_info(&self, id: i64) -> Result<AdminInfo, ServiceError> {
            let admin = self.admin_repo.get_admin_by_id(id).await?
                .ok_or_else(|| ServiceError::NotFound("Admin not found".to_string()))?;
            
            Ok(AdminInfo {
                id: admin.id,
                email: admin.email,
                username: admin.username,
                avatar_url: admin.avatar_url,
                role: admin.role,
                permissions: admin.permissions,
            })
        }
        
        // 更新管理员信息
        pub async fn update_admin(
            &self,
            request: UpdateAdminRequest,
            updater_id: i64,
            ip: Option<IpAddr>,
            user_agent: Option<&str>,
        ) -> Result<Admin, ServiceError> {
            // 检查管理员是否存在
            let admin = self.admin_repo.get_admin_by_id(request.id).await?
                .ok_or_else(|| ServiceError::NotFound("Admin not found".to_string()))?;
            
            // 如果更新邮箱，检查邮箱是否已存在
            if let Some(email) = &request.email {
                if email != &admin.email && self.admin_repo.email_exists(email).await? {
                    return Err(ServiceError::AlreadyExists("Email already exists".to_string()));
                }
            }
            
            // 更新管理员
            let updated_admin = self.admin_repo.update_admin(&request).await?;
            
            // 记录操作日志
            let _ = self.admin_repo.log_admin_operation(
                updater_id,
                "update",
                Some("admin"),
                Some(&updated_admin.id.to_string()),
                Some(&format!("Updated admin: {}", updated_admin.username)),
                ip.map(|ip| ip.to_string()).as_deref(),
                user_agent,
            ).await;
            
            Ok(updated_admin)
        }
        
        // 获取管理员列表
        pub async fn get_admins(
            &self,
            page: i64,
            page_size: i64,
            role: Option<&str>,
        ) -> Result<(Vec<AdminInfo>, i64), ServiceError> {
            let admins = self.admin_repo.get_admins(page, page_size, role).await?;
            let total = self.admin_repo.get_admins_count(role).await?;
            
            let admin_infos = admins.into_iter().map(|admin| {
                AdminInfo {
                    id: admin.id,
                    email: admin.email,
                    username: admin.username,
                    avatar_url: admin.avatar_url,
                    role: admin.role,
                    permissions: admin.permissions,
                }
            }).collect();
            
            Ok((admin_infos, total))
        }
        
        // 创建管理员API Key
        pub async fn create_admin_api_key(
            &self,
            admin_id: i32,
            request: CreateAdminApiKeyRequest,
        ) -> Result<AdminApiKeyResponse, ServiceError> {
            let api_key = format!("ak_{}", generate_random_string(32));
            let api_secret = generate_random_string(64);
            let api_secret_encrypted = self.encryption.encrypt(&api_secret)?;
            
            let expires_at = request.expires_in_days
                .map(|days| Utc::now() + Duration::days(days as i64));
            
            let permissions_json = serde_json::to_value(&request.permissions)?;
            
            let id = sqlx::query_scalar::<_, i32>(
                r#"
                INSERT INTO admin_api_keys (admin_id, name, api_key, api_secret_encrypted, permissions, rate_limit, expires_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                RETURNING id
                "#
            )
            .bind(admin_id)
            .bind(&request.name)
            .bind(&api_key)
            .bind(&api_secret_encrypted)
            .bind(&permissions_json)
            .bind(request.rate_limit)
            .bind(expires_at)
            .execute(&self.db_pool)
            .await?;
            
            Ok(AdminApiKeyResponse {
                id,
                name: request.name,
                api_key,
                api_secret: Some(api_secret), // 只在创建时返回
                permissions: request.permissions,
                rate_limit: request.rate_limit,
                last_used_at: None,
                expires_at,
                is_active: true,
                created_at: Utc::now(),
            })
        }
        
        // 验证管理员API Key和签名
        pub async fn validate_admin_api_request(
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
            // 检查时间戳
            let current_timestamp = Utc::now().timestamp();
            let time_diff = (current_timestamp - timestamp).abs();
            if time_diff > 300 { // 5分钟有效期
                return Ok(SignatureValidation {
                    is_valid: false,
                    admin_session: None,
                    error_message: Some("Request expired".to_string()),
                });
            }
            
            // 获取API Key信息
            let api_key_info = sqlx::query_as::<_, AdminApiKey>(
                "SELECT * FROM admin_api_keys WHERE api_key = $1 AND is_active = true"
            )
            .bind(api_key)
            .fetch_optional(&self.db_pool)
            .await?;
            
            let api_key_info = match api_key_info {
                Some(info) => {
                    // 检查过期时间
                    if let Some(expires_at) = info.expires_at {
                        if Utc::now() > expires_at {
                            return Ok(SignatureValidation {
                                is_valid: false,
                                admin_session: None,
                                error_message: Some("API key expired".to_string()),
                            });
                        }
                    }
                    info
                }
                None => {
                    return Ok(SignatureValidation {
                        is_valid: false,
                        admin_session: None,
                        error_message: Some("Invalid API key".to_string()),
                    });
                }
            };
            
            // 解密API Secret
            let api_secret = self.encryption.decrypt(&api_key_info.api_secret_encrypted)?;
            
            // 验证签名
            let is_valid = SignatureService::verify_signature(
                &api_secret,
                method,
                path,
                timestamp,
                nonce,
                signature,
                body,
                query_params,
            ).map_err(|e| ServiceError::ValidationError(e))?;
            
            if is_valid {
                // 更新最后使用时间
                let _ = sqlx::query(
                    "UPDATE admin_api_keys SET last_used_at = NOW() WHERE id = $1"
                )
                .bind(api_key_info.id)
                .execute(&self.db_pool)
                .await;
                
                Ok(SignatureValidation {
                    is_valid: true,
                    admin_session: Some(AdminSession {
                        admin_id: api_key_info.admin_id,
                        api_key: api_key_info.api_key,
                        permissions: api_key_info.permissions,
                    }),
                    error_message: None,
                })
            } else {
                Ok(SignatureValidation {
                    is_valid: false,
                    admin_session: None,
                    error_message: Some("Invalid signature".to_string()),
                })
            }
        }
    }
}