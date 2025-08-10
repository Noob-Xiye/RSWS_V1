use rsws_model::response::ApiResponse;
use rsws_model::user::*;
use rsws_service::UserService;
use salvo::conn::SocketAddr;
use salvo::oapi::endpoint;
use salvo::prelude::*;
use std::net::IpAddr;
use std::sync::Arc;

#[derive(Clone)]
pub struct UserHandler {
    user_service: Arc<UserService>,
}

// 添加缺失的用户处理函数
impl UserHandler {
    pub fn new(user_service: Arc<UserService>) -> Self {
        Self { user_service }
    }

    pub async fn get_wallet(req: &mut Request, res: &mut Response) -> Result<(), salvo::Error> {
        // 获取用户钱包信息的实现
    }

    pub async fn get_user_transactions(
        req: &mut Request,
        res: &mut Response,
    ) -> Result<(), salvo::Error> {
        // 获取用户交易记录的实现
    }

    pub async fn change_password(
        req: &mut Request,
        res: &mut Response,
    ) -> Result<(), salvo::Error> {
        // 修改密码的实现
    }

    pub async fn send_email_change_code(
        req: &mut Request,
        res: &mut Response,
    ) -> Result<(), salvo::Error> {
        // 发送邮箱修改验证码的实现
    }

    pub async fn verify_email_change(
        req: &mut Request,
        res: &mut Response,
    ) -> Result<(), salvo::Error> {
        // 验证邮箱修改的实现
    }

    pub async fn upload_avatar(req: &mut Request, res: &mut Response) -> Result<(), salvo::Error> {
        // 上传头像的实现
    }
}

#[endpoint(
    tags("用户认证"),
    responses(
        (status_code = 200, description = "验证码发送成功", body = SendVerificationCodeResponse),
        (status_code = 400, description = "请求参数错误"),
        (status_code = 500, description = "服务器内部错误")
    )
)]
pub async fn send_verification_code(
    req: &mut Request,
    res: &mut Response,
    depot: &mut Depot,
) -> Result<(), salvo::Error> {
    let user_handler = depot.obtain::<UserHandler>().unwrap();

    let request: SendVerificationCodeRequest = req
        .parse_json()
        .await
        .map_err(|e| salvo::Error::other(format!("Invalid request: {}", e)))?;

    let response = user_handler
        .user_service
        .send_registration_code(request)
        .await
        .map_err(|e| salvo::Error::other(format!("Service error: {}", e)))?;

    res.render(Json(response));
    Ok(())
}

#[endpoint(
    tags("用户认证"),
    responses(
        (status_code = 200, description = "注册成功", body = RegisterResponse),
        (status_code = 400, description = "验证码错误或已过期"),
        (status_code = 500, description = "服务器内部错误")
    )
)]
pub async fn register_with_code(
    req: &mut Request,
    res: &mut Response,
    depot: &mut Depot,
) -> Result<(), salvo::Error> {
    let user_handler = depot.obtain::<UserHandler>().unwrap();

    let request: VerifyCodeRequest = req
        .parse_json()
        .await
        .map_err(|e| salvo::Error::other(format!("Invalid request: {}", e)))?;

    let response = user_handler
        .user_service
        .register_with_code(request)
        .await
        .map_err(|e| salvo::Error::other(format!("Service error: {}", e)))?;

    res.render(Json(response));
    Ok(())
}

#[endpoint(
    tags("用户认证"),
    responses(
        (status_code = 200, description = "登录验证码发送成功", body = SendLoginCodeResponse),
        (status_code = 400, description = "请求参数错误"),
        (status_code = 500, description = "服务器内部错误")
    )
)]
pub async fn send_login_code(
    req: &mut Request,
    res: &mut Response,
    depot: &mut Depot,
) -> Result<(), salvo::Error> {
    let user_handler = depot.obtain::<UserHandler>().unwrap();

    let request: SendLoginCodeRequest = req
        .parse_json()
        .await
        .map_err(|e| salvo::Error::other(format!("Invalid request: {}", e)))?;

    let response = user_handler
        .user_service
        .send_login_code(request)
        .await
        .map_err(|e| salvo::Error::other(format!("Service error: {}", e)))?;

    res.render(Json(response));
    Ok(())
}

#[endpoint(
    tags("用户认证"),
    responses(
        (status_code = 200, description = "登录成功", body = ApiResponse<TraditionalLoginResponse>),
        (status_code = 400, description = "请求参数错误"),
        (status_code = 500, description = "服务器内部错误")
    )
)]
pub async fn traditional_login(
    req: &mut Request,
    res: &mut Response,
    depot: &mut Depot,
) -> Result<(), salvo::Error> {
    let user_handler = depot.obtain::<UserHandler>().unwrap();

    // 解析请求体
    let request: LoginRequest = match req.parse_json().await {
        Ok(req) => req,
        Err(_) => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(ApiResponse::bad_request("Invalid request format")));
            return Ok(());
        }
    };

    // 获取IP地址
    let ip = req.remote_addr().ip();
    let user_agent = req
        .headers()
        .get("User-Agent")
        .and_then(|v| v.to_str().ok());

    match user_handler
        .user_service
        .traditional_login(request, Some(ip), user_agent)
        .await
    {
        Ok(response) => {
            res.render(Json(ApiResponse::success(response)));
        }
        Err(e) => {
            let (status, message) = match e {
                ServiceError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
                ServiceError::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg),
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                ),
            };
            res.status_code(status);
            res.render(Json(ApiResponse::error(status.as_u16() as i32, &message)));
        }
    }

    Ok(())
}

#[endpoint(
    tags("用户管理"),
    responses(
        (status_code = 200, description = "登出成功", body = LogoutResponse),
        (status_code = 400, description = "请求参数错误"),
        (status_code = 500, description = "服务器内部错误")
    )
)]
pub async fn logout(
    req: &mut Request,
    res: &mut Response,
    depot: &mut Depot,
) -> Result<(), salvo::Error> {
    let user_handler = depot.obtain::<UserHandler>().unwrap();
    let user_id = req
        .headers()
        .get("X-User-ID")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<i64>().ok())
        .ok_or_else(|| salvo::Error::other("Missing or invalid user ID"))?;

    let request: LogoutRequest = req
        .parse_json()
        .await
        .map_err(|e| salvo::Error::other(format!("Invalid request: {}", e)))?;

    let response = user_handler
        .user_service
        .logout(user_id, request)
        .await
        .map_err(|e| salvo::Error::other(format!("Service error: {}", e)))?;

    res.render(Json(response));
    Ok(())
}

#[endpoint(
    tags("用户管理"),
    responses(
        (status_code = 200, description = "获取用户资料成功", body = UserProfile),
        (status_code = 404, description = "用户不存在"),
        (status_code = 500, description = "服务器内部错误")
    )
)]
pub async fn get_user_profile(
    req: &mut Request,
    res: &mut Response,
    depot: &mut Depot,
) -> Result<(), salvo::Error> {
    let user_handler = depot.obtain::<UserHandler>().unwrap();
    let user_id = req
        .headers()
        .get("X-User-ID")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<i64>().ok())
        .ok_or_else(|| salvo::Error::other("Missing or invalid user ID"))?;

    let response = user_handler
        .user_service
        .get_user_profile(user_id)
        .await
        .map_err(|e| salvo::Error::other(format!("Service error: {}", e)))?;

    res.render(Json(response));
    Ok(())
}

#[endpoint(
    tags("用户管理"),
    responses(
        (status_code = 200, description = "更新用户资料成功"),
        (status_code = 400, description = "请求参数错误"),
        (status_code = 500, description = "服务器内部错误")
    )
)]
pub async fn update_user_profile(
    req: &mut Request,
    res: &mut Response,
    depot: &mut Depot,
) -> Result<(), salvo::Error> {
    let user_handler = depot.obtain::<UserHandler>().unwrap();
    let user_id = req
        .headers()
        .get("X-User-ID")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<i64>().ok())
        .ok_or_else(|| salvo::Error::other("Missing or invalid user ID"))?;

    let request: UpdateProfileRequest = req
        .parse_json()
        .await
        .map_err(|e| salvo::Error::other(format!("Invalid request: {}", e)))?;

    user_handler
        .user_service
        .update_user_profile(user_id, request)
        .await
        .map_err(|e| salvo::Error::other(format!("Service error: {}", e)))?;

    res.render(Json(
        serde_json::json!({"success": true, "message": "用户资料更新成功"}),
    ));
    Ok(())
}

#[endpoint(
    tags("用户管理"),
    responses(
        (status_code = 200, description = "获取购买记录成功", body = PaginatedResponse<Order>),
        (status_code = 500, description = "服务器内部错误")
    )
)]
pub async fn get_user_purchases(
    req: &mut Request,
    res: &mut Response,
    depot: &mut Depot,
) -> Result<(), salvo::Error> {
    let user_handler = depot.obtain::<UserHandler>().unwrap();
    let user_id = req
        .headers()
        .get("X-User-ID")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<i64>().ok())
        .ok_or_else(|| salvo::Error::other("Missing or invalid user ID"))?;

    let page = req.query::<u32>("page").unwrap_or(1);
    let page_size = req.query::<u32>("page_size").unwrap_or(10);

    let response = user_handler
        .user_service
        .get_user_purchases(user_id, page, page_size)
        .await
        .map_err(|e| salvo::Error::other(format!("Service error: {}", e)))?;

    res.render(Json(response));
    Ok(())
}
