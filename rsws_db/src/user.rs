use bcrypt::{hash, DEFAULT_COST};
use rand::Rng;
use rsws_common::error::DbError;
use rsws_common::password::PasswordService;
use rsws_common::snowflake;
use rsws_model::user::*;
use sqlx::PgPool;

pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_user(
        &self,
        username: &str,
        email: &str,
        password: &str,
    ) -> Result<User, DbError> {
        let user_id = snowflake::generate_id();
        let password_hash = hash(password, DEFAULT_COST)
            .map_err(|e| DbError::InternalServerError(format!("Password hash failed: {}", e)))?;

        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (id, username, email, password_hash, created_at, updated_at)
            VALUES ($1, $2, $3, $4, NOW(), NOW())
            RETURNING *
            "#,
        )
        .bind(user_id)
        .bind(username)
        .bind(email)
        .bind(password_hash)
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn find_user_by_email(&self, email: &str) -> Result<Option<User>, DbError> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
            .bind(email)
            .fetch_optional(&self.pool)
            .await?;

        Ok(user)
    }

    pub async fn find_user_by_id(&self, user_id: i64) -> Result<Option<User>, DbError> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(user)
    }

    pub async fn update_user_password(
        &self,
        user_id: i64,
        new_password: &str,
    ) -> Result<(), DbError> {
        let password_hash = hash(new_password, DEFAULT_COST)
            .map_err(|e| DbError::InternalServerError(format!("Password hash failed: {}", e)))?;

        sqlx::query("UPDATE users SET password_hash = $1, updated_at = NOW() WHERE id = $2")
            .bind(password_hash)
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn update_user_profile(
        &self,
        user_id: i64,
        username: Option<&str>,
        email: Option<&str>,
    ) -> Result<User, DbError> {
        let mut query = "UPDATE users SET updated_at = NOW()".to_string();
        let mut params: Vec<Box<dyn sqlx::Encode<'_, sqlx::Postgres> + Send + Sync>> = Vec::new();
        let mut param_count = 0;

        if let Some(username) = username {
            param_count += 1;
            query.push_str(&format!(", username = ${}", param_count));
            params.push(Box::new(username.to_string()));
        }

        if let Some(email) = email {
            param_count += 1;
            query.push_str(&format!(", email = ${}", param_count));
            params.push(Box::new(email.to_string()));
        }

        param_count += 1;
        query.push_str(&format!(" WHERE id = ${} RETURNING *", param_count));

        let user = sqlx::query_as::<_, User>(&query)
            .bind(user_id)
            .fetch_one(&self.pool)
            .await?;

        Ok(user)
    }
}
