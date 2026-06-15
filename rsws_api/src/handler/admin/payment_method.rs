//! 支付方式管理
//!
//! 列表、创建/启用、删除/禁用支付方式

use crate::state::get_state;
use rsws_common::snowflake;
use rsws_common::{ResponseExt, RswsError};
use salvo::prelude::*;
use salvo_oapi::endpoint;
use serde::{Deserialize, Serialize};
use sqlx;

/// 支付方式列表项
#[derive(Debug, Serialize, Deserialize, salvo_oapi::ToSchema)]
pub struct PaymentMethodItem {
    pub id: i64,
    pub method_type: String,
    pub method_name: String,
    pub is_enabled: bool,
    pub config: serde_json::Value,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// 获取支付方式列表
#[endpoint(
    responses(
        (status_code = 200, description = "获取成功"),
        (status_code = 401, description = "未认证"),
    )
)]
pub async fn list_payment_methods(_req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let state = get_state(depot);
    let rows = sqlx::query_as::<_, (i64, String, String, bool, serde_json::Value, Option<chrono::DateTime<chrono::Utc>>, Option<chrono::DateTime<chrono::Utc>>)>(
        "SELECT id, method_type, method_name, is_enabled, config, created_at, updated_at FROM payment_methods ORDER BY id"
    )
    .fetch_all(&state.pool)
    .await;
    match rows {
        Ok(rows) => {
            let list: Vec<PaymentMethodItem> = rows
                .into_iter()
                .map(|(id, mt, mn, ie, cfg, ca, ua)| PaymentMethodItem {
                    id,
                    method_type: mt,
                    method_name: mn,
                    is_enabled: ie,
                    config: cfg,
                    created_at: ca,
                    updated_at: ua,
                })
                .collect();
            res.success(serde_json::json!(list))
        }
        Err(e) => res.error(RswsError::internal(format!(
            "Failed to list payment methods: {}",
            e
        ))),
    }
}

/// 创建/启用支付方式请求
#[derive(Debug, Deserialize, salvo_oapi::ToSchema)]
pub struct CreatePaymentMethodBody {
    pub method_type: String,
    pub method_name: String,
    pub is_enabled: Option<bool>,
    pub config: Option<serde_json::Value>,
}

/// 创建或启用支付方式（upsert）
#[endpoint(
    request_body = CreatePaymentMethodBody,
    responses(
        (status_code = 200, description = "创建/更新成功"),
        (status_code = 401, description = "未认证"),
    )
)]
pub async fn create_payment_method(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let body: Result<CreatePaymentMethodBody, _> = req.parse_json().await;
    let data = match body {
        Ok(d) => d,
        Err(e) => {
            res.http_error(
                salvo::http::StatusCode::BAD_REQUEST,
                format!("Invalid request: {}", e),
            );
            return;
        }
    };
    let state = get_state(depot);
    let is_enabled = data.is_enabled.unwrap_or(true);
    let config = data.config.unwrap_or(serde_json::json!({}));
    let id = snowflake::next_id();
    let result = sqlx::query(
        "INSERT INTO payment_methods (id, method_type, method_name, is_enabled, config) VALUES ($1, $2, $3, $4, $5) ON CONFLICT (method_type) DO UPDATE SET is_enabled = EXCLUDED.is_enabled, config = EXCLUDED.config, updated_at = NOW()"
    )
    .bind(id)
    .bind(&data.method_type)
    .bind(&data.method_name)
    .bind(is_enabled)
    .bind(&config)
    .execute(&state.pool)
    .await;
    match result {
        Ok(_) => res.success(serde_json::json!({"success": true})),
        Err(e) => res.error(RswsError::internal(format!(
            "Failed to create payment method: {}",
            e
        ))),
    }
}

/// 删除/禁用支付方式（按 id）
#[endpoint(
    responses(
        (status_code = 200, description = "删除成功"),
        (status_code = 401, description = "未认证"),
        (status_code = 404, description = "不存在"),
    )
)]
pub async fn delete_payment_method(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let state = get_state(depot);
    let id: i64 = match req.param::<i64>("id") {
        Some(v) => v,
        None => {
            res.http_error(salvo::http::StatusCode::BAD_REQUEST, "Invalid id");
            return;
        }
    };
    // 软删除：设为禁用
    let result = sqlx::query(
        "UPDATE payment_methods SET is_enabled = false, updated_at = NOW() WHERE id = $1",
    )
    .bind(id)
    .execute(&state.pool)
    .await;
    match result {
        Ok(r) if r.rows_affected() > 0 => res.success(serde_json::json!({"success": true})),
        Ok(_) => res.error(RswsError::not_found("Payment method not found")),
        Err(e) => res.error(RswsError::internal(format!(
            "Failed to delete payment method: {}",
            e
        ))),
    }
}
