//! RSWS 核心公共模块
//!
//! 提供所有模块共用的基础功能:
//! - 错误码定义
//! - 错误类型
//! - 响应格式
//! - 配置管理
//! - 加密解密
//! - 签名验证
//! - ID 生成
//! - 工具函数

pub mod error_code;
pub mod error;
pub mod response;
pub mod config;
pub mod email;
pub mod encryption;
pub mod password;
pub mod signature;
pub mod snowflake;
pub mod utils;

// 重新导出核心类型
pub use error_code::ErrorCode;
pub use error::{RswsError, RswsResult, DbError, ServiceError, ApiError};
pub use response::{ApiResponse, PaginatedData, PaginatedResponse, ListData, ListResponse};
