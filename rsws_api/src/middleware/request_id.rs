//! Request ID 中间件
//!
//! 为每个请求生成唯一的 request_id，用于日志追踪和分布式追踪。
//!
//! 行为:
//! 1. 如果请求头中已有 `X-Request-ID`，沿用该值
//! 2. 否则生成新的 UUID v4
//! 3. 将 request_id 注入到响应头和 tracing span 中

use salvo::prelude::*;
use tracing::Span;
use uuid::Uuid;

/// Request ID 请求头名称
pub const REQUEST_ID_HEADER: &str = "X-Request-ID";

/// Request ID 中间件
///
/// 用法:
/// ```rust
/// let router = Router::new()
///     .hoop(request_id_middleware)
///     .push(...);
/// ```
#[handler]
pub async fn request_id_middleware(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    // 尝试从请求头获取已有的 request_id
    let rid = req
        .header::<String>(REQUEST_ID_HEADER)
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    // 存入 depot 供后续 handler 使用
    depot.insert("request_id", rid.clone());

    // 添加到响应头
    res.add_header(REQUEST_ID_HEADER, &rid, true).ok();

    // 注入到 tracing span（当前 span）
    Span::current().record("request_id", &rid);
}

/// 从 depot 获取 request_id
pub fn get_request_id(depot: &Depot) -> Option<&str> {
    depot.get::<String>("request_id").ok().map(|s| s.as_str())
}
