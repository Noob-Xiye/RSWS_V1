use sqlx::PgPool;
use chrono::{DateTime, Utc};
use bcrypt::{hash, verify, DEFAULT_COST};
use rsws_model::user::admin::*;
use rsws_common::error::DbError;
use rsws_common::{snowflake, password::PasswordService};

pub struct AdminRepository {
    pool: PgPool,
}

impl AdminRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    
    // 检查邮箱是否已存在
    pub async fn email_exists(&self, email: &str) -> Result<bool, DbError> {
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM admins WHERE email = $1"
        )
        .bind(email)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::QueryError(format!("Failed to check admin email existence: {}", e)))?;
        
        Ok(count.0 > 0)
    }
    
    // 创建管理员
    pub async fn create_admin(
        &self,
        request: &CreateAdminRequest,
    ) -> Result<Admin, DbError> {
        // 密码加密
        let password_hash = hash(&request.password, DEFAULT_COST)
            .map_err(|e| DbError::HashError(format!("Failed to hash password: {}", e)))?;
        
        // 默认权限
        let permissions = request.permissions.clone().unwrap_or_else(|| vec![]);
        
        let admin = sqlx::query_as::<_, Admin>(
            r#"INSERT INTO admins 
               (email, password_hash, username, role, permissions) 
               VALUES ($1, $2, $3, $4, $5) 
               RETURNING *"#
        )
        .bind(&request.email)
        .bind(&password_hash)
        .bind(&request.username)
        .bind(&request.role)
        .bind(serde_json::to_value(&permissions).unwrap_or(serde_json::Value::Array(vec![])))
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::QueryError(format!("Failed to create admin: {}", e)))?;
        
        Ok(admin)
    }
    
    // 通过ID获取管理员
    pub async fn get_admin_by_id(&self, id: i64) -> Result<Option<Admin>, DbError> {
        let admin = sqlx::query_as::<_, Admin>(
            "SELECT * FROM admins WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::QueryError(format!("Failed to get admin by id: {}", e)))?;
        
        Ok(admin)
    }
    
    // 通过邮箱获取管理员
    pub async fn get_admin_by_email(&self, email: &str) -> Result<Option<Admin>, DbError> {
        let admin = sqlx::query_as::<_, Admin>(
            "SELECT * FROM admins WHERE email = $1"
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::QueryError(format!("Failed to get admin by email: {}", e)))?;
        
        Ok(admin)
    }
    
    // 更新管理员信息
    pub async fn update_admin(&self, request: &UpdateAdminRequest) -> Result<Admin, DbError> {
        let mut query_builder = sqlx::QueryBuilder::new("UPDATE admins SET ");
        let mut separated = query_builder.separated(", ");
        
        if let Some(email) = &request.email {
            separated.push("email = ");
            separated.push_bind(email);
        }
        
        if let Some(password) = &request.password {
            let password_hash = hash(password, DEFAULT_COST)
                .map_err(|e| DbError::HashError(format!("Failed to hash password: {}", e)))?;
            
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
        
        let admin = query
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DbError::QueryError(format!("Failed to update admin: {}", e)))?;
        
        Ok(admin)
    }
    
    // 更新管理员最后登录信息
    pub async fn update_admin_login(&self, id: i64, ip: Option<&str>) -> Result<(), DbError> {
        sqlx::query(
            "UPDATE admins SET last_login_at = $1, last_login_ip = $2 WHERE id = $3"
        )
        .bind(Utc::now())
        .bind(ip)
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::QueryError(format!("Failed to update admin login info: {}", e)))?;
        
        Ok(())
    }
    
    // 记录管理员操作日志
    pub async fn log_admin_operation(
        &self,
        admin_id: i64,
        operation_type: &str,
        operation_target: Option<&str>,
        target_id: Option<&str>,
        operation_content: Option<&str>,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
    ) -> Result<AdminOperationLog, DbError> {
        let log = sqlx::query_as::<_, AdminOperationLog>(
            r#"INSERT INTO admin_operation_logs 
               (admin_id, operation_type, operation_target, target_id, operation_content, ip_address, user_agent) 
               VALUES ($1, $2, $3, $4, $5, $6, $7) 
               RETURNING *"#
        )
        .bind(admin_id)
        .bind(operation_type)
        .bind(operation_target)
        .bind(target_id)
        .bind(operation_content)
        .bind(ip_address)
        .bind(user_agent)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::QueryError(format!("Failed to log admin operation: {}", e)))?;
        
        Ok(log)
    }
    
    // 获取管理员列表
    pub async fn get_admins(
        &self,
        page: i64,
        page_size: i64,
        role: Option<&str>,
    ) -> Result<Vec<Admin>, DbError> {
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
        
        let admins = query
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DbError::QueryError(format!("Failed to get admins: {}", e)))?;
        
        Ok(admins)
    }
    
    // 获取管理员总数
    pub async fn get_admins_count(&self, role: Option<&str>) -> Result<i64, DbError> {
        let mut query_builder = sqlx::QueryBuilder::new("SELECT COUNT(*) FROM admins");
        
        if let Some(role) = role {
            query_builder.push(" WHERE role = ");
            query_builder.push_bind(role);
        }
        
        let query = query_builder.build_query_as::<(i64,)>();
        
        let (count,) = query
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DbError::QueryError(format!("Failed to get admins count: {}", e)))?;
        
        Ok(count)
    }
}

impl AdminRepository {
    // 验证管理员登录凭据（使用Argon2）
    pub async fn verify_admin_credentials(
        &self,
        email: &str,
        password: &str,
    ) -> Result<Option<Admin>, DbError> {
        let admin = self.get_admin_by_email(email).await?;
        
        if let Some(admin) = admin {
            let is_valid = PasswordService::verify_password(password, &admin.password_hash)
                .map_err(|e| DbError::HashError(format!("Password verification failed: {}", e)))?;
                
            if is_valid && admin.is_active {
                Ok(Some(admin))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }
    
    // 创建管理员（使用雪花ID和Argon2）
    pub async fn create_admin(
        &self,
        email: &str,
        password: &str,
        username: &str,
        role: &str,
    ) -> Result<Admin, DbError> {
        let admin_id = snowflake::next_id();
        let password_hash = PasswordService::hash_password(password)
            .map_err(|e| DbError::HashError(format!("Failed to hash password: {}", e)))?;
            
        let admin = sqlx::query_as::<_, Admin>(
            r#"
            INSERT INTO admins (id, email, password_hash, username, role, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, TRUE, NOW(), NOW())
            RETURNING *
            "#
        )
        .bind(admin_id)
        .bind(email)
        .bind(password_hash)
        .bind(username)
        .bind(role)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::QueryError(format!("Failed to create admin: {}", e)))?;
        
        Ok(admin)
    }
    
    // 记录管理员操作日志（使用雪花ID）
    pub async fn log_admin_operation(
        &self,
        admin_id: i64,
        operation_type: &str,
        operation_target: Option<&str>,
        target_id: Option<&str>,
        operation_content: Option<&str>,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
    ) -> Result<(), DbError> {
        let log_id = snowflake::next_id();
        
        sqlx::query(
            r#"
            INSERT INTO admin_operation_logs (id, admin_id, operation_type, operation_target, target_id, operation_content, ip_address, user_agent)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#
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
        .map_err(|e| DbError::QueryError(format!("Failed to log admin operation: {}", e)))?;
        
        Ok(())
    }
}