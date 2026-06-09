//! Tracing 中间件
//!
//! 基于 tracing crate 的请求日志中间件。

use crate::middleware::request_id::get_request_id;
use salvo::prelude::*;
use std::time::Instant;

#[handler]
pub async fn tracing_logger(
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
    ctrl: &mut FlowCtrl,
) {
    let start = Instant::now();
    ctrl.call_next(req, depot, res).await;

    let request_id = get_request_id(depot).unwrap_or("unknown");
    let method = req.method().as_str();
    let path = req.uri().path();
    let duration_ms = start.elapsed().as_millis() as u64;
    let status = res.status_code.unwrap_or(StatusCode::OK);
    let status_u16 = status.as_u16();

    let client_ip = match req.remote_addr() {
        salvo::conn::SocketAddr::IPv4(v4) => v4.ip().to_string(),
        salvo::conn::SocketAddr::IPv6(v6) => v6.ip().to_string(),
        _ => "unknown".to_string(),
    };

    let user_agent = req
        .headers()
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("-")
        .to_string();

    let _user_id: Option<i64> = depot.get::<i64>("user_id").ok().copied();
    let _is_admin: bool = depot.get::<bool>("is_admin").ok().copied().unwrap_or(false);

    if status_u16 >= 500 {
        tracing::error!(
            target: "rsws_http",
            request_id = %request_id,
            method = %method,
            path = %path,
            status = status_u16,
            duration_ms = duration_ms,
            client_ip = %client_ip,
            user_agent = %user_agent,
            "HTTP {} {} {status} ({duration_ms}ms)",
            method, path,
        );
    } else if status_u16 >= 400 {
        tracing::warn!(
            target: "rsws_http",
            request_id = %request_id,
            method = %method,
            path = %path,
            status = status_u16,
            duration_ms = duration_ms,
            client_ip = %client_ip,
            user_agent = %user_agent,
            "HTTP {} {} {status} ({duration_ms}ms)",
            method, path,
        );
    } else {
        tracing::info!(
            target: "rsws_http",
            request_id = %request_id,
            method = %method,
            path = %path,
            status = status_u16,
            duration_ms = duration_ms,
            client_ip = %client_ip,
            user_agent = %user_agent,
            "HTTP {} {} {status} ({duration_ms}ms)",
            method, path,
        );
    }

    if duration_ms > 1000 {
        tracing::warn!(
            target: "rsws_slow_request",
            request_id = %request_id,
            method = %method,
            path = %path,
            duration_ms = duration_ms,
            "Slow request (>1s)",
        );
    }
}