//! 管理员仓储层

use chrono::Utc;
use rsws_common::error::RswsError;
use rsws_common::password::PasswordService;
use rsws_common::snowflake;
use rsws_model::user_models::admin::*;
use sqlx::PgPool;

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
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM admins WHERE email = $1")
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
        let mut needs_comma = false;

        if let Some(email) = &request.email {
            if needs_comma {
                query_builder.push(", ");
            }
            query_builder.push("email = ").push_bind(email);
            needs_comma = true;
        }

        if let Some(password) = &request.password {
            if needs_comma {
                query_builder.push(", ");
            }
            let password_hash = PasswordService::hash(password)?;
            query_builder
                .push("password_hash = ")
                .push_bind(password_hash);
            needs_comma = true;
        }

        if let Some(username) = &request.username {
            if needs_comma {
                query_builder.push(", ");
            }
            query_builder.push("username = ").push_bind(username);
            needs_comma = true;
        }

        if let Some(avatar_url) = &request.avatar_url {
            if needs_comma {
                query_builder.push(", ");
            }
            query_builder.push("avatar_url = ").push_bind(avatar_url);
            needs_comma = true;
        }

        if let Some(is_active) = request.is_active {
            if needs_comma {
                query_builder.push(", ");
            }
            query_builder.push("is_active = ").push_bind(is_active);
            needs_comma = true;
        }

        if let Some(role) = &request.role {
            if needs_comma {
                query_builder.push(", ");
            }
            query_builder.push("role = ").push_bind(role);
            needs_comma = true;
        }

        if let Some(permissions) = &request.permissions {
            if needs_comma {
                query_builder.push(", ");
            }
            query_builder.push("permissions = ").push_bind(
                serde_json::to_value(permissions).unwrap_or(serde_json::Value::Array(vec![])),
            );
            needs_comma = true;
        }

        // Always update updated_at
        if needs_comma {
            query_builder.push(", ");
        }
        query_builder.push("updated_at = ").push_bind(Utc::now());

        query_builder.push(" WHERE id = ").push_bind(request.id);
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
            .map_err(|e| {
                RswsError::internal(format!("Failed to update admin login info: {}", e))
            })?;

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
}
