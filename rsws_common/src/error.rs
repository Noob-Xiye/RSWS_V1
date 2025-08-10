use redis::RedisError;
use sqlx::Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CommonError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Email error: {0}")]
    Email(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Not found")]
    NotFound,

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Internal server error: {0}")]
    InternalServerError(String),

    #[error("Hash error: {0}")]
    HashError(String),

    #[error("Signature error: {0}")]
    SignatureError(String),
}

// 为数据库操作提供别名
pub type DbError = CommonError;

// 为服务层提供别名
pub type ServiceError = CommonError;
