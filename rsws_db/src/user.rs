//! 用户仓储层

use rsws_common::error::RswsError;
use rsws_common::error_code::ErrorCode;
use rsws_model::user_models::user::User;
use sqlx::PgPool;

/// 用户仓储
pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    /// 创建用户仓储实例
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 创建用户
    pub async fn create_user(
        &self,
        username: &str,
        nickname: &str,
        email: &str,
        password_hash: &str,
    ) -> Result<User, RswsError> {
        let user_id = rsws_common::snowflake::next_id();

        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (id, username, nickname, email, password_hash, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, true, NOW(), NOW())
            RETURNING *
            "#,
        )
        .bind(user_id)
        .bind(username)
        .bind(nickname)
        .bind(email)
        .bind(password_hash)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to create user: {}", e)))?;

        Ok(user)
    }

    /// 根据用户名查找用户
    pub async fn find_user_by_username(&self, username: &str) -> Result<Option<User>, RswsError> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = $1")
            .bind(username)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| RswsError::internal(format!("Failed to find user by username: {}", e)))?;

        Ok(user)
    }

    /// 根据邮箱查找用户
    pub async fn find_user_by_email(&self, email: &str) -> Result<Option<User>, RswsError> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
            .bind(email)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| RswsError::internal(format!("Failed to find user by email: {}", e)))?;

        Ok(user)
    }

    /// 根据 ID 查找用户
    pub async fn find_user_by_id(&self, user_id: i64) -> Result<Option<User>, RswsError> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| RswsError::internal(format!("Failed to find user by id: {}", e)))?;

        Ok(user)
    }

    /// 更新用户昵称
    pub async fn update_user_nickname(
        &self,
        user_id: i64,
        nickname: &str,
    ) -> Result<User, RswsError> {
        let user = sqlx::query_as::<_, User>(
            "UPDATE users SET nickname = $1, updated_at = NOW() WHERE id = $2 RETURNING *"
        )
        .bind(nickname)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to update nickname: {}", e)))?;

        Ok(user)
    }

    /// 更新用户密码
    pub async fn update_user_password(
        &self,
        user_id: i64,
        password_hash: &str,
    ) -> Result<(), RswsError> {
        sqlx::query("UPDATE users SET password_hash = $1, updated_at = NOW() WHERE id = $2")
            .bind(password_hash)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(|e| RswsError::internal(format!("Failed to update password: {}", e)))?;

        Ok(())
    }

    /// 更新用户激活状态
    pub async fn update_user_active(&self, user_id: i64, is_active: bool) -> Result<(), RswsError> {
        sqlx::query("UPDATE users SET is_active = $1, updated_at = NOW() WHERE id = $2")
            .bind(is_active)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(|e| RswsError::internal(format!("Failed to update user active status: {}", e)))?;

        Ok(())
    }

    /// 更新用户资料
    pub async fn update_user_profile(
        &self,
        user_id: i64,
        nickname: Option<&str>,
        avatar_url: Option<&str>,
    ) -> Result<User, RswsError> {
        match (nickname, avatar_url) {
            (Some(nick), Some(url)) => {
                let user = sqlx::query_as::<_, User>(
                    "UPDATE users SET nickname = $1, avatar_url = $2, updated_at = NOW() WHERE id = $3 RETURNING *"
                )
                .bind(nick)
                .bind(url)
                .bind(user_id)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| RswsError::internal(format!("Failed to update profile: {}", e)))?;
                Ok(user)
            }
            (Some(nick), None) => {
                let user = sqlx::query_as::<_, User>(
                    "UPDATE users SET nickname = $1, updated_at = NOW() WHERE id = $2 RETURNING *"
                )
                .bind(nick)
                .bind(user_id)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| RswsError::internal(format!("Failed to update profile: {}", e)))?;
                Ok(user)
            }
            (None, Some(url)) => {
                let user = sqlx::query_as::<_, User>(
                    "UPDATE users SET avatar_url = $1, updated_at = NOW() WHERE id = $2 RETURNING *"
                )
                .bind(url)
                .bind(user_id)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| RswsError::internal(format!("Failed to update profile: {}", e)))?;
                Ok(user)
            }
            (None, None) => {
                self.find_user_by_id(user_id)
                    .await?
                    .ok_or_else(|| RswsError::business(ErrorCode::USER_NOT_FOUND))
            }
        }
    }

    /// 分页获取用户列表（管理员用）
    pub async fn get_users(
        &self,
        page: i64,
        page_size: i64,
        email_filter: Option<&str>,
        username_filter: Option<&str>,
        is_active_filter: Option<bool>,
    ) -> Result<Vec<User>, RswsError> {
        let mut query_builder = sqlx::QueryBuilder::new("SELECT * FROM users WHERE 1=1 ");

        if let Some(email) = email_filter {
            query_builder.push(" AND email ILIKE ");
            query_builder.push_bind(format!("%{}%", email));
        }
        if let Some(username) = username_filter {
            query_builder.push(" AND username ILIKE ");
            query_builder.push_bind(format!("%{}%", username));
        }
        if let Some(is_active) = is_active_filter {
            query_builder.push(" AND is_active = ");
            query_builder.push_bind(is_active);
        }

        query_builder.push(" ORDER BY id DESC LIMIT ");
        query_builder.push_bind(page_size);
        query_builder.push(" OFFSET ");
        query_builder.push_bind((page.saturating_sub(1)).saturating_mul(page_size));

        let query = query_builder.build_query_as::<User>();
        query
            .fetch_all(&self.pool)
            .await
            .map_err(|e| RswsError::internal(format!("Failed to get users: {}", e)))
    }

    /// 获取用户总数
    pub async fn get_users_count(
        &self,
        email_filter: Option<&str>,
        username_filter: Option<&str>,
        is_active_filter: Option<bool>,
    ) -> Result<i64, RswsError> {
        let mut query_builder = sqlx::QueryBuilder::new("SELECT COUNT(*) FROM users WHERE 1=1 ");

        if let Some(email) = email_filter {
            query_builder.push(" AND email ILIKE ");
            query_builder.push_bind(format!("%{}%", email));
        }
        if let Some(username) = username_filter {
            query_builder.push(" AND username ILIKE ");
            query_builder.push_bind(format!("%{}%", username));
        }
        if let Some(is_active) = is_active_filter {
            query_builder.push(" AND is_active = ");
            query_builder.push_bind(is_active);
        }

        let count: (i64,) = query_builder
            .build_query_as()
            .fetch_one(&self.pool)
            .await
            .map_err(|e| RswsError::internal(format!("Failed to count users: {}", e)))?;
        Ok(count.0)
    }

    /// 获取基础统计（用户总数 + 过去30天新增用户数）
    pub async fn get_basic_stats(&self) -> Result<(i64, i64), RswsError> {
        let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| RswsError::internal(format!("Failed to count users: {}", e)))?;

        let new_users_30d: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM users WHERE created_at >= NOW() - INTERVAL '30 days'"
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to count new users: {}", e)))?;

        Ok((total.0, new_users_30d.0))
    }
}

// ==================== 单元测试 ====================

#[cfg(test)]
mod tests {
    #[test]
    fn test_user_repository_new() {
        // 仅测试构造函数
        // 实际数据库测试需要 test container
    }
}
