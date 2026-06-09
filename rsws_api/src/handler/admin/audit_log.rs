//! Admin audit log handlers

use crate::state::get_state;
use rsws_service::audit_log_service::{AuditLogQuery, ResourceType};
use salvo::prelude::*;
use salvo_oapi::endpoint;

/// List audit logs
#[endpoint]
pub async fn list_audit_logs(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let state = get_state(depot);

    let query = AuditLogQuery {
        user_id: None,
        admin_id: req.query("admin_id"),
        action: req.query("action"),
        resource_type: req.query("resource_type"),
        resource_id: req.query("resource_id"),
        risk_level: req.query("risk_level"),
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

    match state.audit_log_service.query(query).await {
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

/// Get audit statistics
#[endpoint]
pub async fn get_audit_stats(depot: &mut Depot, res: &mut Response) {
    let state = get_state(depot);

    match state.audit_log_service.get_stats(24).await {
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

/// Get resource change history
#[endpoint]
pub async fn get_resource_history(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let state = get_state(depot);

    let resource_type_str: Option<String> = req.query("resource_type");
    let resource_id: Option<i64> = req.query("resource_id");
    let limit: i64 = req.query("limit").unwrap_or(50);

    let (rt, ri) = match (resource_type_str, resource_id) {
        (Some(t), Some(id)) => (t, id),
        _ => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(serde_json::json!({
                "code": 400,
                "message": "resource_type and resource_id are required",
                "data": null
            })));
            return;
        }
    };

    let resource_type = match rt.as_str() {
        "user" => ResourceType::User,
        "admin" => ResourceType::Admin,
        "order" => ResourceType::Order,
        "wallet" => ResourceType::Wallet,
        "resource" => ResourceType::Resource,
        "config" => ResourceType::Config,
        "api_key" => ResourceType::ApiKey,
        "system" => ResourceType::System,
        _ => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(serde_json::json!({
                "code": 400,
                "message": format!("Invalid resource_type: {}", rt),
                "data": null
            })));
            return;
        }
    };

    match state
        .audit_log_service
        .get_resource_history(resource_type, ri, limit)
        .await
    {
        Ok(logs) => {
            res.render(Json(serde_json::json!({
                "code": 200,
                "message": "Success",
                "data": { "items": logs }
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
