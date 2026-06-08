//! 用户处理器
//!
//! 使用 ResponseExt 和 AuthHandler trait 简化样板代码

use crate::state::get_state;
use base64::Engine as _;
use chrono::{Duration, Utc};
use rsws_common::{error_code::ErrorCode, AuthHandler, ResponseExt, RswsError};
use rsws_model::user_models::user::{
    ChangePasswordRequest, LoginRequest, RegisterRequest, UpdateProfileRequest,
};
use salvo::prelude::*;
use salvo_oapi::endpoint;
use serde::Deserialize;

/// 获取用户信息（按 ID）
#[endpoint(
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
            res.success(serde_json::json!({
                "id": user.id,
                "email": user.email,
                "username": user.username,
                "nickname": user.nickname,
                "avatar_url": user.avatar_url,
                "is_active": user.is_active,
                "created_at": user.created_at,
                "updated_at": user.updated_at,
            }));
        }
        Err(e) => {
            res.error(e);
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
                    // 注册成功，自动登录：创建并持久化 api_key
                    let create_req = rsws_model::api_key::CreateApiKeyRequest {
                        name: "login_session".to_string(),
                        permissions: vec!["all".to_string()],
                        rate_limit: Some(1000),
                        expires_in_days: Some(7),
                    };

                    match state.user_api_key_manager.create(user.id, create_req).await {
                        Ok(api_key_resp) => {
                            res.success(serde_json::json!({
                                "user": {
                                    "id": user.id,
                                    "email": user.email,
                                    "username": user.username,
                                    "nickname": user.nickname,
                                    "avatar_url": user.avatar_url,
                                    "is_active": user.is_active,
                                },
                                "api_key": api_key_resp.api_key,
                                "expires_at": api_key_resp.expires_at
                                    .unwrap_or_else(|| chrono::Utc::now() + chrono::Duration::days(7))
                                    .to_rfc3339(),
                            }));
                        }
                        Err(e) => {
                            tracing::error!("Failed to create api_key on register: {}", e);
                            // 注册成功但 api_key 创建失败，仍返回用户信息
                            res.success(serde_json::json!({
                                "user": {
                                    "id": user.id,
                                    "email": user.email,
                                    "username": user.username,
                                    "nickname": user.nickname,
                                    "avatar_url": user.avatar_url,
                                    "is_active": user.is_active,
                                },
                                "message": "Registration successful, but auto-login failed"
                            }));
                        }
                    }
                }
                Err(e) => {
                    res.error(e);
                }
            }
        }
        Err(e) => {
            res.http_error(StatusCode::BAD_REQUEST, format!("Invalid request: {}", e));
        }
    }
}

/// 用户登录
///
/// 流程：
/// 1. user_service 验证凭据
/// 2. api_key_service 创建并持久化 api_key
/// 3. 返回 user_info + session_data(api_key)
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
                Ok(mut login_response) => {
                    // 登录成功后，创建并持久化 api_key
                    if let Some(ref user) = login_response.user {
                        let create_req = rsws_model::api_key::CreateApiKeyRequest {
                            name: "login_session".to_string(),
                            permissions: vec!["all".to_string()],
                            rate_limit: Some(1000),
                            expires_in_days: Some(7),
                        };

                        match state.user_api_key_manager.create(user.id, create_req).await {
                            Ok(api_key_resp) => {
                                login_response.api_key = Some(api_key_resp.api_key);
                                login_response.expires_at = Some(
                                    api_key_resp
                                        .expires_at
                                        .unwrap_or_else(|| Utc::now() + Duration::days(7)),
                                );
                            }
                            Err(e) => {
                                tracing::error!("Failed to create api_key on login: {}", e);
                                // 登录成功但 api_key 创建失败，仍然返回用户信息
                                // 前端需要处理 api_key 为 None 的情况
                            }
                        }
                    }
                    res.success(login_response);
                }
                Err(e) => {
                    res.error(e);
                }
            }
        }
        Err(e) => {
            res.http_error(StatusCode::BAD_REQUEST, format!("Invalid request: {}", e));
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
    let user_id = match res.auth_require_user_id(depot) {
        Some(id) => id,
        None => return,
    };

    let state = get_state(depot);

    match state.user_service.get_user(user_id).await {
        Ok(user) => {
            res.success(serde_json::json!({
                "id": user.id,
                "email": user.email,
                "username": user.username,
                "nickname": user.nickname,
                "avatar_url": user.avatar_url,
                "is_active": user.is_active,
                "created_at": user.created_at,
                "updated_at": user.updated_at,
            }));
        }
        Err(e) => {
            res.error(e);
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
    let user_id = match res.auth_require_user_id(depot) {
        Some(id) => id,
        None => return,
    };

    let body = req.parse_json::<UpdateProfileRequest>().await;

    match body {
        Ok(data) => {
            let state = get_state(depot);

            if let Some(nickname) = data.nickname {
                match state.user_service.update_nickname(user_id, &nickname).await {
                    Ok(user) => {
                        res.success(serde_json::json!({
                            "id": user.id,
                            "email": user.email,
                            "username": user.username,
                            "nickname": user.nickname,
                            "avatar_url": user.avatar_url,
                            "is_active": user.is_active,
                            "created_at": user.created_at,
                            "updated_at": user.updated_at,
                        }));
                    }
                    Err(e) => {
                        res.error(e);
                    }
                }
            } else {
                res.error_msg(
                    RswsError::from(ErrorCode::INVALID_PARAMETER),
                    "No fields to update",
                );
            }
        }
        Err(e) => {
            res.http_error(StatusCode::BAD_REQUEST, format!("Invalid request: {}", e));
        }
    }
}

/// 修改密码
#[endpoint(
    request_body = ChangePasswordRequest,
    responses(
        (status_code = 200, description = "修改成功"),
        (status_code = 401, description = "未认证"),
        (status_code = 400, description = "密码错误"),
    )
)]
pub async fn change_password(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let user_id = match res.auth_require_user_id(depot) {
        Some(id) => id,
        None => return,
    };

    let body = req.parse_json::<ChangePasswordRequest>().await;

    match body {
        Ok(data) => {
            let state = get_state(depot);

            match state
                .user_service
                .change_password(user_id, &data.old_password, &data.new_password)
                .await
            {
                Ok(()) => {
                    res.success(serde_json::json!({
                        "message": "Password changed successfully"
                    }));
                }
                Err(e) => {
                    res.error(e);
                }
            }
        }
        Err(e) => {
            res.http_error(StatusCode::BAD_REQUEST, format!("Invalid request: {}", e));
        }
    }
}

/// 上传头像（base64 编码）
#[derive(Debug, Deserialize)]
pub struct UploadAvatarRequest {
    /// data:image/png;base64,xxxxx
    pub avatar_data: String,
}

#[endpoint(
    responses(
        (status_code = 200, description = "上传成功，返回 avatar_url"),
        (status_code = 401, description = "未认证"),
        (status_code = 400, description = "文件无效或过大"),
    )
)]
pub async fn upload_avatar(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let user_id = match res.auth_require_user_id(depot) {
        Some(id) => id,
        None => return,
    };

    let state = get_state(depot);
    let config = &state.config;

    let body = match req.parse_json::<UploadAvatarRequest>().await {
        Ok(b) => b,
        Err(e) => {
            res.http_error(StatusCode::BAD_REQUEST, format!("Invalid request: {}", e));
            return;
        }
    };

    // 解析 data URI: data:image/png;base64,xxxxx
    let data_uri = &body.avatar_data;
    let (mime_type, b64_content) = match data_uri.split_once(',') {
        Some((prefix, content)) => {
            let mime = prefix.strip_prefix("data:").unwrap_or("");
            (mime.to_string(), content.to_string())
        }
        None => {
            res.http_error(StatusCode::BAD_REQUEST, "Invalid data URI format");
            return;
        }
    };

    // 校验文件类型
    let allowed = ["image/jpeg", "image/png", "image/gif", "image/webp"];
    if !allowed.contains(&mime_type.as_str()) {
        res.http_error(
            StatusCode::BAD_REQUEST,
            "Invalid file type, only JPEG/PNG/GIF/WebP allowed",
        );
        return;
    }

    // 解码 base64
    let file_bytes = match base64::engine::general_purpose::STANDARD.decode(&b64_content) {
        Ok(b) => b,
        Err(e) => {
            res.http_error(StatusCode::BAD_REQUEST, format!("Invalid base64: {}", e));
            return;
        }
    };

    // 校验文件大小（2MB）
    if file_bytes.len() > 2 * 1024 * 1024 {
        res.http_error(StatusCode::BAD_REQUEST, "File too large, max 2MB");
        return;
    }

    // 确保目录存在
    let avatar_dir = format!("{}/avatars", config.server.upload_dir);
    std::fs::create_dir_all(&avatar_dir).ok();

    // 生成唯一文件名
    let ext = match mime_type.as_str() {
        "image/jpeg" => "jpg",
        "image/png" => "png",
        "image/gif" => "gif",
        "image/webp" => "webp",
        _ => "bin",
    };
    let filename = format!("{}_{}.{}", user_id, chrono::Utc::now().timestamp(), ext);
    let filepath = format!("{}/{}", avatar_dir, filename);

    // 保存文件
    match tokio::fs::write(&filepath, file_bytes).await {
        Ok(()) => {}
        Err(e) => {
            tracing::error!("Failed to save avatar: {}", e);
            res.http_error(StatusCode::INTERNAL_SERVER_ERROR, "Failed to save file");
            return;
        }
    }

    let avatar_url = format!("/uploads/avatars/{}", filename);

    // 更新数据库
    match state.user_service.update_avatar(user_id, &avatar_url).await {
        Ok(user) => {
            res.success(serde_json::json!({
                "avatar_url": user.avatar_url,
            }));
        }
        Err(e) => {
            res.error(e);
        }
    }
}

/// 发送验证码
#[derive(Debug, Deserialize)]
pub struct SendCodeRequest {
    pub email: String,
    pub code_type: String,
}

#[endpoint(
    responses(
        (status_code = 200, description = "发送成功"),
        (status_code = 400, description = "请求格式错误"),
        (status_code = 429, description = "发送过于频繁"),
    )
)]
pub async fn send_code(req: &mut Request, _depot: &mut Depot, res: &mut Response) {
    let body = req.parse_json::<SendCodeRequest>().await;

    match body {
        Ok(data) => {
            let state = get_state(_depot);

            match state
                .user_service
                .send_verification_code(&data.email, &data.code_type)
                .await
            {
                Ok(_ttl) => {
                    res.success(serde_json::json!({
                        "success": true,
                        "message": "Verification code sent"
                    }));
                }
                Err(e) => {
                    res.error(e);
                }
            }
        }
        Err(e) => {
            res.http_error(StatusCode::BAD_REQUEST, format!("Invalid request: {}", e));
        }
    }
}
