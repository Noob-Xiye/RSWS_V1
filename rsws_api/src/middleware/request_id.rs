//! Request ID 中间件
//!
//! 使用 Salvo 内置 RequestId 中间件（默认 UlidGenerator）。
//! 此文件仅保留便捷访问函数和常量定义。

use salvo::prelude::*;

/// Request ID 请求头名称（与 Salvo 内置一致）
pub const REQUEST_ID_HEADER: &str = "x-request-id";

/// 从请求头获取 request_id（Salvo 内置 RequestId 中间件自动注入）
pub fn get_request_id(req: &Request) -> Option<String> {
    req.header::<String>(REQUEST_ID_HEADER)
}
