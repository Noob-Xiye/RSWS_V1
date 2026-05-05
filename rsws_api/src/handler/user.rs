//! 用户处理器

use salvo::prelude::*;
use salvo_oapi::endpoint;
use rsws_common::response::ApiResponse;
use rsws_common::error_code::ErrorCode;
use rsws_model::user::user::{RegisterRequest, LoginRequest, ChangePasswordRequest, UpdateProfileRequest};
use crate::state::{get_state, require_user_id};

/// 获取用户信息（按 ID）
#[endpoint(
    parameters(
        ("id", description = "用户ID"),
    ),
    responses(
        (status_code = 200, description = "成功"),
        (status_code = 404, description = "用户不存在"),
    )
)]
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
#[endpoint(
    request_body = RegisterRequest,
    responses(
        (status_code = 200, description = "注册成功"),
        (status_code = 400, description = "请求格式错误"),
        (status_code = 409, description = "用户已存在"),
    )
)]
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
#[endpoint(
    request_body = LoginRequest,
    responses(
        (status_code = 200, description = "登录成功"),
        (status_code = 400, description = "请求格式错误"),
        (status_code = 401, description = "认证失败"),
    )
)]
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
#[endpoint(
    responses(
        (status_code = 200, description = "成功"),
        (status_code = 401, description = "未认证"),
    )
)]
pub async fn get_current_user(_req: &mut Request, depot: &mut Depot, res: &mut Response) {
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
#[endpoint(
    request_body = UpdateProfileRequest,
    responses(
        (status_code = 200, description = "更新成功"),
        (status_code = 401, description = "未认证"),
        (status_code = 400, description = "请求格式错误"),
    )
)]
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
#[endpoint(
    request_body = ChangePasswordRequest,
    responses(
        (status_code = 200, description = "修改成功"),
        (status_code = 401, description = "未认证"),
        (status_code = 400, description = "旧密码错误"),
    )
)]
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
