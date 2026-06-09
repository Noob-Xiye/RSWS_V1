//! Admin login log handlers

use crate::state::get_state;
use rsws_service::login_log_service::LoginLogQuery;
use salvo::prelude::*;
use salvo_oapi::endpoint;

/// List login logs
#[endpoint]
pub async fn list_login_logs(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let state = get_state(depot);

    let query = LoginLogQuery {
        user_id: req.query("user_id"),
        status: req.query("status"),
        login_type: req.query("login_type"),
        ip_address: req.query("ip_address"),
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

    match state.login_log_service.query_logs(query).await {
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

/// Get login statistics
#[endpoint]
pub async fn get_login_stats(_depot: &mut Depot, res: &mut Response) {
    res.render(Json(serde_json::json!({
        "code": 200,
        "message": "Success",
        "data": { "status": "operational" }
    })));
}
