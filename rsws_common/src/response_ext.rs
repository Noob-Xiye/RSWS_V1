//! Response 扩展 Trait
//!
//! 为 Salvo Response 提供统一的成功/错误响应方法，减少 handler 中的样板代码

use crate::error::RswsError;
use crate::response::ApiResponse;
use salvo::prelude::*;
use serde::Serialize;

/// Response 扩展 Trait
pub trait ResponseExt {
    /// 发送成功响应（带数据）
    fn success<T: Serialize + Send + 'static>(&mut self, data: T);

    /// 发送成功响应（带数据和自定义消息）
    fn success_msg<T: Serialize + Send + 'static>(&mut self, data: T, msg: impl Into<String>);

    /// 发送成功响应（无数据）
    fn ok(&mut self);

    /// 发送错误响应
    fn error(&mut self, err: RswsError);

    /// 发送错误响应（带自定义消息）
    fn error_msg(&mut self, err: RswsError, msg: impl Into<String>);

    /// 发送 HTTP 错误状态码 + 错误消息
    fn http_error(&mut self, status: salvo::http::StatusCode, msg: impl Into<String>);
}

impl ResponseExt for Response {
    fn success<T: Serialize + Send + 'static>(&mut self, data: T) {
        let code = crate::ErrorCode::SUCCESS;
        let status = salvo::http::StatusCode::from_u16(code.http_status())
            .unwrap_or(salvo::http::StatusCode::OK);
        self.status_code(status);
        self.render(Json(ApiResponse::success(data)));
    }

    fn success_msg<T: Serialize + Send + 'static>(&mut self, data: T, msg: impl Into<String>) {
        let code = crate::ErrorCode::SUCCESS;
        let status = salvo::http::StatusCode::from_u16(code.http_status())
            .unwrap_or(salvo::http::StatusCode::OK);
        self.status_code(status);
        self.render(Json(ApiResponse::success_with_message(data, msg)));
    }

    fn ok(&mut self) {
        self.success(());
    }

    fn error(&mut self, err: RswsError) {
        let code = err.error_code();
        let status = salvo::http::StatusCode::from_u16(code.http_status())
            .unwrap_or(salvo::http::StatusCode::INTERNAL_SERVER_ERROR);
        self.status_code(status);
        self.render(Json(ApiResponse::<()>::error_with_message(
            code,
            err.to_string(),
        )));
    }

    fn error_msg(&mut self, err: RswsError, msg: impl Into<String>) {
        let code = err.error_code();
        let status = salvo::http::StatusCode::from_u16(code.http_status())
            .unwrap_or(salvo::http::StatusCode::INTERNAL_SERVER_ERROR);
        self.status_code(status);
        self.render(Json(ApiResponse::<()>::error_with_message(code, msg)));
    }

    fn http_error(&mut self, status: salvo::http::StatusCode, msg: impl Into<String>) {
        self.status_code(status);
        self.render(Json(ApiResponse::<()>::error_with_message(
            crate::ErrorCode::from_status(status),
            msg,
        )));
    }
}
