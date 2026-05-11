//! 用户服务

use rsws_common::email::EmailService;
use rsws_common::error::RswsError;
use rsws_common::error_code::ErrorCode;
use rsws_common::password::PasswordService;
use rsws_db::{RedisService, UserRepository};
use rsws_model::user_models::user::{
    LoginRequest, LoginResponse, RegisterRequest, User, UserInfo,
};
use rsws_model::user_models::AdminUserView;
use tracing::{info, warn};

/// 用户服务
pub struct UserService {
    user_repo: UserRepository,
    redis: Option<RedisService>,
    email_service: Option<EmailService>,
}

impl UserService {
    /// 创建用户服务实例
    pub fn new(user_repo: UserRepository) -> Self {
        Self {
            user_repo,
            redis: None,
            email_service: None,
        }
    }

    /// 创建用户服务实例（带 Redis）
    pub fn with_redis(user_repo: UserRepository, redis: RedisService) -> Self {
        Self {
            user_repo,
            redis: Some(redis),
            email_service: None,
        }
    }

    /// 创建用户服务实例（完整）
    pub fn with_services(
        user_repo: UserRepository,
        redis: RedisService,
        email_service: EmailService,
    ) -> Self {
        Self {
            user_repo,
            redis: Some(redis),
            email_service: Some(email_service),
        }
    }

    /// 创建用户服务实例（带 Email，无 Redis）
    pub fn with_redis_and_email(user_repo: UserRepository, email_service: EmailService) -> Self {
        Self {
            user_repo,
            redis: None,
            email_service: Some(email_service),
        }
    }

    /// 用户注册
    pub async fn register(&self, req: &RegisterRequest) -> Result<User, RswsError> {
        // 检查用户名是否已存在
        if self
            .user_repo
            .find_user_by_username(&req.username)
            .await?
            .is_some()
        {
            return Err(RswsError::business(ErrorCode::USER_USERNAME_EXISTS));
        }

        // 检查邮箱是否已存在
        if self
            .user_repo
            .find_user_by_email(&req.email)
            .await?
            .is_some()
        {
            return Err(RswsError::business(ErrorCode::USER_EMAIL_EXISTS));
        }

        // 哈希密码
        let password_hash = PasswordService::hash(&req.password)?;

        // 创建用户
        let user = self
            .user_repo
            .create_user(&req.username, &req.nickname, &req.email, &password_hash)
            .await?;

        info!("User registered: {} ({})", user.id, user.username);

        Ok(user)
    }

    /// 用户登录（支持两种方式）
    pub async fn login(&self, req: &LoginRequest) -> Result<LoginResponse, RswsError> {
        match req.login_type.as_str() {
            "password" => self.login_by_password(req).await,
            "code" => self.login_by_code(req).await,
            _ => Err(RswsError::business(ErrorCode::INVALID_PARAMETER)),
        }
    }

    /// 用户名 + 密码登录
    async fn login_by_password(&self, req: &LoginRequest) -> Result<LoginResponse, RswsError> {
        let username = req
            .username
            .as_ref()
            .ok_or_else(|| RswsError::business(ErrorCode::INVALID_PARAMETER))?;
        let password = req
            .password
            .as_ref()
            .ok_or_else(|| RswsError::business(ErrorCode::INVALID_PARAMETER))?;

        // 先从 Redis 缓存读取
        let user = if let Some(ref redis) = self.redis {
            if let Some(cached) = redis.get_cached_user::<User>(0).await.ok().flatten() {
                if cached.username == *username {
                    info!("User {} found in Redis cache", username);
                    cached
                } else {
                    self.find_and_cache_user_by_username(username, redis)
                        .await?
                }
            } else {
                self.find_and_cache_user_by_username(username, redis)
                    .await?
            }
        } else {
            // 无缓存，直接查数据库
            self.user_repo
                .find_user_by_username(username)
                .await?
                .ok_or_else(|| RswsError::business(ErrorCode::USER_NOT_FOUND))?
        };

        // 验证密码
        let valid = PasswordService::verify(password, &user.password_hash)?;
        if !valid {
            warn!("Invalid password for user: {}", username);
            return Err(RswsError::business(ErrorCode::AUTH_INVALID_CREDENTIALS));
        }

        // 检查用户状态
        if !user.is_active {
            return Err(RswsError::business(ErrorCode::USER_DISABLED));
        }

        self.create_login_response(user).await
    }

    /// 邮箱 + 验证码登录
    async fn login_by_code(&self, req: &LoginRequest) -> Result<LoginResponse, RswsError> {
        let email = req
            .email
            .as_ref()
            .ok_or_else(|| RswsError::business(ErrorCode::INVALID_PARAMETER))?;
        let code = req
            .verification_code
            .as_ref()
            .ok_or_else(|| RswsError::business(ErrorCode::INVALID_PARAMETER))?;

        // 验证验证码
        let redis = self
            .redis
            .as_ref()
            .ok_or_else(|| RswsError::internal("Redis not configured"))?;

        let (valid, remaining) = redis.verify_code(email, "login", code).await?;
        if !valid {
            warn!(
                "Invalid verification code for email: {} (remaining attempts: {})",
                email, remaining
            );
            return Err(RswsError::business_with_message(
                ErrorCode::AUTH_INVALID_CREDENTIALS,
                format!("验证码错误，剩余尝试次数: {}", remaining),
            ));
        }

        // 查找用户
        let user = self.find_and_cache_user_by_email(email, redis).await?;

        // 检查用户状态
        if !user.is_active {
            return Err(RswsError::business(ErrorCode::USER_DISABLED));
        }

        self.create_login_response(user).await
    }

    /// 查找用户并缓存（按用户名）
    async fn find_and_cache_user_by_username(
        &self,
        username: &str,
        redis: &RedisService,
    ) -> Result<User, RswsError> {
        let user = self
            .user_repo
            .find_user_by_username(username)
            .await?
            .ok_or_else(|| RswsError::business(ErrorCode::USER_NOT_FOUND))?;

        // 缓存用户信息
        let _ = redis.cache_user(user.id, &user).await;
        Ok(user)
    }

    /// 查找用户并缓存（按邮箱）
    async fn find_and_cache_user_by_email(
        &self,
        email: &str,
        redis: &RedisService,
    ) -> Result<User, RswsError> {
        let user = self
            .user_repo
            .find_user_by_email(email)
            .await?
            .ok_or_else(|| RswsError::business(ErrorCode::USER_NOT_FOUND))?;

        // 缓存用户信息
        let _ = redis.cache_user(user.id, &user).await;
        Ok(user)
    }

    /// 创建登录响应
    ///
    /// 注意：session_data 由 handler 层通过 api_key_service.create() 生成并附加
    /// 这样确保 api_key 正确持久化到数据库，供后续验签查找
    async fn create_login_response(&self, user: User) -> Result<LoginResponse, RswsError> {
        info!("User logged in: {} ({})", user.id, user.username);

        Ok(LoginResponse {
            success: true,
            message: "登录成功".to_string(),
            user_info: Some(UserInfo {
                id: user.id,
                email: user.email.clone(),
                username: user.username.clone(),
                nickname: user.nickname.clone(),
                avatar_url: user.avatar_url.clone(),
                is_active: user.is_active,
            }),
            session_data: None, // handler 层填充
        })
    }

    /// 发送登录验证码
    pub async fn send_login_code(&self, email: &str) -> Result<i64, RswsError> {
        let redis = self
            .redis
            .as_ref()
            .ok_or_else(|| RswsError::internal("Redis not configured"))?;

        // 检查用户是否存在
        let user = self
            .user_repo
            .find_user_by_email(email)
            .await?
            .ok_or_else(|| RswsError::business(ErrorCode::USER_NOT_FOUND))?;

        if !user.is_active {
            return Err(RswsError::business(ErrorCode::USER_DISABLED));
        }

        // 检查是否已有验证码（防止频繁发送）
        if redis.has_verification_code(email, "login").await? {
            return Err(RswsError::business_with_message(
                ErrorCode::RATE_LIMIT_EXCEEDED,
                "验证码已发送，请稍后再试",
            ));
        }

        // 生成 6 位验证码
        let code = format!("{:06}", rand::random::<u32>() % 1_000_000);

        // 存储验证码
        redis.set_verification_code(email, "login", &code).await?;

        // 发送邮件
        if let Some(ref email_service) = self.email_service {
            email_service
                .send_verification_code(email, &code, "login")
                .await?;
        }

        info!("Login verification code sent to: {}", email);
        Ok(300) // 返回有效期（秒）
    }

    /// 获取用户信息
    pub async fn get_user(&self, user_id: i64) -> Result<User, RswsError> {
        // 先从 Redis 读取
        if let Some(ref redis) = self.redis {
            if let Some(user) = redis.get_cached_user::<User>(user_id).await? {
                return Ok(user);
            }
        }

        // 从数据库读取
        let user = self
            .user_repo
            .find_user_by_id(user_id)
            .await?
            .ok_or_else(|| RswsError::business(ErrorCode::USER_NOT_FOUND))?;

        // 缓存
        if let Some(ref redis) = self.redis {
            let _ = redis.cache_user(user_id, &user).await;
        }

        Ok(user)
    }

    /// 更新用户昵称
    pub async fn update_nickname(&self, user_id: i64, nickname: &str) -> Result<User, RswsError> {
        let user = self
            .user_repo
            .update_user_nickname(user_id, nickname)
            .await?;

        // 清除缓存
        if let Some(ref redis) = self.redis {
            let _ = redis.clear_user_cache(user_id).await;
        }

        info!("User {} nickname updated to: {}", user_id, nickname);
        Ok(user)
    }

    /// 修改密码
    pub async fn change_password(
        &self,
        user_id: i64,
        old_password: &str,
        new_password: &str,
    ) -> Result<(), RswsError> {
        // 获取用户
        let user = self.get_user(user_id).await?;

        // 验证旧密码
        let valid = PasswordService::verify(old_password, &user.password_hash)?;
        if !valid {
            return Err(RswsError::business(ErrorCode::AUTH_INVALID_CREDENTIALS));
        }

        // 哈希新密码
        let new_hash = PasswordService::hash(new_password)?;

        // 更新密码
        self.user_repo
            .update_user_password(user_id, &new_hash)
            .await?;

        // 清除缓存
        if let Some(ref redis) = self.redis {
            let _ = redis.clear_user_cache(user_id).await;
        }

        info!("Password changed for user: {}", user_id);
        Ok(())
    }

    /// 发送验证码（通用，支持 register / login / reset_password）
    pub async fn send_verification_code(&self, email: &str, scene: &str) -> Result<i64, RswsError> {
        let redis = self
            .redis
            .as_ref()
            .ok_or_else(|| RswsError::internal("Redis not configured"))?;

        // register 场景不检查用户存在性，login / reset_password 需要检查
        if scene != "register" {
            let user = self
                .user_repo
                .find_user_by_email(email)
                .await?
                .ok_or_else(|| RswsError::business(ErrorCode::USER_NOT_FOUND))?;
            if !user.is_active {
                return Err(RswsError::business(ErrorCode::USER_DISABLED));
            }
        }

        // 检查是否已有验证码（防止频繁发送）
        if redis.has_verification_code(email, scene).await? {
            return Err(RswsError::business_with_message(
                ErrorCode::RATE_LIMIT_EXCEEDED,
                "验证码已发送，请稍后再试",
            ));
        }

        // 生成 6 位验证码
        let code = format!("{:06}", rand::random::<u32>() % 1_000_000);

        // 存储验证码
        redis.set_verification_code(email, scene, &code).await?;

        // 发送邮件
        if let Some(ref email_service) = self.email_service {
            email_service
                .send_verification_code(email, &code, scene)
                .await?;
        }

        info!("Verification code sent to {} for scene: {}", email, scene);
        Ok(300)
    }

    /// 禁用用户（管理员操作）
    pub async fn deactivate_user(&self, user_id: i64) -> Result<(), RswsError> {
        let user = self
            .user_repo
            .find_user_by_id(user_id)
            .await?
            .ok_or_else(|| RswsError::business(ErrorCode::USER_NOT_FOUND))?;

        if !user.is_active {
            return Err(RswsError::business_with_message(
                ErrorCode::INVALID_PARAMETER,
                "用户已被禁用",
            ));
        }

        self.user_repo.update_user_active(user_id, false).await?;

        // 清除缓存
        if let Some(ref redis) = self.redis {
            let _ = redis.clear_user_cache(user_id).await;
        }

        info!("User {} deactivated by admin", user_id);
        Ok(())
    }

    /// 启用用户（管理员操作）
    pub async fn activate_user(&self, user_id: i64) -> Result<(), RswsError> {
        let user = self
            .user_repo
            .find_user_by_id(user_id)
            .await?
            .ok_or_else(|| RswsError::business(ErrorCode::USER_NOT_FOUND))?;

        if user.is_active {
            return Err(RswsError::business_with_message(
                ErrorCode::INVALID_PARAMETER,
                "用户已启用",
            ));
        }

        self.user_repo.update_user_active(user_id, true).await?;

        // 清除缓存
        if let Some(ref redis) = self.redis {
            let _ = redis.clear_user_cache(user_id).await;
        }

        info!("User {} activated by admin", user_id);
        Ok(())
    }

    /// 获取用户列表（管理员分页查询）
    pub async fn list_users(
        &self,
        page: i64,
        page_size: i64,
        email: Option<&str>,
        username: Option<&str>,
        is_active: Option<bool>,
    ) -> Result<(Vec<AdminUserView>, i64), RswsError> {
        let users = self
            .user_repo
            .get_users(page, page_size, email, username, is_active)
            .await?
            .into_iter()
            .map(AdminUserView::from)
            .collect();
        let total = self
            .user_repo
            .get_users_count(email, username, is_active)
            .await?;
        Ok((users, total))
    }
}

// ==================== 单元测试 ====================

#[cfg(test)]
mod tests {

    #[test]
    fn test_user_service_new() {
        // 仅测试构造函数
    }
}
