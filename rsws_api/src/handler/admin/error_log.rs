//! Admin error log handlers

use crate::state::get_state;
use rsws_service::error_log_service::{ErrorLogQuery, ResolveErrorRequest};
use salvo::prelude::*;
use salvo_oapi::endpoint;

/// List error logs
#[endpoint]
pub async fn list_error_logs(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let state = get_state(depot);

    let query = ErrorLogQuery {
        error_type: req.query("error_type"),
        resolved: req.query("resolved"),
        user_id: req.query("user_id"),
        request_id: req.query("request_id"),
        from_date: req
            .query::<String>("from_date")
            .and_then(|d| chrono::DateTime::parse_from_rfc3339(&d).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc)),
        to_date: req
            .query::<String>("to_date")
            .and_then(|d| chrono::DateTime::parse_from_rfc3339(&d).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc)),
        page: req.query("page").unwrap_or(1),
        page_size: req.query("page_size").unwrap_or(20),
    };

    match state.error_log_service.query_errors(query).await {
        Ok(page) => {
            res.render(Json(serde_json::json!({
                "code": 200,
                "message": "Success",
                "data": {
                    "items": page.items,
                    "total": page.total,
                    "page": page.page,
                    "page_size": page.page_size,
                }
            })));
        }
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "code": 500,
                "message": e.to_string(),
                "data": null
            })));
        }
    }
}

/// Get error statistics
#[endpoint]
pub async fn get_error_stats(depot: &mut Depot, res: &mut Response) {
    let state = get_state(depot);

    match state.error_log_service.get_stats(24).await {
        Ok(stats) => {
            res.render(Json(serde_json::json!({
                "code": 200,
                "message": "Success",
                "data": stats,
            })));
        }
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "code": 500,
                "message": e.to_string(),
                "data": null
            })));
        }
    }
}

/// Resolve an error
#[endpoint]
pub async fn resolve_error(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let state = get_state(depot);

    let body: serde_json::Value = match req.parse_json().await {
        Ok(b) => b,
        Err(e) => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(serde_json::json!({
                "code": 400,
                "message": format!("Invalid request body: {}", e),
                "data": null
            })));
            return;
        }
    };

    let error_id = body.get("error_id").and_then(|v| v.as_i64()).unwrap_or(0);
    let admin_id = depot.get::<i64>("admin_id").ok().copied().unwrap_or(0);

    match state
        .error_log_service
        .resolve_error(ResolveErrorRequest {
            error_id,
            resolved_by: admin_id,
        })
        .await
    {
        Ok(log) => {
            res.render(Json(serde_json::json!({
                "code": 200,
                "message": "Error resolved",
                "data": log,
            })));
        }
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "code": 500,
                "message": e.to_string(),
                "data": null
            })));
        }
    }
}

/// Get error log details (placeholder)
#[endpoint]
pub async fn get_error_log(depot: &mut Depot, res: &mut Response) {
    let state = get_state(depot);

    match state.error_log_service.get_stats(24).await {
        Ok(stats) => {
            res.render(Json(serde_json::json!({
                "code": 200,
                "message": "Success",
                "data": stats,
            })));
        }
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "code": 500,
                "message": e.to_string(),
                "data": null
            })));
        }
    }
}
