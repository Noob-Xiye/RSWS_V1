//! 用户处理器

use salvo::prelude::*;
use rsws_common::response::ApiResponse;
use rsws_common::error::RswsError;
use rsws_model::user::user::{RegisterRequest, LoginRequest};
use serde::Deserialize;

/// 用户路径参数
#[derive(Debug, Deserialize)]
pub struct UserPath {
    pub id: i64,
}

/// 获取用户信息
#[handler]
pub async fn get_user(req: &mut Request, res: &mut Response) {
    let id: i64 = req.param("id").unwrap_or(0);

    // TODO: 从数据库获取用户
    // let user = user_service.get_user(id).await?;

    res.render(Json(ApiResponse::success(serde_json::json!({
        "id": id,
        "email": "user@example.com",
        "username": "user",
        "avatar": null,
        "created_at": "2026-05-05T00:00:00Z"
    }))));
}

/// 用户注册
#[handler]
pub async fn register(req: &mut Request, res: &mut Response) {
    let body = req.parse_json::<RegisterRequest>().await;

    match body {
        Ok(data) => {
            // TODO: 调用用户服务注册
            // let user = user_service.register(&data.email, &data.password, data.username.as_deref()).await?;

            res.render(Json(ApiResponse::success(serde_json::json!({
                "id": 1,
                "email": data.email,
                "username": data.nickname,
                "message": "Registration successful, please check your email to verify"
            }))));
        }
        Err(e) => {
            res.render(Json(ApiResponse::<()>::error_with_message(
                rsws_common::error_code::ErrorCode::INVALID_REQUEST_FORMAT,
                format!("Invalid request: {}", e)
            )));
        }
    }
}

/// 用户登录
#[handler]
pub async fn login(req: &mut Request, res: &mut Response) {
    let body = req.parse_json::<LoginRequest>().await;

    match body {
        Ok(data) => {
            // TODO: 调用用户服务登录
            // let login_response = user_service.login(&data.email, &data.password).await?;

            res.render(Json(ApiResponse::success(serde_json::json!({
                "api_key": "ak_example",
                "api_secret": "sk_example",
                "expires_at": "2026-05-12T00:00:00Z",
                "user": {
                    "id": 1,
                    "email": data.email,
                    "username": "user"
                }
            }))));
        }
        Err(e) => {
            res.render(Json(ApiResponse::<()>::error_with_message(
                rsws_common::error_code::ErrorCode::INVALID_REQUEST_FORMAT,
                format!("Invalid request: {}", e)
            )));
        }
    }
}

/// 获取当前用户信息
#[handler]
pub async fn get_current_user(_req: &mut Request, res: &mut Response) {
    // TODO: 从认证中间件获取用户 ID
    // let user_id = req.get_user_id()?;

    res.render(Json(ApiResponse::success(serde_json::json!({
        "id": 1,
        "email": "current@example.com",
        "username": "current_user",
        "avatar": null,
        "balance": "0.00",
        "created_at": "2026-05-05T00:00:00Z"
    }))));
}

/// 更新用户资料
#[handler]
pub async fn update_profile(req: &mut Request, res: &mut Response) {
    let body = req.parse_json::<serde_json::Value>().await;

    match body {
        Ok(_data) => {
            // TODO: 调用用户服务更新资料

            res.render(Json(ApiResponse::success(serde_json::json!({
                "message": "Profile updated successfully"
            }))));
        }
        Err(e) => {
            res.render(Json(ApiResponse::<()>::error_with_message(
                rsws_common::error_code::ErrorCode::INVALID_REQUEST_FORMAT,
                format!("Invalid request: {}", e)
            )));
        }
    }
}

/// 修改密码
#[handler]
pub async fn change_password(req: &mut Request, res: &mut Response) {
    let body = req.parse_json::<serde_json::Value>().await;

    match body {
        Ok(_data) => {
            // TODO: 验证旧密码并更新新密码

            res.render(Json(ApiResponse::success(serde_json::json!({
                "message": "Password changed successfully"
            }))));
        }
        Err(e) => {
            res.render(Json(ApiResponse::<()>::error_with_message(
                rsws_common::error_code::ErrorCode::INVALID_REQUEST_FORMAT,
                format!("Invalid request: {}", e)
            )));
        }
    }
}
