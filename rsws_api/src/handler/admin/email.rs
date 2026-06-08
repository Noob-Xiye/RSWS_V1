//! 邮件配置管理
//!
//! 获取、更新邮件（SMTP）配置

use crate::state::get_state;
use rsws_common::{ResponseExt, RswsError};
use salvo::prelude::*;
use salvo_oapi::endpoint;
use serde::Deserialize;
use sqlx;

/// 更新邮件配置请求
#[derive(Debug, Deserialize, salvo_oapi::ToSchema)]
pub struct UpdateEmailConfigBody {
    pub provider: Option<String>,
    pub host: Option<String>,
    pub port: Option<i32>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub use_tls: Option<bool>,
    pub from_email: Option<String>,
    pub from_name: Option<String>,
    pub reply_to: Option<String>,
}

/// 获取邮件配置（公开配置，不含密码）
#[endpoint(
    responses(
        (status_code = 200, description = "成功"),
        (status_code = 401, description = "未认证"),
    )
)]
pub async fn get_email_config(_req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let state = get_state(depot);
    match state.config_service.get_email_config().await {
        Ok(Some(cfg)) => {
            // 不返回密码
            res.success(serde_json::json!({
                "provider": cfg.provider,
                "host": cfg.host,
                "port": cfg.port,
                "username": cfg.username,
                "use_tls": cfg.use_tls,
                "from_email": cfg.from_email,
                "from_name": cfg.from_name,
                "reply_to": cfg.reply_to,
            }))
        }
        Ok(None) => res.success(()), // 未配置
        Err(e) => res.error(e),
    }
}

/// 更新邮件配置（upsert，仅一个活跃配置）
#[endpoint(
    request_body = UpdateEmailConfigBody,
    responses(
        (status_code = 200, description = "更新成功"),
        (status_code = 401, description = "未认证"),
    )
)]
pub async fn update_email_config(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let body: Result<UpdateEmailConfigBody, _> = req.parse_json().await;
    let data = match body {
        Ok(d) => d,
        Err(e) => {
            res.http_error(StatusCode::BAD_REQUEST, format!("Invalid request: {}", e));
            return;
        }
    };

    let state = get_state(depot);

    // 先停用所有现有配置
    let _ = sqlx::query("UPDATE email_configs SET is_active = false")
        .execute(&state.pool)
        .await;

    // 检查是否有活跃配置
    let existing: Option<(i64,)> =
        sqlx::query_as("SELECT id FROM email_configs WHERE is_active = true LIMIT 1")
            .fetch_optional(&state.pool)
            .await
            .ok()
            .flatten();

    let password = match &data.password {
        Some(pwd) => pwd.clone(), // TODO: 实际应使用加密，暂明文存储
        None => {
            // 保留现有密码
            match &existing {
                Some((id,)) => {
                    let row: Option<(String,)> =
                        sqlx::query_as("SELECT password FROM email_configs WHERE id = $1")
                            .bind(id)
                            .fetch_optional(&state.pool)
                            .await
                            .ok()
                            .flatten();
                    row.map(|r| r.0).unwrap_or_default()
                }
                None => String::new(),
            }
        }
    };

    let result = if let Some((id,)) = existing {
        // 更新现有
        let mut q = sqlx::QueryBuilder::new("UPDATE email_configs SET ");
        let mut sep = q.separated(", ");
        if let Some(v) = &data.provider {
            sep.push("provider = ").push_bind(v);
        }
        if let Some(v) = &data.host {
            sep.push("host = ").push_bind(v);
        }
        if let Some(v) = &data.port {
            sep.push("port = ").push_bind(v);
        }
        if let Some(v) = &data.username {
            sep.push("username = ").push_bind(v);
        }
        if data.password.is_some() {
            sep.push("password = ").push_bind(&password);
        }
        if let Some(v) = &data.use_tls {
            sep.push("use_tls = ").push_bind(v);
        }
        if let Some(v) = &data.from_email {
            sep.push("from_email = ").push_bind(v);
        }
        if let Some(v) = &data.from_name {
            sep.push("from_name = ").push_bind(v);
        }
        if let Some(v) = &data.reply_to {
            sep.push("reply_to = ").push_bind(v);
        }
        sep.push("updated_at = NOW()");
        q.push(" WHERE id = ").push_bind(id);
        q.build().execute(&state.pool).await
    } else {
        // 插入新配置
        sqlx::query(
            r#"
            INSERT INTO email_configs (provider, host, port, username, password, use_tls, from_email, from_name, reply_to, is_active)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, true)
            "#
        )
        .bind(data.provider.as_deref().unwrap_or("smtp"))
        .bind(data.host.as_deref().unwrap_or(""))
        .bind(data.port.unwrap_or(465))
        .bind(data.username.as_deref().unwrap_or(""))
        .bind(&password)
        .bind(data.use_tls.unwrap_or(true))
        .bind(data.from_email.as_deref().unwrap_or(""))
        .bind(data.from_name.as_deref().unwrap_or(""))
        .bind(data.reply_to.as_deref().unwrap_or(""))
        .execute(&state.pool)
        .await
    };

    match result {
        Ok(_) => {
            // 重新加载配置
            let _ = state.config_service.get_email_config().await;
            res.success(serde_json::json!({"success": true}))
        }
        Err(e) => res.error(RswsError::internal(format!(
            "Failed to update email config: {}",
            e
        ))),
    }
}
