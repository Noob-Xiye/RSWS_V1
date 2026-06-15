//! RSWS 统一响应格式
//!
//! 所有 API 返回统一的响应结构

use super::error_code::ErrorCode;
use salvo::prelude::*;
use salvo_oapi::ToSchema;
use serde::{Deserialize, Serialize};

/// 统一 API 响应格式
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ApiResponse<T> {
    /// 错误码 (0 = 成功)
    pub code: i32,
    /// 错误消息
    pub msg: String,
    /// 响应数据
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    /// 请求 ID (用于追踪)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

impl<T> ApiResponse<T> {
    /// 成功响应
    pub fn success(data: T) -> Self {
        Self {
            code: ErrorCode::SUCCESS.0,
            msg: ErrorCode::SUCCESS.message().to_string(),
            data: Some(data),
            request_id: None,
        }
    }

    /// 成功响应 (带自定义消息)
    pub fn success_with_message(data: T, msg: impl Into<String>) -> Self {
        Self {
            code: ErrorCode::SUCCESS.0,
            msg: msg.into(),
            data: Some(data),
            request_id: None,
        }
    }

    /// 错误响应
    pub fn error(code: ErrorCode) -> ApiResponse<()> {
        ApiResponse {
            code: code.0,
            msg: code.message().to_string(),
            data: None,
            request_id: None,
        }
    }

    /// 错误响应 (带自定义消息)
    pub fn error_with_message(code: ErrorCode, msg: impl Into<String>) -> ApiResponse<()> {
        ApiResponse {
            code: code.0,
            msg: msg.into(),
            data: None,
            request_id: None,
        }
    }

    /// 设置请求 ID
    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }
}

// ==================== 常用响应快捷方法 ====================

impl ApiResponse<()> {
    /// 成功 (无数据)
    pub fn ok() -> Self {
        Self::success(())
    }

    /// 错误请求
    pub fn bad_request(msg: impl Into<String>) -> Self {
        Self::error_with_message(ErrorCode::BAD_REQUEST, msg)
    }

    /// 未授权
    pub fn unauthorized(msg: impl Into<String>) -> Self {
        Self::error_with_message(ErrorCode::UNAUTHORIZED, msg)
    }

    /// 禁止访问
    pub fn forbidden(msg: impl Into<String>) -> Self {
        Self::error_with_message(ErrorCode::FORBIDDEN, msg)
    }

    /// 未找到
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::error_with_message(ErrorCode::NOT_FOUND, msg)
    }

    /// 内部错误
    pub fn internal_error(msg: impl Into<String>) -> Self {
        Self::error_with_message(ErrorCode::INTERNAL_ERROR, msg)
    }

    /// 服务不可用
    pub fn service_unavailable(msg: impl Into<String>) -> Self {
        Self::error_with_message(ErrorCode::SERVICE_UNAVAILABLE, msg)
    }

    /// 请求超时
    pub fn timeout(msg: impl Into<String>) -> Self {
        Self::error_with_message(ErrorCode::TIMEOUT, msg)
    }

    /// 速率限制
    pub fn rate_limited(msg: impl Into<String>) -> Self {
        Self::error_with_message(ErrorCode::RATE_LIMIT_EXCEEDED, msg)
    }
}

// ==================== 分页响应 ====================

/// 分页数据
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PaginatedData<T> {
    /// 数据列表
    pub items: Vec<T>,
    /// 总数
    pub total: i64,
    /// 当前页
    pub page: i64,
    /// 每页大小
    pub page_size: i64,
    /// 总页数
    pub total_pages: i64,
}

impl<T> PaginatedData<T> {
    pub fn new(items: Vec<T>, total: i64, page: i64, page_size: i64) -> Self {
        let total_pages = if page_size > 0 {
            (total + page_size - 1) / page_size
        } else {
            0
        };
        Self {
            items,
            total,
            page,
            page_size,
            total_pages,
        }
    }
}

/// 分页响应
pub type PaginatedResponse<T> = ApiResponse<PaginatedData<T>>;

// ==================== 列表响应 ====================

/// 列表数据 (无分页)
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ListData<T> {
    /// 数据列表
    pub items: Vec<T>,
    /// 总数
    pub total: i64,
}

impl<T> ListData<T> {
    pub fn new(items: Vec<T>) -> Self {
        let total = items.len() as i64;
        Self { items, total }
    }

    pub fn with_total(items: Vec<T>, total: i64) -> Self {
        Self { items, total }
    }
}

pub type ListResponse<T> = ApiResponse<ListData<T>>;

// ==================== Salvo 集成 ====================

impl<T: Serialize + Send> ApiResponse<T> {
    /// 转换为 Salvo Response
    pub fn into_response(self) -> Response {
        use salvo::prelude::Json;

        let status = salvo::http::StatusCode::from_u16(ErrorCode::from(self.code).http_status())
            .unwrap_or(salvo::http::StatusCode::INTERNAL_SERVER_ERROR);

        let mut res = Response::new();
        res.status_code(status);
        res.render(Json(self));
        res
    }
}

// ==================== 错误转换 ====================

impl From<ErrorCode> for ApiResponse<()> {
    fn from(code: ErrorCode) -> Self {
        Self::error(code)
    }
}

impl From<sqlx::Error> for ApiResponse<()> {
    fn from(err: sqlx::Error) -> Self {
        let code = match err {
            sqlx::Error::RowNotFound => ErrorCode::NOT_FOUND,
            _ => ErrorCode::DB_QUERY_FAILED,
        };
        Self::error_with_message(code, err.to_string())
    }
}
