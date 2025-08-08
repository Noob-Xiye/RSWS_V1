use crate::config_handler::ConfigHandler;
use crate::user_handler::{
    login_with_code, register_with_code, send_login_code, send_verification_code,
};
use salvo::oapi::OpenApi;
use salvo::prelude::*;

use crate::middleware::auth::AuthMiddleware;
use crate::middleware::signature_auth::SignatureAuthMiddleware;
use crate::middleware::unified_auth::UnifiedAuthMiddleware;
use rsws_service::{AdminService, AuthService};

pub fn create_router(config_handler: Arc<ConfigHandler>) -> Router {
    Router::new()
        .push(Router::with_path("register/send_code").post(crate::user::send_register_code))
        .push(Router::with_path("register/verify").post(crate::user::verify_register_code));
    // 其他模块路由可在此添加

    Router::new()
        .push(user_router)
        // .push(resource_router)
        // .push(order_router)
        // .push(request_router)
        // .push(config_router)
        // .push(log_router)
        .push(
            Router::with_path("/api/v1/config")
                .get(config_handler.get_config)
                .post(config_handler.set_config)
                .push(Router::with_path("/cache/clear").post(config_handler.clear_cache)),
        )
}

use crate::order::{
    cancel_order, create_order, get_order, get_payment_methods, get_user_orders, pay_order,
    verify_payment,
};

// 在现有的 router.rs 文件中添加管理员路由

// 导入管理员处理器
use crate::admin_handler::AdminHandler;

// 在 create_router 函数中添加管理员路由
pub fn create_router() -> Router {
    use crate::config_handler;
    use crate::middleware::auth::AuthMiddleware;
    use crate::middleware::signature_auth::SignatureAuthMiddleware;
    use crate::user_handler;
    use salvo::oapi::OpenApi;
    use salvo::prelude::Router;

    // 公开路由（不需要认证）
    let public_router = Router::new().push(
        Router::with_path("auth")
            .push(
                Router::with_path("register/send_code").post(user_handler::send_verification_code),
            )
            .push(Router::with_path("register/verify").post(user_handler::register_with_code))
            .push(Router::with_path("login/send_code").post(user_handler::send_login_code))
            .push(Router::with_path("login/verify_code").post(user_handler::login_with_code))
            .push(Router::with_path("login").post(user_handler::traditional_login)),
    );

    // 需要认证的路由
    let protected_router = Router::new()
        .hoop(SignatureAuthMiddleware::new())
        .push(
            Router::with_path("user")
                .push(
                    Router::with_path("profile")
                        .get(user_handler::get_user_profile)
                        .put(user_handler::update_user_profile),
                )
                .push(Router::with_path("logout").post(user_handler::logout))
                .push(Router::with_path("purchases").get(user_handler::get_user_purchases))
                .push(Router::with_path("password").put(user_handler::change_password))
                .push(
                    Router::with_path("email/send_code").post(user_handler::send_email_change_code),
                )
                .push(Router::with_path("email/verify").post(user_handler::verify_email_change))
                .push(Router::with_path("avatar").post(user_handler::upload_avatar)),
        )
        .push(
            Router::with_path("orders")
                .post(create_order)
                .get(get_user_orders)
                .push(Router::with_path("<id>").get(get_order))
                .push(Router::with_path("<id>/pay").post(pay_order))
                .push(Router::with_path("<id>/cancel").post(cancel_order)),
        )
        .push(
            Router::with_path("payment")
                .push(Router::with_path("methods").get(get_payment_methods))
                .push(Router::with_path("<payment_id>/verify").get(verify_payment)),
        );

    let api_router = Router::new().push(
        Router::with_path("v1")
            .push(public_router)
            .push(protected_router)
            .push(
                Router::with_path("config")
                    .get(config_handler::get_config)
                    .post(config_handler::set_config),
            ),
    );

    let doc = OpenApi::new("RSWS API", "1.0.0").paths(api_router.paths().clone());

    Router::new()
        .push(Router::with_path("api").push(api_router))
        .push(doc.into_router("/openapi.json"))
        .push(SwaggerUi::new("/swagger-ui/").into_router("/swagger-ui/*any"));

    // 管理员路由（独立的认证系统）
    let admin_router = Router::with_path("admin")
        .push(Router::with_path("auth/login").post(admin_handler.login))
        .push(
            Router::new()
                .push(
                    Router::with_path("admins")
                        .get(admin_handler.get_admins)
                        .post(admin_handler.create_admin),
                )
                .push(
                    Router::with_path("admins/:id")
                        .get(admin_handler.get_admin_info)
                        .put(admin_handler.update_admin),
                ), // 其他管理员API路由
                   // ...
        );

    Router::new()
        .push(public_router)
        .push(protected_router)
        .push(admin_router) // 添加管理员路由
        .push(doc.into_router("/openapi.json"))
        .push(SwaggerUi::new("/swagger-ui/").into_router("/swagger-ui/*any"))
}
