use crate::admin_handler::AdminHandler;
use crate::config_handler::ConfigHandler;
use crate::middleware::auth::AuthMiddleware;
use crate::middleware::signature_auth::SignatureAuthMiddleware;
use crate::middleware::unified_auth::UnifiedAuthMiddleware;
use crate::order::{
    cancel_order, create_order, get_order, get_payment_methods, get_user_orders, pay_order,
    verify_payment,
};
use crate::user_handler;
use rsws_service::{AdminService, AuthService};
use salvo::oapi::OpenApi;
use salvo::prelude::*;
use std::sync::Arc;

pub fn create_router(config_handler: Arc<ConfigHandler>) -> Router {
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

    // 需要认证的用户路由
    let protected_user_router = Router::new()
        .hoop(SignatureAuthMiddleware::new(config_handler.clone()))
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
                .push(Router::with_path("avatar").post(user_handler::upload_avatar))
                .push(Router::with_path("wallet").get(user_handler::get_wallet))
                .push(Router::with_path("orders").get(user_handler::get_user_orders))
                .push(Router::with_path("transactions").get(user_handler::get_user_transactions)),
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
                .push(Router::with_path("create-order").post(create_order))
                .push(Router::with_path("process/<order_id>").post(pay_order))
                .push(Router::with_path("status/<order_id>").get(get_order))
                .push(Router::with_path("qrcode/<order_id>").get(generate_qr_code))
                .push(Router::with_path("<payment_id>/verify").get(verify_payment)),
        )
        .push(
            Router::with_path("resources")
                .get(get_resources)
                .post(upload_resource)
                .push(Router::with_path("search").get(search_resources))
                .push(
                    Router::with_path("<id>")
                        .get(get_resource_detail)
                        .put(update_resource)
                        .delete(delete_resource),
                )
                .push(Router::with_path("<id>/download").get(download_resource)),
        );

    // 管理员路由（独立的认证系统）
    let admin_router = Router::with_path("admin")
        .push(Router::with_path("auth/login").post(AdminHandler::login))
        .push(
            Router::new()
                .hoop(AuthMiddleware::new()) // 管理员认证中间件
                .push(
                    Router::with_path("users")
                        .get(AdminHandler::get_users)
                        .push(
                            Router::with_path("<user_id>")
                                .get(AdminHandler::get_user_detail)
                                .put(AdminHandler::update_user)
                                .delete(AdminHandler::delete_user),
                        )
                        .push(Router::with_path("<user_id>/ban").post(AdminHandler::ban_user))
                        .push(Router::with_path("<user_id>/unban").post(AdminHandler::unban_user)),
                )
                .push(
                    Router::with_path("resources")
                        .get(AdminHandler::get_resources)
                        .push(
                            Router::with_path("<resource_id>")
                                .get(AdminHandler::get_resource_detail)
                                .delete(AdminHandler::delete_resource),
                        )
                        .push(
                            Router::with_path("<resource_id>/approve")
                                .post(AdminHandler::approve_resource),
                        )
                        .push(
                            Router::with_path("<resource_id>/reject")
                                .post(AdminHandler::reject_resource),
                        ),
                )
                .push(
                    Router::with_path("orders")
                        .get(AdminHandler::get_orders)
                        .push(Router::with_path("<order_id>").get(AdminHandler::get_order_detail))
                        .push(
                            Router::with_path("<order_id>/status")
                                .put(AdminHandler::update_order_status),
                        )
                        .push(
                            Router::with_path("<order_id>/refund").post(AdminHandler::refund_order),
                        ),
                )
                .push(
                    Router::with_path("config")
                        .get(AdminHandler::get_config)
                        .put(AdminHandler::update_config)
                        .push(
                            Router::with_path("payment-methods")
                                .get(AdminHandler::get_payment_methods),
                        )
                        .push(
                            Router::with_path("payment-methods/<method_id>")
                                .put(AdminHandler::update_payment_method),
                        ),
                )
                .push(
                    Router::with_path("statistics")
                        .push(Router::with_path("dashboard").get(AdminHandler::get_dashboard_stats))
                        .push(Router::with_path("users").get(AdminHandler::get_user_stats))
                        .push(Router::with_path("revenue").get(AdminHandler::get_revenue_stats))
                        .push(Router::with_path("resources").get(AdminHandler::get_resource_stats)),
                ),
        );

    // 公共配置路由
    let config_router = Router::with_path("config")
        .push(Router::with_path("public").get(config_handler.get_public_config))
        .push(Router::with_path("categories").get(config_handler.get_categories))
        .push(Router::with_path("tags").get(config_handler.get_tags));

    let api_router = Router::new().push(
        Router::with_path("api")
            .push(public_router)
            .push(protected_user_router)
            .push(admin_router)
            .push(config_router),
    );

    let doc = OpenApi::new("RSWS API", "1.0.0").paths(api_router.paths().clone());

    Router::new()
        .push(api_router)
        .push(doc.into_router("/openapi.json"))
        .push(SwaggerUi::new("/swagger-ui/").into_router("/swagger-ui/*any"))
}
