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

pub mod auth_handler; // 认证辅助 trait
pub mod config;
pub mod email;
pub mod encryption;
pub mod error;
pub mod error_code;
pub mod password;
pub mod response;
pub mod response_ext; // Response 扩展 trait
pub mod signature;
pub mod snowflake;
pub mod utils;

// 重新导出核心类型
pub use auth_handler::AuthHandler;
pub use error::{ApiError, DbError, RswsError, RswsResult, ServiceError};
pub use error_code::ErrorCode;
pub use response::{ApiResponse, ListData, ListResponse, PaginatedData, PaginatedResponse};
pub use response_ext::ResponseExt;
