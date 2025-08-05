use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub code: i32,
    pub msg: String,
    pub data: Option<T>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            code: 200,
            msg: "Success".to_string(),
            data: Some(data),
        }
    }

    pub fn success_with_message(data: T, message: &str) -> Self {
        Self {
            code: 200,
            msg: message.to_string(),
            data: Some(data),
        }
    }

    pub fn error(code: i32, message: &str) -> ApiResponse<()> {
        ApiResponse {
            code,
            msg: message.to_string(),
            data: None,
        }
    }

    pub fn bad_request(message: &str) -> ApiResponse<()> {
        Self::error(400, message)
    }

    pub fn unauthorized(message: &str) -> ApiResponse<()> {
        Self::error(401, message)
    }

    pub fn forbidden(message: &str) -> ApiResponse<()> {
        Self::error(403, message)
    }

    pub fn not_found(message: &str) -> ApiResponse<()> {
        Self::error(404, message)
    }

    pub fn internal_error(message: &str) -> ApiResponse<()> {
        Self::error(500, message)
    }
}

// 分页响应
#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedData<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub total_pages: i64,
}

pub type PaginatedResponse<T> = ApiResponse<PaginatedData<T>>;
