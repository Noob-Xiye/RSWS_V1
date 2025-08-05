use rsws_db::UserRepository;
use rsws_model::user::*;
use rsws_common::email::EmailService;
use rsws_common::error::ServiceError;
use chrono::{Duration, Utc};
use rand::Rng;
use sha2::{Digest, Sha256};
use base64::{Engine as _, engine::general_purpose};
use std::net::IpAddr;
use rsws_common::snowflake;

pub struct UserService {
    user_repo: UserRepository,
    email_service: EmailService,
}

impl UserService {
    pub fn new(user_repo: UserRepository, email_service: EmailService) -> Self {
        Self {
            user_repo,
            email_service,
        }
    }
    
    // 发送注册验证码
    pub async fn send_registration_code(
        &self,
        request: SendVerificationCodeRequest,
    ) -> Result<SendCodeResponse, ServiceError> {
        // 检查邮箱是否已注册
        if self.user_repo.email_exists(&request.email).await? {
            return Ok(SendCodeResponse {
                success: false,
                message: "该邮箱已被注册".to_string(),
                expires_in: 0,
            });
        }
        
        // 创建验证码
        let verification_code = self.user_repo
            .create_verification_code(&request.email, &request.code_type)
            .await?;
            
        // 发送邮件
        self.email_service
            .send_verification_code(
                &request.email,
                &verification_code.code,
                &request.code_type,
            )
            .await
            .map_err(|e| ServiceError::EmailError(e.to_string()))?;
            
        Ok(SendCodeResponse {
            success: true,
            message: "验证码已发送到您的邮箱".to_string(),
            expires_in: 600, // 10分钟
        })
    }
    
    // 验证码注册
    pub async fn register_with_code(
        &self,
        request: VerifyCodeRequest,
    ) -> Result<RegisterResponse, ServiceError> {
        // 验证验证码
        let is_valid = self.user_repo
            .verify_code(&request.email, &request.code, "registration")
            .await?;
            
        if !is_valid {
            return Ok(RegisterResponse {
                success: false,
                message: "验证码无效或已过期".to_string(),
                user_id: None,
            });
        }
        
        // 再次检查邮箱和昵称是否已存在
        if self.user_repo.email_exists(&request.email).await? {
            return Ok(RegisterResponse {
                success: false,
                message: "该邮箱已被注册".to_string(),
                user_id: None,
            });
        }
        
        if self.user_repo.nickname_exists(&request.nickname).await? {
            return Ok(RegisterResponse {
                success: false,
                message: "该昵称已被使用".to_string(),
                user_id: None,
            });
        }
        
        // 创建用户
        let user = self.user_repo
            .create_user(&request.nickname, &request.email, &request.password)
            .await?;
            
        Ok(RegisterResponse {
            success: true,
            message: "注册成功".to_string(),
            user_id: Some(user.id),
        })
    }
    
    // 发送登录验证码
    pub async fn send_login_code(
        &self,
        request: SendLoginCodeRequest,
    ) -> Result<SendCodeResponse, ServiceError> {
        // 验证用户凭据
        let user = self.user_repo
            .verify_user_credentials(&request.email, &request.password)
            .await?
            .ok_or_else(|| ServiceError::AuthError("邮箱或密码错误".to_string()))?;
            
        if !user.is_email_verified {
            return Ok(SendCodeResponse {
                success: false,
                message: "邮箱未验证，请先完成邮箱验证".to_string(),
                expires_in: 0,
            });
        }
        
        // 创建登录验证码
        let verification_code = self.user_repo
            .create_verification_code(&request.email, "login")
            .await?;
            
        // 发送邮件
        self.email_service
            .send_verification_code(
                &request.email,
                &verification_code.code,
                "login",
            )
            .await
            .map_err(|e| ServiceError::EmailError(e.to_string()))?;
            
        Ok(SendCodeResponse {
            success: true,
            message: "登录验证码已发送到您的邮箱".to_string(),
            expires_in: 600, // 10分钟
        })
    }
    
    // 验证码登录
    pub async fn login_with_code(
        &self,
        request: VerifyLoginCodeRequest,
        ip_address: Option<IpAddr>,
    ) -> Result<LoginResponse, ServiceError> {
        // 验证用户凭据
        let user = self.user_repo
            .verify_user_credentials(&request.email, &request.password)
            .await?
            .ok_or_else(|| ServiceError::AuthError("邮箱或密码错误".to_string()))?;
            
        // 验证验证码
        let is_valid = self.user_repo
            .verify_code(&request.email, &request.code, "login")
            .await?;
            
        if !is_valid {
            return Ok(LoginResponse {
                success: false,
                message: "验证码无效或已过期".to_string(),
                user_info: None,
                session_data: None,
            });
        }
        
        // 清理过期会话
        self.user_repo.cleanup_expired_sessions(user.id).await?;
        
        // 检查活跃会话数量（限制最多5个同时登录）
        let active_sessions = self.user_repo.get_active_session_count(user.id).await?;
        if active_sessions >= 5 {
            return Ok(LoginResponse {
                success: false,
                message: "登录会话过多，请先退出其他设备".to_string(),
                user_info: None,
                session_data: None,
            });
        }
        
        // 生成会话数据
        let session_token = self.generate_session_token();
        let (api_key, api_secret) = self.generate_api_credentials();
        let expires_at = Utc::now() + Duration::days(7); // 7天过期
        
        // 创建会话
        let _session_id = self.user_repo
            .create_user_session(
                user.id,
                &session_token,
                &api_key,
                &api_secret,
                request.device_info,
                ip_address,
                request.user_agent,
                expires_at,
            )
            .await?;
            
        // 构建响应
        let user_info = UserInfo {
            id: user.id,
            nickname: user.nickname,
            email: user.email,
            avatar: user.avatar,
            is_email_verified: user.is_email_verified,
        };
        
        let session_data = SessionData {
            session_token,
            api_key,
            api_secret,
            expires_at,
            signature_info: SignatureInfo {
                algorithm: "HMAC-SHA256".to_string(),
                timestamp_header: "X-Timestamp".to_string(),
                signature_header: "X-Signature".to_string(),
                api_key_header: "X-API-Key".to_string(),
            },
        };
        
        Ok(LoginResponse {
            success: true,
            message: "登录成功".to_string(),
            user_info: Some(user_info),
            session_data: Some(session_data),
        })
    }
    
    // 生成会话令牌
    fn generate_session_token(&self) -> String {
        let mut rng = rand::thread_rng();
        let random_bytes: [u8; 32] = rng.gen();
        let timestamp = Utc::now().timestamp();
        
        let mut hasher = Sha256::new();
        hasher.update(&random_bytes);
        hasher.update(timestamp.to_be_bytes());
        let hash = hasher.finalize();
        
        general_purpose::URL_SAFE_NO_PAD.encode(hash)
    }
    
    // 生成API凭据
    fn generate_api_credentials(&self) -> (String, String) {
        let mut rng = rand::thread_rng();
        
        // 生成API Key (32字节)
        let api_key_bytes: [u8; 32] = rng.gen();
        let api_key = general_purpose::URL_SAFE_NO_PAD.encode(api_key_bytes);
        
        // 生成API Secret (64字节)
        let api_secret_bytes: [u8; 64] = rng.gen();
        let api_secret = general_purpose::URL_SAFE_NO_PAD.encode(api_secret_bytes);
        
        (api_key, api_secret)
    }
    
    // 在 UserService 中添加以下方法
    
    // 检查用户资料完整性
    pub async fn check_profile_completion(
        &self,
        user_id: i64,
    ) -> Result<ProfileCompletionResponse, ServiceError> {
        // 获取用户信息
        let user = self.user_repo.get_user_by_id(user_id).await?
            .ok_or_else(|| ServiceError::NotFound("用户不存在".to_string()))?;
        
        let mut missing_fields = Vec::new();
        let mut suggestions = Vec::new();
        
        // 检查昵称
        if user.nickname.trim().is_empty() {
            missing_fields.push("昵称".to_string());
            suggestions.push("请设置您的昵称".to_string());
        }
        
        // 检查头像
        if user.avatar.is_none() {
            missing_fields.push("头像".to_string());
            suggestions.push("上传一个头像可以让您的个人资料更加完整".to_string());
        }
        
        // 检查邮箱验证
        if !user.is_email_verified {
            missing_fields.push("邮箱验证".to_string());
            suggestions.push("请验证您的邮箱以确保账号安全".to_string());
        }
        
        // 计算完整度百分比
        let total_fields = 3; // 昵称、头像、邮箱验证
        let completed_fields = total_fields - missing_fields.len();
        let completion_percentage = (completed_fields as f32 / total_fields as f32) * 100.0;
        
        Ok(ProfileCompletionResponse {
            completion_percentage,
            missing_fields,
            suggestions,
        })
    }
    
    // 获取用户资料
    pub async fn get_user_profile(&self, user_id: i64) -> Result<UserProfile, ServiceError> {
        let user = self.user_repo.get_user_by_id(user_id).await?
            .ok_or_else(|| ServiceError::NotFound("User not found".to_string()))?;
            
        Ok(UserProfile {
            id: user.id,
            nickname: user.nickname,
            email: user.email,
            avatar: user.avatar,
            is_email_verified: user.is_email_verified,
            created_at: user.created_at,
        })
    }
    
    // 更新用户资料
    pub async fn update_user_profile(
        &self,
        user_id: i64,
        request: UpdateProfileRequest,
    ) -> Result<(), ServiceError> {
        // 检查昵称是否已被使用（如果要更改昵称）
        if let Some(nickname) = &request.nickname {
            let current_user = self.user_repo.get_user_by_id(user_id).await?
                .ok_or_else(|| ServiceError::NotFound("User not found".to_string()))?;
                
            if nickname != &current_user.nickname && self.user_repo.nickname_exists(nickname).await? {
                return Err(ServiceError::ValidationError("Nickname already exists".to_string()));
            }
        }
        
        // 更新用户资料
        self.user_repo.update_user_profile(user_id, request).await?;
        
        Ok(())
    }
    
    // 上传用户头像
    pub async fn upload_avatar(
        &self,
        user_id: i64,
        file_data: Vec<u8>,
        file_name: String,
        content_type: String,
    ) -> Result<String, ServiceError> {
        // 验证文件类型
        if !content_type.starts_with("image/") {
            return Err(ServiceError::ValidationError("只支持上传图片文件".to_string()));
        }
        
        // 获取文件扩展名
        let extension = match file_name.split('.').last() {
            Some(ext) => ext.to_lowercase(),
            None => return Err(ServiceError::ValidationError("无效的文件名".to_string())),
        };
        
        // 验证文件扩展名
        if !vec!["jpg", "jpeg", "png", "gif"].contains(&extension.as_str()) {
            return Err(ServiceError::ValidationError("只支持jpg、jpeg、png、gif格式的图片".to_string()));
        }
        
        // 验证文件大小（最大2MB）
        if file_data.len() > 2 * 1024 * 1024 {
            return Err(ServiceError::ValidationError("文件大小不能超过2MB".to_string()));
        }
        
        // 生成唯一文件名
        let unique_filename = format!("{}_{}_{}.{}", user_id, Utc::now().timestamp(), rand::thread_rng().gen_range(1000..9999), extension);
        
        // 保存文件到本地存储（实际项目中可能会使用云存储服务）
        let avatar_dir = "uploads/avatars";
        std::fs::create_dir_all(avatar_dir).map_err(|e| ServiceError::IOError(format!("Failed to create avatar directory: {}", e)))?;
        
        let file_path = format!("{}/{}", avatar_dir, unique_filename);
        std::fs::write(&file_path, &file_data).map_err(|e| ServiceError::IOError(format!("Failed to save avatar file: {}", e)))?;
        
        // 生成头像URL
        let avatar_url = format!("/avatars/{}", unique_filename);
        
        // 更新用户头像
        let update_req = UpdateProfileRequest {
            nickname: None,
            avatar: Some(avatar_url.clone()),
        };
        
        self.user_repo.update_user_profile(user_id, update_req).await?;
        
        Ok(avatar_url)
    }
}

// 在 UserService 中添加以下方法

// 发送邮箱修改验证码
pub async fn send_email_change_code(
    &self,
    user_id: i64,
    request: SendEmailChangeCodeRequest,
) -> Result<SendEmailChangeCodeResponse, ServiceError> {
    // 获取用户信息
    let user = self.user_repo.get_user_by_id(user_id).await?
        .ok_or_else(|| ServiceError::NotFound("用户不存在".to_string()))?;
    
    // 检查新邮箱是否与当前邮箱相同
    if user.email == request.new_email {
        return Ok(SendEmailChangeCodeResponse {
            success: false,
            message: "新邮箱与当前邮箱相同".to_string(),
            expires_in: 0,
        });
    }
    
    // 检查新邮箱是否已被使用
    if self.user_repo.email_exists(&request.new_email).await? {
        return Ok(SendEmailChangeCodeResponse {
            success: false,
            message: "该邮箱已被使用".to_string(),
            expires_in: 0,
        });
    }
    
    // 创建邮箱修改验证码
    let verification_code = self.user_repo
        .create_verification_code(&request.new_email, "email_change")
        .await?;
        
    // 发送邮件
    self.email_service
        .send_verification_code(
            &request.new_email,
            &verification_code.code,
            "email_change",
        )
        .await
        .map_err(|e| ServiceError::EmailError(e.to_string()))?;
        
    Ok(SendEmailChangeCodeResponse {
        success: true,
        message: "邮箱修改验证码已发送到新邮箱".to_string(),
        expires_in: 600, // 10分钟
    })
}

// 验证邮箱修改
pub async fn verify_email_change(
    &self,
    user_id: i64,
    request: VerifyEmailChangeRequest,
) -> Result<(), ServiceError> {
    // 获取用户信息
    let user = self.user_repo.get_user_by_id(user_id).await?
        .ok_or_else(|| ServiceError::NotFound("用户不存在".to_string()))?;
    
    // 检查新邮箱是否与当前邮箱相同
    if user.email == request.new_email {
        return Err(ServiceError::ValidationError("新邮箱与当前邮箱相同".to_string()));
    }
    
    // 检查新邮箱是否已被使用
    if self.user_repo.email_exists(&request.new_email).await? {
        return Err(ServiceError::ValidationError("该邮箱已被使用".to_string()));
    }
    
    // 验证验证码
    let is_valid = self.user_repo
        .verify_code(&request.new_email, &request.code, "email_change")
        .await?;
        
    if !is_valid {
        return Err(ServiceError::ValidationError("验证码无效或已过期".to_string()));
    }
    
    // 更新邮箱
    self.user_repo.update_email(user_id, &request.new_email).await?;
    
    Ok(())
}

// 在 UserService 中添加以下方法

// 修改密码
// 移除所有 bcrypt 相关导入，使用 PasswordService
use rsws_common::password::PasswordService;

impl UserService {
    // 修改密码验证逻辑
    pub async fn change_password(
        &self,
        user_id: i64,
        request: ChangePasswordRequest,
    ) -> Result<(), ServiceError> {
        if request.new_password != request.confirm_password {
            return Err(ServiceError::ValidationError("新密码和确认密码不一致".to_string()));
        }
        
        let user = self.user_repo.get_user_by_id(user_id).await?
            .ok_or_else(|| ServiceError::NotFound("用户不存在".to_string()))?;
        
        // 使用Argon2验证当前密码
        let is_valid = PasswordService::verify_password(&request.current_password, &user.password_hash)
            .map_err(|e| ServiceError::HashError(format!("密码验证失败: {}", e)))?;
        
        if !is_valid {
            return Err(ServiceError::AuthError("当前密码错误".to_string()));
        }
        
        // 更新密码
        self.user_repo.update_password(user_id, &request.new_password).await?;
        
        Ok(())
    }
}

// 传统邮箱/密码登录
pub async fn traditional_login(
    &self,
    request: LoginRequest,
    ip_address: Option<IpAddr>,
) -> Result<TraditionalLoginResponse, ServiceError> {
    // 验证用户凭据
    let user = self.user_repo
        .verify_user_credentials(&request.email, &request.password)
        .await?
        .ok_or_else(|| ServiceError::AuthError("邮箱或密码错误".to_string()))?;
        
    if !user.is_active {
        return Ok(TraditionalLoginResponse {
            success: false,
            message: "账户已被禁用".to_string(),
            user_info: None,
            session_data: None,
        });
    }
    
    if !user.is_email_verified {
        return Ok(TraditionalLoginResponse {
            success: false,
            message: "邮箱未验证，请先完成邮箱验证".to_string(),
            user_info: None,
            session_data: None,
        });
    }
    
    // 清理过期会话
    self.user_repo.cleanup_expired_sessions(user.id).await?;
    
    // 检查活跃会话数量（限制最多5个同时登录）
    let active_sessions = self.user_repo.get_active_session_count(user.id).await?;
    if active_sessions >= 5 {
        return Ok(TraditionalLoginResponse {
            success: false,
            message: "登录会话过多，请先退出其他设备".to_string(),
            user_info: None,
            session_data: None,
        });
    }
    
    // 生成会话数据
    let session_token = self.generate_session_token();
    let (api_key, api_secret) = self.generate_api_credentials();
    let expires_at = Utc::now() + Duration::days(7); // 7天过期
    
    // 创建会话
    let _session_id = self.user_repo
        .create_user_session(
            user.id,
            &session_token,
            &api_key,
            &api_secret,
            request.device_info,
            ip_address,
            request.user_agent,
            expires_at,
        )
        .await?;
        
    Ok(TraditionalLoginResponse {
        success: true,
        message: "登录成功".to_string(),
        user_info: Some(UserInfo {
            id: user.id,
            nickname: user.nickname,
            email: user.email,
            avatar: user.avatar,
            is_email_verified: user.is_email_verified,
        }),
        session_data: Some(SessionData {
            session_token,
            api_key,
            api_secret,
            expires_at,
            signature_info: SignatureInfo {
                algorithm: "HMAC-SHA256".to_string(),
                timestamp_header: "X-Timestamp".to_string(),
                signature_header: "X-Signature".to_string(),
                api_key_header: "X-API-Key".to_string(),
            },
        }),
    })
}

// 用户登出
pub async fn logout(
    &self,
    user_id: i64,
    request: LogoutRequest,
) -> Result<LogoutResponse, ServiceError> {
    if request.logout_all_devices {
        // 登出所有设备
        self.user_repo.deactivate_all_sessions(user_id).await?;
    } else if let Some(session_token) = request.session_token {
        // 登出指定会话
        self.user_repo.deactivate_session_by_token(&session_token).await?;
    }
    
    Ok(LogoutResponse {
        success: true,
        message: "登出成功".to_string(),
    })
}

// 获取用户购买记录
pub async fn get_user_purchases(
    &self,
    user_id: i64,
    page: u32,
    page_size: u32,
) -> Result<PaginatedResponse<Order>, ServiceError> {
    // 这里需要调用订单服务获取用户购买记录
    // 暂时返回空结果，等订单服务实现后再完善
    Ok(PaginatedResponse {
        data: vec![],
        total: 0,
        page,
        page_size,
        total_pages: 0,
    })
}
}