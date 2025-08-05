use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("Configuration error: {0}")]
    Config(#[from] config::ConfigError),

    #[error("Email error: {0}")]
    Email(#[from] lettre::error::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Internal server error: {0}")]
    InternalServerError(String),
}

#[derive(Debug, thiserror::Error)]
pub enum CommonError {
    #[error("Email error: {0}")]
    EmailError(String),
    #[error("Hash error: {0}")]
    HashError(String),
}

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("Authentication error: {0}")]
    AuthError(String),
    #[error("Email error: {0}")]
    EmailError(String),
    #[error("Database error: {0}")]
    DatabaseError(#[from] DbError),
}
