//! 管理员 API Key 管理
//!
//! 创建、列表、删除、切换状态

use crate::state::{get_state, require_user_id};
use rsws_common::ResponseExt;
use rsws_model::api_key::CreateApiKeyRequest;
use salvo::http::StatusCode;
use salvo::prelude::*;
use salvo_oapi::endpoint;
use serde::Deserialize;

/// 创建管理员 API Key 请求
#[derive(Debug, Deserialize, salvo_oapi::ToSchema)]
pub struct CreateAdminApiKeyBody {
    pub name: String,
    pub permissions: Vec<String>,
    pub rate_limit: Option<i32>,
    pub expires_in_days: Option<i32>,
}

/// 创建管理员 API Key
#[endpoint(
    request_body = CreateAdminApiKeyBody,
    responses(
        (status_code = 201, description = "创建成功"),
        (status_code = 403, description = "非管理员"),
    )
)]
pub async fn create_api_key(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let admin_id = match require_user_id(depot) {
        Ok(id) => id,
        Err(status) => {
            res.status_code(status);
            return;
        }
    };

    let body: Result<CreateAdminApiKeyBody, _> = req.parse_json().await;

    match body {
        Ok(data) => {
            let state = get_state(depot);
            let create_req = CreateApiKeyRequest {
                name: data.name,
                permissions: data.permissions,
                rate_limit: data.rate_limit,
                expires_in_days: data.expires_in_days,
            };
            match state
                .admin_api_key_manager
                .create(admin_id, create_req)
                .await
            {
                Ok(response) => {
                    res.status_code(StatusCode::CREATED);
                    res.success(response);
                }
                Err(e) => res.error(e),
            }
        }
        Err(e) => {
            res.http_error(StatusCode::BAD_REQUEST, format!("Invalid request: {}", e));
        }
    }
}

/// 获取管理员的 API Key 列表
#[endpoint(
    responses(
        (status_code = 200, description = "成功"),
        (status_code = 403, description = "非管理员"),
    )
)]
pub async fn list_api_keys(_req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let admin_id = match require_user_id(depot) {
        Ok(id) => id,
        Err(status) => {
            res.status_code(status);
            return;
        }
    };

    let state = get_state(depot);
    match state.admin_api_key_manager.get(admin_id).await {
        Ok(Some(key)) => res.success(vec![key]),
        Ok(None) => res.success(Vec::<rsws_model::api_key::ApiKey>::new()),
        Err(e) => res.error(e),
    }
}

/// 删除管理员 API Key
#[endpoint(
    responses(
        (status_code = 200, description = "删除成功"),
        (status_code = 403, description = "非管理员"),
    )
)]
pub async fn delete_api_key(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let admin_id = match require_user_id(depot) {
        Ok(id) => id,
        Err(status) => {
            res.status_code(status);
            return;
        }
    };

    let _key_id: i64 = req.param("key_id").unwrap_or(0);

    let state = get_state(depot);
    match state.admin_api_key_manager.delete(admin_id).await {
        Ok(deleted) => res.success(serde_json::json!({
            "deleted": deleted
        })),
        Err(e) => res.error(e),
    }
}

/// 切换管理员 API Key 状态请求
#[derive(Debug, Deserialize, salvo_oapi::ToSchema)]
pub struct ToggleApiKeyStatusBody {
    pub is_active: bool,
}

/// 切换管理员 API Key 状态
#[endpoint(
    request_body = ToggleApiKeyStatusBody,
    responses(
        (status_code = 200, description = "切换成功"),
        (status_code = 403, description = "非管理员"),
    )
)]
pub async fn toggle_api_key_status(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let admin_id = match require_user_id(depot) {
        Ok(id) => id,
        Err(status) => {
            res.status_code(status);
            return;
        }
    };

    let _key_id: i64 = req.param("key_id").unwrap_or(0);

    let body: Result<ToggleApiKeyStatusBody, _> = req.parse_json().await;

    match body {
        Ok(data) => {
            let state = get_state(depot);
            match state
                .admin_api_key_manager
                .toggle_status(admin_id, data.is_active)
                .await
            {
                Ok(_) => res.success("API key status updated"),
                Err(e) => res.error(e),
            }
        }
        Err(e) => {
            res.http_error(StatusCode::BAD_REQUEST, format!("Invalid request: {}", e));
        }
    }
}
