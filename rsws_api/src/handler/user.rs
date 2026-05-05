//! 用户处理器

use salvo::prelude::*;
use rsws_common::response::ApiResponse;
use rsws_common::error_code::ErrorCode;
use rsws_model::user::user::{RegisterRequest, LoginRequest, ChangePasswordRequest, UpdateProfileRequest};
use crate::state::{get_state, require_user_id};

/// 获取用户信息（按 ID）
#[handler]
pub async fn get_user(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let id: i64 = req.param("id").unwrap_or(0);
    let state = get_state(depot);

    match state.user_service.get_user(id).await {
        Ok(user) => {
            res.render(Json(ApiResponse::success(serde_json::json!({
                "id": user.id,
                "email": user.email,
                "username": user.username,
                "nickname": user.nickname,
                "avatar_url": user.avatar_url,
                "is_active": user.is_active,
                "created_at": user.created_at,
                "updated_at": user.updated_at,
            }))));
        }
        Err(e) => {
            let code = e.error_code();
            let msg = e.to_string();
            let status = salvo::http::StatusCode::from_u16(code.http_status())
                .unwrap_or(salvo::http::StatusCode::INTERNAL_SERVER_ERROR);
            res.status_code(status);
            res.render(Json(ApiResponse::<()>::error_with_message(code, msg)));
        }
    }
}

/// 用户注册
#[handler]
pub async fn register(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let body = req.parse_json::<RegisterRequest>().await;

    match body {
        Ok(data) => {
            let state = get_state(depot);

            match state.user_service.register(&data).await {
                Ok(user) => {
                    res.render(Json(ApiResponse::success(serde_json::json!({
                        "id": user.id,
                        "email": user.email,
                        "username": user.username,
                        "nickname": user.nickname,
                        "message": "Registration successful"
                    }))));
                }
                Err(e) => {
                    let code = e.error_code();
                    let status = salvo::http::StatusCode::from_u16(code.http_status())
                        .unwrap_or(salvo::http::StatusCode::INTERNAL_SERVER_ERROR);
                    res.status_code(status);
                    res.render(Json(ApiResponse::<()>::error_with_message(code, e.to_string())));
                }
            }
        }
        Err(e) => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(ApiResponse::<()>::error_with_message(
                ErrorCode::INVALID_REQUEST_FORMAT,
                format!("Invalid request: {}", e)
            )));
        }
    }
}

/// 用户登录
#[handler]
pub async fn login(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let body = req.parse_json::<LoginRequest>().await;

    match body {
        Ok(data) => {
            let state = get_state(depot);

            match state.user_service.login(&data).await {
                Ok(response) => {
                    res.render(Json(ApiResponse::success(response)));
                }
                Err(e) => {
                    let code = e.error_code();
                    let status = salvo::http::StatusCode::from_u16(code.http_status())
                        .unwrap_or(salvo::http::StatusCode::INTERNAL_SERVER_ERROR);
                    res.status_code(status);
                    res.render(Json(ApiResponse::<()>::error_with_message(code, e.to_string())));
                }
            }
        }
        Err(e) => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(ApiResponse::<()>::error_with_message(
                ErrorCode::INVALID_REQUEST_FORMAT,
                format!("Invalid request: {}", e)
            )));
        }
    }
}

/// 获取当前用户信息
#[handler]
pub async fn get_current_user(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let _ = req; // 不使用 req，但 Salvo handler 签名可能需要
    let user_id = match require_user_id(depot) {
        Ok(id) => id,
        Err(_) => {
            res.status_code(StatusCode::UNAUTHORIZED);
            res.render(Json(ApiResponse::<()>::error(
                ErrorCode::AUTH_MISSING_CREDENTIALS
            )));
            return;
        }
    };

    let state = get_state(depot);

    match state.user_service.get_user(user_id).await {
        Ok(user) => {
            res.render(Json(ApiResponse::success(serde_json::json!({
                "id": user.id,
                "email": user.email,
                "username": user.username,
                "nickname": user.nickname,
                "avatar_url": user.avatar_url,
                "is_active": user.is_active,
                "created_at": user.created_at,
                "updated_at": user.updated_at,
            }))));
        }
        Err(e) => {
            let code = e.error_code();
            let status = salvo::http::StatusCode::from_u16(code.http_status())
                .unwrap_or(salvo::http::StatusCode::INTERNAL_SERVER_ERROR);
            res.status_code(status);
            res.render(Json(ApiResponse::<()>::error_with_message(code, e.to_string())));
        }
    }
}

/// 更新用户资料
#[handler]
pub async fn update_profile(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let user_id = match require_user_id(depot) {
        Ok(id) => id,
        Err(_) => {
            res.status_code(StatusCode::UNAUTHORIZED);
            res.render(Json(ApiResponse::<()>::error(
                ErrorCode::AUTH_MISSING_CREDENTIALS
            )));
            return;
        }
    };

    let body = req.parse_json::<UpdateProfileRequest>().await;

    match body {
        Ok(data) => {
            let state = get_state(depot);

            if let Some(nickname) = data.nickname {
                match state.user_service.update_nickname(user_id, &nickname).await {
                    Ok(user) => {
                        res.render(Json(ApiResponse::success(serde_json::json!({
                            "id": user.id,
                            "nickname": user.nickname,
                            "avatar_url": user.avatar_url,
                            "message": "Profile updated successfully"
                        }))));
                    }
                    Err(e) => {
                        let code = e.error_code();
                        let status = salvo::http::StatusCode::from_u16(code.http_status())
                            .unwrap_or(salvo::http::StatusCode::INTERNAL_SERVER_ERROR);
                        res.status_code(status);
                        res.render(Json(ApiResponse::<()>::error_with_message(code, e.to_string())));
                    }
                }
            } else {
                res.render(Json(ApiResponse::<()>::error_with_message(
                    ErrorCode::INVALID_PARAMETER,
                    "No fields to update"
                )));
            }
        }
        Err(e) => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(ApiResponse::<()>::error_with_message(
                ErrorCode::INVALID_REQUEST_FORMAT,
                format!("Invalid request: {}", e)
            )));
        }
    }
}

/// 修改密码
#[handler]
pub async fn change_password(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let user_id = match require_user_id(depot) {
        Ok(id) => id,
        Err(_) => {
            res.status_code(StatusCode::UNAUTHORIZED);
            res.render(Json(ApiResponse::<()>::error(
                ErrorCode::AUTH_MISSING_CREDENTIALS
            )));
            return;
        }
    };

    let body = req.parse_json::<ChangePasswordRequest>().await;

    match body {
        Ok(data) => {
            let state = get_state(depot);

            match state.user_service.change_password(
                user_id,
                &data.old_password,
                &data.new_password,
            ).await {
                Ok(()) => {
                    res.render(Json(ApiResponse::success(serde_json::json!({
                        "message": "Password changed successfully"
                    }))));
                }
                Err(e) => {
                    let code = e.error_code();
                    let status = salvo::http::StatusCode::from_u16(code.http_status())
                        .unwrap_or(salvo::http::StatusCode::INTERNAL_SERVER_ERROR);
                    res.status_code(status);
                    res.render(Json(ApiResponse::<()>::error_with_message(code, e.to_string())));
                }
            }
        }
        Err(e) => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(ApiResponse::<()>::error_with_message(
                ErrorCode::INVALID_REQUEST_FORMAT,
                format!("Invalid request: {}", e)
            )));
        }
    }
}
