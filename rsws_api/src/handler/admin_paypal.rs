//! PayPal 配置管理处理器
//!
//! **权限说明：**
//! - 所有 handler 已通过 `require_admin` 中间件保护
//! - handler 内部无需再检查权限

use crate::state::get_state;
use rsws_common::{ResponseExt, RswsError};
use rsws_db::PayPalConfigRepository;
use rsws_model::payment::UpdatePayPalConfigRequest;
use salvo::prelude::*;
use salvo_oapi::endpoint;

/// 获取所有 PayPal 配置
#[endpoint(
    responses(
        (status_code = 200, description = "PayPal 配置列表"),
        (status_code = 401, description = "未授权"),
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn list_paypal_configs(depot: &mut Depot, res: &mut Response) {
    let pool = get_state(depot).pool();
    let repo = PayPalConfigRepository::new(pool);

    match repo.list_all().await {
        Ok(configs) => res.success(configs),
        Err(e) => res.error(e),
    }
}

/// 获取单个 PayPal 配置
#[endpoint(
    responses(
        (status_code = 200, description = "PayPal 配置"),
        (status_code = 404, description = "配置不存在"),
        (status_code = 401, description = "未授权"),
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn get_paypal_config(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let id: i32 = match req.param("id") {
        Some(id) => id,
        None => {
            res.error(RswsError::bad_request("Missing id parameter"));
            return;
        }
    };

    let pool = get_state(depot).pool();
    let repo = PayPalConfigRepository::new(pool);

    match repo.get_by_id(id).await {
        Ok(Some(config)) => res.success(config),
        Ok(None) => res.error(RswsError::not_found("PayPal config not found")),
        Err(e) => res.error(e),
    }
}

/// 更新 PayPal 配置
#[endpoint(
    request_body = UpdatePayPalConfigRequest,
    responses(
        (status_code = 200, description = "更新成功"),
        (status_code = 404, description = "配置不存在"),
        (status_code = 401, description = "未授权"),
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn update_paypal_config(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let id: i32 = match req.param("id") {
        Some(id) => id,
        None => {
            res.error(RswsError::bad_request("Missing id parameter"));
            return;
        }
    };

    let body: UpdatePayPalConfigRequest = match req.parse_json().await {
        Ok(body) => body,
        Err(e) => {
            res.error(RswsError::bad_request(format!(
                "Invalid request body: {}",
                e
            )));
            return;
        }
    };

    let pool = get_state(depot).pool();
    let repo = PayPalConfigRepository::new(pool);

    match repo.update(id, &body).await {
        Ok(config) => res.success(config),
        Err(e) => res.error(e),
    }
}

/// 设置 PayPal 配置激活状态
#[endpoint(
    responses(
        (status_code = 200, description = "设置成功"),
        (status_code = 404, description = "配置不存在"),
        (status_code = 401, description = "未授权"),
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn set_paypal_config_active(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let id: i32 = match req.param("id") {
        Some(id) => id,
        None => {
            res.error(RswsError::bad_request("Missing id parameter"));
            return;
        }
    };

    let is_active: bool = match req.param("active") {
        Some(active) => active,
        None => {
            res.error(RswsError::bad_request("Missing active parameter"));
            return;
        }
    };

    let pool = get_state(depot).pool();
    let repo = PayPalConfigRepository::new(pool);

    match repo.set_active(id, is_active).await {
        Ok(()) => res.success(serde_json::json!({"message": "Updated"})),
        Err(e) => res.error(e),
    }
}
