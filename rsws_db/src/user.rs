use sqlx::PgPool;
use chrono::{DateTime, Utc, Duration};
use bcrypt::{hash, DEFAULT_COST};
use rand::Rng;
use rsws_model::user::*;
use rsws_common::error::DbError;
use rsws_common::snowflake;
use rsws_common::{snowflake, password::PasswordService};

pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    
    // 检查邮箱是否已存在
    pub async fn email_exists(&self, email: &str) -> Result<bool, DbError> {
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM users WHERE email = $1"
        )
        .bind(email)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::QueryError(format!("Failed to check email existence: {}", e)))?;
        
        Ok(count.0 > 0)
    }
    
    // 检查昵称是否已存在
    pub async fn nickname_exists(&self, nickname: &str) -> Result<bool, DbError> {
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM users WHERE nickname = $1"
        )
        .bind(nickname)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::QueryError(format!("Failed to check nickname existence: {}", e)))?;
        
        Ok(count.0 > 0)
    }
    
    // 创建用户（使用雪花ID和Argon2）
    pub async fn create_user(
        &self,
        nickname: &str,
        email: &str,
        password: &str,
    ) -> Result<User, DbError> {
        let user_id = snowflake::next_id();
        let password_hash = PasswordService::hash_password(password)
            .map_err(|e| DbError::HashError(format!("Failed to hash password: {}", e)))?;
            
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (id, username, email, password_hash, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, TRUE, NOW(), NOW())
            RETURNING *
            "#
        )
        .bind(user_id)
        .bind(nickname)
        .bind(email)
        .bind(password_hash)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::QueryError(format!("Failed to create user: {}", e)))?;
        
        Ok(user)
    }
    
    // 验证用户登录凭据（使用Argon2）
    pub async fn verify_user_credentials(
        &self,
        email: &str,
        password: &str,
    ) -> Result<Option<User>, DbError> {
        let user = self.get_user_by_email(email).await?;
        
        if let Some(user) = user {
            let is_valid = PasswordService::verify_password(password, &user.password_hash)
                .map_err(|e| DbError::HashError(format!("Password verification failed: {}", e)))?;
                
            if is_valid && user.is_active {
                Ok(Some(user))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }
    
    // 更新用户密码（使用Argon2）
    pub async fn update_password(
        &self,
        user_id: i64,
        new_password: &str,
    ) -> Result<(), DbError> {
        let password_hash = PasswordService::hash_password(new_password)
            .map_err(|e| DbError::HashError(format!("Failed to hash password: {}", e)))?;
        
        sqlx::query(
            "UPDATE users SET password_hash = $1, updated_at = CURRENT_TIMESTAMP WHERE id = $2"
        )
        .bind(password_hash)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::QueryError(format!("Failed to update password: {}", e)))?;
        
        Ok(())
    }
    
    // 创建用户会话（使用雪花ID）
    pub async fn create_user_session(
        &self,
        user_id: i64,
        session_token: &str,
        api_key: &str,
        api_secret: &str,
        device_info: Option<serde_json::Value>,
        ip_address: Option<std::net::IpAddr>,
        user_agent: Option<String>,
        expires_at: DateTime<Utc>,
    ) -> Result<i64, DbError> {
        let session_id = snowflake::next_id();
        
        sqlx::query(
            r#"
            INSERT INTO user_sessions (id, user_id, session_token, api_key, api_secret, device_info, ip_address, user_agent, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#
        )
        .bind(session_id)
        .bind(user_id)
        .bind(session_token)
        .bind(api_key)
        .bind(api_secret)
        .bind(device_info)
        .bind(ip_address)
        .bind(user_agent)
        .bind(expires_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::QueryError(format!("Failed to create user session: {}", e)))?;
        
        Ok(session_id)
    }
}

// 在 UserRepository 中添加以下方法

// 根据ID获取用户
pub async fn get_user_by_id(&self, user_id: i64) -> Result<Option<User>, DbError> {
    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE id = $1"
    )
    .bind(user_id)
    .fetch_optional(&self.pool)
    .await
    .map_err(|e| DbError::QueryError(format!("Failed to get user by id: {}", e)))?;
    
    Ok(user)
}

// 更新用户邮箱
pub async fn update_email(
    &self,
    user_id: i64,
    new_email: &str,
) -> Result<(), DbError> {
    // 更新邮箱
    sqlx::query(
        "UPDATE users SET email = $1, is_email_verified = true, updated_at = CURRENT_TIMESTAMP WHERE id = $2"
    )
    .bind(new_email)
    .bind(user_id)
    .execute(&self.pool)
    .await
    .map_err(|e| DbError::QueryError(format!("Failed to update email: {}", e)))?;
    
    Ok(())
}

// 更新用户密码
pub async fn update_password(
    &self,
    user_id: i64,
    new_password: &str,
) -> Result<(), DbError> {
    // 对新密码进行哈希处理
    let password_hash = hash(new_password, DEFAULT_COST)
        .map_err(|e| DbError::HashError(format!("Failed to hash password: {}", e)))?;
    
    // 更新密码
    sqlx::query(
        "UPDATE users SET password_hash = $1, updated_at = CURRENT_TIMESTAMP WHERE id = $2"
    )
    .bind(password_hash)
    .bind(user_id)
    .execute(&self.pool)
    .await
    .map_err(|e| DbError::QueryError(format!("Failed to update password: {}", e)))?;
    
    Ok(())
}

// 更新用户资料
pub async fn update_user_profile(
    &self,
    user_id: i64,
    request: UpdateProfileRequest,
) -> Result<(), DbError> {
    let mut query_builder = sqlx::QueryBuilder::new("UPDATE users SET ");
    let mut separated = query_builder.separated(", ");
    
    if let Some(nickname) = &request.nickname {
        separated.push("nickname = ");
        separated.push_bind(nickname);
    }
    
    if let Some(avatar) = &request.avatar {
        separated.push("avatar = ");
        separated.push_bind(avatar);
    }
    
    separated.push("updated_at = CURRENT_TIMESTAMP");
    
    query_builder.push(" WHERE id = ");
    query_builder.push_bind(user_id);
    
    let query = query_builder.build();
    query.execute(&self.pool)
        .await
        .map_err(|e| DbError::QueryError(format!("Failed to update user profile: {}", e)))?;
        
    Ok(())
}