//! RSWS 核心错误类型
//!
//! 统一的错误处理体系，支持错误码转换

use thiserror::Error;
use super::error_code::ErrorCode;

/// RSWS 核心错误类型
#[derive(Error, Debug)]
pub enum RswsError {
    // ==================== 系统错误 ====================
    #[error("{0}")]
    BadRequest(String),

    #[error("{0}")]
    Unauthorized(String),

    #[error("{0}")]
    Forbidden(String),

    #[error("{0}")]
    NotFound(String),

    #[error("{0}")]
    Conflict(String),

    #[error("{0}")]
    InternalError(String),

    #[error("{0}")]
    Timeout(String),

    #[error("{0}")]
    RateLimited(String),

    // ==================== 业务错误 ====================
    #[error("Error code {0}: {1}")]
    Business(ErrorCode, String),

    // ==================== 外部错误 ====================
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("HTTP error: {0}")]
    Http(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl RswsError {
    /// 转换为错误码
    pub fn error_code(&self) -> ErrorCode {
        match self {
            Self::BadRequest(_) => ErrorCode::BAD_REQUEST,
            Self::Unauthorized(_) => ErrorCode::UNAUTHORIZED,
            Self::Forbidden(_) => ErrorCode::FORBIDDEN,
            Self::NotFound(_) => ErrorCode::NOT_FOUND,
            Self::Conflict(_) => ErrorCode::CONFLICT,
            Self::InternalError(_) => ErrorCode::INTERNAL_ERROR,
            Self::Timeout(_) => ErrorCode::TIMEOUT,
            Self::RateLimited(_) => ErrorCode::RATE_LIMIT_EXCEEDED,
            Self::Business(code, _) => *code,
            Self::Database(e) => match e {
                sqlx::Error::RowNotFound => ErrorCode::NOT_FOUND,
                _ => ErrorCode::DB_QUERY_FAILED,
            },
            Self::Redis(_) => ErrorCode::CACHE_CONNECTION_FAILED,
            Self::Serialization(_) => ErrorCode::BAD_REQUEST,
            Self::Http(_) => ErrorCode::INTERNAL_ERROR,
            Self::Io(_) => ErrorCode::INTERNAL_ERROR,
        }
    }

    /// 创建业务错误
    pub fn business(code: ErrorCode) -> Self {
        Self::Business(code, code.message().to_string())
    }

    /// 创建业务错误 (带消息)
    pub fn business_with_message(code: ErrorCode, msg: impl Into<String>) -> Self {
        Self::Business(code, msg.into())
    }
}

// ==================== 快捷构造方法 ====================

impl RswsError {
    pub fn bad_request(msg: impl Into<String>) -> Self {
        Self::BadRequest(msg.into())
    }

    pub fn unauthorized(msg: impl Into<String>) -> Self {
        Self::Unauthorized(msg.into())
    }

    pub fn forbidden(msg: impl Into<String>) -> Self {
        Self::Forbidden(msg.into())
    }

    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }

    pub fn conflict(msg: impl Into<String>) -> Self {
        Self::Conflict(msg.into())
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        Self::InternalError(msg.into())
    }
}

// ==================== From 实现 ====================

impl From<ErrorCode> for RswsError {
    fn from(code: ErrorCode) -> Self {
        Self::business(code)
    }
}

impl From<reqwest::Error> for RswsError {
    fn from(err: reqwest::Error) -> Self {
        Self::Http(err.to_string())
    }
}

// ==================== 类型别名 ====================

/// 数据库错误
pub type DbError = RswsError;

/// 服务错误
pub type ServiceError = RswsError;

/// API 错误
pub type ApiError = RswsError;

/// 结果类型
pub type RswsResult<T> = Result<T, RswsError>;
