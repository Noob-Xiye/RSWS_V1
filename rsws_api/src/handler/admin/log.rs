//! 日志配置与查询
//!
//! 日志配置 CRUD + 系统日志查询

use crate::state::get_state;
use chrono::DateTime;
use chrono::Utc;
use rsws_common::{error_code::ErrorCode, ResponseExt, RswsError};
use rsws_service::UpdateLogConfigRequest;
use salvo::prelude::*;
use salvo_oapi::endpoint;
use serde::Deserialize;

/// 获取所有日志配置
#[endpoint(
    responses(
        (status_code = 200, description = "成功"),
        (status_code = 403, description = "非管理员"),
    )
)]
pub async fn list_log_configs(_req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let state = get_state(depot);
    match state.log_service.list_log_configs().await {
        Ok(configs) => res.success(configs),
        Err(e) => res.error(e),
    }
}

/// 获取指定日志配置
#[endpoint(
    responses(
        (status_code = 200, description = "成功"),
        (status_code = 403, description = "非管理员"),
        (status_code = 404, description = "配置不存在"),
    )
)]
pub async fn get_log_config(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let key: String = req.param("key").unwrap_or_default();
    if key.is_empty() {
        res.error_msg(
            RswsError::from(ErrorCode::INVALID_PARAMETER),
            "Config key is required",
        );
        return;
    }

    let state = get_state(depot);
    match state.log_service.get_log_config(&key).await {
        Ok(Some(config)) => res.success(config),
        Ok(None) => res.http_error(StatusCode::NOT_FOUND, "Config not found"),
        Err(e) => res.error(e),
    }
}

/// 创建日志配置请求
#[derive(Debug, Deserialize, salvo_oapi::ToSchema)]
pub struct SetLogConfigBody {
    pub config_key: String,
    pub config_value: String,
    pub config_type: Option<String>,
    pub description: Option<String>,
}

/// 创建日志配置（不存在则创建）
#[endpoint(
    request_body = SetLogConfigBody,
    responses(
        (status_code = 201, description = "创建成功"),
        (status_code = 403, description = "非管理员"),
    )
)]
pub async fn create_log_config(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let body: Result<SetLogConfigBody, _> = req.parse_json().await;
    match body {
        Ok(data) => {
            let state = get_state(depot);
            let config_type = data.config_type.as_deref().unwrap_or("string");
            match state
                .log_service
                .set_log_config(
                    &data.config_key,
                    &data.config_value,
                    config_type,
                    data.description.as_deref(),
                )
                .await
            {
                Ok(config) => {
                    res.status_code(StatusCode::CREATED);
                    res.success(config);
                }
                Err(e) => res.error(e),
            }
        }
        Err(e) => {
            res.http_error(StatusCode::BAD_REQUEST, format!("Invalid request: {}", e));
        }
    }
}

/// 更新日志配置请求
#[derive(Debug, Deserialize, salvo_oapi::ToSchema)]
pub struct UpdateLogConfigBody {
    pub config_value: String,
    pub config_type: Option<String>,
    pub description: Option<String>,
    pub is_active: Option<bool>,
}

/// 更新日志配置
#[endpoint(
    request_body = UpdateLogConfigBody,
    responses(
        (status_code = 200, description = "更新成功"),
        (status_code = 403, description = "非管理员"),
        (status_code = 404, description = "配置不存在"),
    )
)]
pub async fn update_log_config(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let key: String = req.param("key").unwrap_or_default();
    if key.is_empty() {
        res.error_msg(
            RswsError::from(ErrorCode::INVALID_PARAMETER),
            "Config key is required",
        );
        return;
    }

    let body: Result<UpdateLogConfigBody, _> = req.parse_json().await;
    match body {
        Ok(data) => {
            let state = get_state(depot);
            let request = UpdateLogConfigRequest {
                config_key: key,
                config_value: data.config_value,
                config_type: data.config_type,
                description: data.description,
                is_active: data.is_active,
            };
            match state.log_service.update_log_config(&request).await {
                Ok(config) => res.success(config),
                Err(e) => res.error(e),
            }
        }
        Err(e) => {
            res.http_error(StatusCode::BAD_REQUEST, format!("Invalid request: {}", e));
        }
    }
}

/// 删除日志配置
#[endpoint(
    responses(
        (status_code = 200, description = "删除成功"),
        (status_code = 403, description = "非管理员"),
    )
)]
pub async fn delete_log_config(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let key: String = req.param("key").unwrap_or_default();
    if key.is_empty() {
        res.error_msg(
            RswsError::from(ErrorCode::INVALID_PARAMETER),
            "Config key is required",
        );
        return;
    }

    let state = get_state(depot);
    match state.log_service.delete_log_config(&key).await {
        Ok(deleted) => res.success(serde_json::json!({
            "deleted": deleted
        })),
        Err(e) => res.error(e),
    }
}

/// 查询系统日志
#[endpoint(
    parameters(
        ("level", Query, description = "日志级别筛选"),
        ("page", Query, description = "页码"),
        ("page_size", Query, description = "每页数量"),
    ),
    responses(
        (status_code = 200, description = "成功"),
        (status_code = 403, description = "非管理员"),
    )
)]
pub async fn query_system_logs(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let level: Option<String> = req.query("level");
    let module: Option<String> = req.query("module");
    let user_id: Option<i64> = req.query("user_id");
    let start_time: Option<String> = req.query("start_time");
    let end_time: Option<String> = req.query("end_time");
    let page: i64 = req.query("page").unwrap_or(1);
    let page_size: i64 = req.query("page_size").unwrap_or(20);

    // 解析时间参数
    let start_time_dt: Option<DateTime<Utc>> = start_time
        .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
        .map(|dt| dt.with_timezone(&Utc));

    let end_time_dt: Option<DateTime<Utc>> = end_time
        .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
        .map(|dt| dt.with_timezone(&Utc));

    let state = get_state(depot);
    let params = rsws_service::log_service::LogQueryParams {
        level,
        module,
        user_id,
        start_time: start_time_dt,
        end_time: end_time_dt,
    };
    match state
        .log_service
        .query_system_logs(params, page, page_size)
        .await
    {
        Ok((logs, total)) => {
            let page_count = (total + page_size - 1) / page_size;
            res.success(serde_json::json!({
                "list": logs,
                "total": total,
                "page": page,
                "page_size": page_size,
                "page_count": page_count
            }));
        }
        Err(e) => {
            res.error(e);
        }
    }
}
