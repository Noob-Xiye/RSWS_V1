//! 管理员仓储层

use rsws_common::error::RswsError;
use rsws_common::snowflake;
use rsws_common::password::PasswordService;
use rsws_model::user_models::admin::*;
use sqlx::PgPool;
use chrono::Utc;

/// 管理员仓储
pub struct AdminRepository {
    pool: PgPool,
}

impl AdminRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 检查邮箱是否已存在
    pub async fn email_exists(&self, email: &str) -> Result<bool, RswsError> {
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM admins WHERE email = $1"
        )
        .bind(email)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to check admin email: {}", e)))?;

        Ok(count.0 > 0)
    }

    /// 创建管理员（雪花ID + Argon2）
    pub async fn create_admin(
        &self,
        email: &str,
        password: &str,
        username: &str,
        role: &str,
    ) -> Result<Admin, RswsError> {
        let admin_id = snowflake::next_id();
        let password_hash = PasswordService::hash(password)?;

        let admin = sqlx::query_as::<_, Admin>(
            r#"
            INSERT INTO admins (id, email, password_hash, username, role, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, TRUE, NOW(), NOW())
            RETURNING *
            "#,
        )
        .bind(admin_id)
        .bind(email)
        .bind(&password_hash)
        .bind(username)
        .bind(role)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to create admin: {}", e)))?;

        Ok(admin)
    }

    /// 通过 ID 获取管理员
    pub async fn get_admin_by_id(&self, id: i64) -> Result<Option<Admin>, RswsError> {
        sqlx::query_as::<_, Admin>("SELECT * FROM admins WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| RswsError::internal(format!("Failed to get admin by id: {}", e)))
    }

    /// 通过邮箱获取管理员
    pub async fn get_admin_by_email(&self, email: &str) -> Result<Option<Admin>, RswsError> {
        sqlx::query_as::<_, Admin>("SELECT * FROM admins WHERE email = $1")
            .bind(email)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| RswsError::internal(format!("Failed to get admin by email: {}", e)))
    }

    /// 验证管理员登录凭据（Argon2）
    pub async fn verify_admin_credentials(
        &self,
        email: &str,
        password: &str,
    ) -> Result<Option<Admin>, RswsError> {
        let admin = self.get_admin_by_email(email).await?;

        if let Some(admin) = admin {
            let is_valid = PasswordService::verify(password, &admin.password_hash)?;
            if is_valid && admin.is_active {
                Ok(Some(admin))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    /// 更新管理员信息
    pub async fn update_admin(&self, request: &UpdateAdminRequest) -> Result<Admin, RswsError> {
        let mut query_builder = sqlx::QueryBuilder::new("UPDATE admins SET ");
        let mut separated = query_builder.separated(", ");

        if let Some(email) = &request.email {
            separated.push("email = ");
            separated.push_bind(email);
        }

        if let Some(password) = &request.password {
            let password_hash = PasswordService::hash(password)?;
            separated.push("password_hash = ");
            separated.push_bind(password_hash);
        }

        if let Some(username) = &request.username {
            separated.push("username = ");
            separated.push_bind(username);
        }

        if let Some(avatar_url) = &request.avatar_url {
            separated.push("avatar_url = ");
            separated.push_bind(avatar_url);
        }

        if let Some(is_active) = request.is_active {
            separated.push("is_active = ");
            separated.push_bind(is_active);
        }

        if let Some(role) = &request.role {
            separated.push("role = ");
            separated.push_bind(role);
        }

        if let Some(permissions) = &request.permissions {
            separated.push("permissions = ");
            separated.push_bind(serde_json::to_value(permissions).unwrap_or(serde_json::Value::Array(vec![])));
        }

        separated.push("updated_at = ");
        separated.push_bind(Utc::now());

        query_builder.push(" WHERE id = ");
        query_builder.push_bind(request.id);
        query_builder.push(" RETURNING *");

        let query = query_builder.build_query_as::<Admin>();

        query
            .fetch_one(&self.pool)
            .await
            .map_err(|e| RswsError::internal(format!("Failed to update admin: {}", e)))
    }

    /// 更新管理员最后登录信息
    pub async fn update_admin_login(&self, id: i64, ip: Option<&str>) -> Result<(), RswsError> {
        sqlx::query("UPDATE admins SET last_login_at = $1, last_login_ip = $2 WHERE id = $3")
            .bind(Utc::now())
            .bind(ip)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| RswsError::internal(format!("Failed to update admin login info: {}", e)))?;

        Ok(())
    }

    /// 获取管理员列表
    pub async fn get_admins(
        &self,
        page: i64,
        page_size: i64,
        role: Option<&str>,
    ) -> Result<Vec<Admin>, RswsError> {
        let mut query_builder = sqlx::QueryBuilder::new("SELECT * FROM admins ");

        if let Some(role) = role {
            query_builder.push(" WHERE role = ");
            query_builder.push_bind(role);
        }

        query_builder.push(" ORDER BY id DESC LIMIT ");
        query_builder.push_bind(page_size);
        query_builder.push(" OFFSET ");
        query_builder.push_bind((page - 1) * page_size);

        let query = query_builder.build_query_as::<Admin>();

        query
            .fetch_all(&self.pool)
            .await
            .map_err(|e| RswsError::internal(format!("Failed to get admins: {}", e)))
    }

    /// 获取管理员总数
    pub async fn get_admins_count(&self, role: Option<&str>) -> Result<i64, RswsError> {
        let mut query_builder = sqlx::QueryBuilder::new("SELECT COUNT(*) FROM admins");

        if let Some(role) = role {
            query_builder.push(" WHERE role = ");
            query_builder.push_bind(role);
        }

        let query = query_builder.build_query_as::<(i64,)>();

        let (count,) = query
            .fetch_one(&self.pool)
            .await
            .map_err(|e| RswsError::internal(format!("Failed to get admins count: {}", e)))?;

        Ok(count)
    }

    /// 记录管理员操作日志
    #[allow(clippy::too_many_arguments)]
    pub async fn log_admin_operation(
        &self,
        admin_id: i64,
        operation_type: &str,
        operation_target: Option<&str>,
        target_id: Option<&str>,
        operation_content: Option<&str>,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
    ) -> Result<(), RswsError> {
        let log_id = snowflake::next_id();

        sqlx::query(
            r#"
            INSERT INTO admin_operation_logs (id, admin_id, operation_type, operation_target, target_id, operation_content, ip_address, user_agent)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(log_id)
        .bind(admin_id)
        .bind(operation_type)
        .bind(operation_target)
        .bind(target_id)
        .bind(operation_content)
        .bind(ip_address)
        .bind(user_agent)
        .execute(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to log admin operation: {}", e)))?;

        Ok(())
    }

    // ==================== Admin API Key ====================

    /// 验证管理员 API Key，返回 AdminApiKey + Admin
    pub async fn validate_admin_api_key(
        &self,
        api_key: &str,
        api_secret: &str,
    ) -> Result<Option<(AdminApiKey, Admin)>, RswsError> {
        let key_record = sqlx::query_as::<_, AdminApiKey>(
            r#"
            SELECT * FROM admin_api_keys
            WHERE api_key = $1 AND is_active = true
            AND (expires_at IS NULL OR expires_at > NOW())
            "#,
        )
        .bind(api_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to query admin API key: {}", e)))?;

        let key_record = match key_record {
            Some(k) => k,
            None => return Ok(None),
        };

        // 验证 api_secret（admin_api_keys 存储的是 api_secret_encrypted）
        // 使用 PasswordService 做常量时间比较
        let is_valid = PasswordService::verify(api_secret, &key_record.api_secret_encrypted)?;
        if !is_valid {
            return Ok(None);
        }

        // 查关联的 admin
        let admin = sqlx::query_as::<_, Admin>(
            "SELECT * FROM admins WHERE id = $1 AND is_active = true"
        )
        .bind(key_record.admin_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to get admin by API key: {}", e)))?;

        match admin {
            Some(a) => Ok(Some((key_record, a))),
            None => Ok(None),
        }
    }

    /// 更新管理员 API Key 最后使用时间
    pub async fn update_admin_api_key_last_used(&self, key_id: i64) -> Result<(), RswsError> {
        sqlx::query("UPDATE admin_api_keys SET last_used_at = NOW() WHERE id = $1")
            .bind(key_id)
            .execute(&self.pool)
            .await
            .map_err(|e| RswsError::internal(format!("Failed to update admin API key last used: {}", e)))?;
        Ok(())
    }

    /// 创建管理员 API Key
    pub async fn create_admin_api_key(
        &self,
        admin_id: i64,
        name: &str,
        permissions: Vec<String>,
        rate_limit: Option<i32>,
        expires_in_days: Option<i32>,
    ) -> Result<(AdminApiKey, String), RswsError> {
        use rand::{Rng, SeedableRng};
        use base64::{Engine as _, engine::general_purpose};

        let mut rng = rand::rngs::StdRng::from_os_rng();
        let key_bytes: [u8; 32] = rng.random();
        let api_key = format!("adm_ak_{}", general_purpose::URL_SAFE_NO_PAD.encode(key_bytes));

        let secret_bytes: [u8; 64] = rng.random();
        let api_secret = format!("adm_sk_{}", general_purpose::URL_SAFE_NO_PAD.encode(secret_bytes));

        // 用 Argon2 加密 secret 存储
        let api_secret_encrypted = PasswordService::hash(&api_secret)?;

        let permissions_json = serde_json::to_value(&permissions)
            .map_err(|e| RswsError::internal(format!("Failed to serialize permissions: {}", e)))?;

        let expires_at = expires_in_days.map(|d| Utc::now() + chrono::Duration::days(d as i64));

        let record = sqlx::query_as::<_, AdminApiKey>(
            r#"
            INSERT INTO admin_api_keys (admin_id, name, api_key, api_secret_encrypted, permissions, rate_limit, expires_at, is_active)
            VALUES ($1, $2, $3, $4, $5, $6, $7, true)
            RETURNING *
            "#,
        )
        .bind(admin_id)
        .bind(name)
        .bind(&api_key)
        .bind(&api_secret_encrypted)
        .bind(&permissions_json)
        .bind(rate_limit)
        .bind(expires_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to create admin API key: {}", e)))?;

        Ok((record, api_secret))
    }

    /// 获取管理员的所有 API Key
    pub async fn get_admin_api_keys(&self, admin_id: i64) -> Result<Vec<AdminApiKey>, RswsError> {
        sqlx::query_as::<_, AdminApiKey>(
            "SELECT * FROM admin_api_keys WHERE admin_id = $1 ORDER BY created_at DESC"
        )
        .bind(admin_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to get admin API keys: {}", e)))
    }

    /// 根据 api_key 值获取管理员 API Key 记录
    pub async fn get_admin_api_key_by_key(&self, api_key: &str) -> Result<Option<AdminApiKey>, RswsError> {
        sqlx::query_as::<_, AdminApiKey>(
            "SELECT * FROM admin_api_keys WHERE api_key = $1 AND is_active = true"
        )
        .bind(api_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to get admin API key by key: {}", e)))
    }

    /// 删除管理员 API Key
    pub async fn delete_admin_api_key(&self, key_id: i64, admin_id: i64) -> Result<bool, RswsError> {
        let result = sqlx::query("DELETE FROM admin_api_keys WHERE id = $1 AND admin_id = $2")
            .bind(key_id)
            .bind(admin_id)
            .execute(&self.pool)
            .await
            .map_err(|e| RswsError::internal(format!("Failed to delete admin API key: {}", e)))?;

        Ok(result.rows_affected() > 0)
    }

    /// 更新管理员 API Key 状态
    pub async fn update_api_key_status(
        &self,
        key_id: i64,
        admin_id: i64,
        is_active: bool,
    ) -> Result<bool, RswsError> {
        let result = sqlx::query(
            "UPDATE admin_api_keys SET is_active = $1, updated_at = NOW() WHERE id = $2 AND admin_id = $3"
        )
        .bind(is_active)
        .bind(key_id)
        .bind(admin_id)
        .execute(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to update API key status: {}", e)))?;

        Ok(result.rows_affected() > 0)
    }

    /// 禁用管理员所有 API Key（停用管理员 / 改密码时使用）
    pub async fn deactivate_admin_api_keys(&self, admin_id: i64) -> Result<u64, RswsError> {
        let result = sqlx::query("UPDATE admin_api_keys SET is_active = false WHERE admin_id = $1 AND is_active = true")
            .bind(admin_id)
            .execute(&self.pool)
            .await
            .map_err(|e| RswsError::internal(format!("Failed to deactivate admin API keys: {}", e)))?;
        Ok(result.rows_affected())
    }
}
