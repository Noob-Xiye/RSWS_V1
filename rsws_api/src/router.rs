//! 路由定义

use salvo::prelude::*;
use salvo::affix_state;
use salvo::oapi::OpenApi;
use salvo_oapi::swagger_ui::SwaggerUi;
use crate::handler;
use crate::middleware::auth::api_key_auth;
use crate::state::AppState;

/// 创建路由（带 State 注入 + OpenAPI 文档）
pub fn create_router(state: AppState) -> Router {
    let api_routes = Router::new()
        // 健康检查（无需认证）
        .push(Router::with_path("health").get(handler::health))

        // API v1（带认证）
        .push(
            Router::with_path("api/v1")
                .hoop(api_key_auth)
                // 用户相关
                .push(Router::with_path("user")
                    .push(Router::new()
                        .get(handler::user::get_current_user)
                        .push(Router::with_path("register").post(handler::user::register))
                        .push(Router::with_path("login").post(handler::user::login))
                        .push(Router::with_path("profile").put(handler::user::update_profile))
                        .push(Router::with_path("password").put(handler::user::change_password))
                    )
                    .push(Router::with_path("<id>").get(handler::user::get_user))
                )

                // 资源相关
                .push(Router::with_path("resource")
                    .push(Router::new()
                        .get(handler::resource::list_resources)
                        .post(handler::resource::create_resource)
                    )
                    .push(Router::with_path("<id>")
                        .get(handler::resource::get_resource)
                        .put(handler::resource::update_resource)
                        .delete(handler::resource::delete_resource)
                    )
                )

                // 订单相关
                .push(Router::with_path("order")
                    .push(Router::new()
                        .get(handler::order::list_orders)
                        .post(handler::order::create_order)
                    )
                    .push(Router::with_path("<id>")
                        .get(handler::order::get_order)
                        .push(Router::with_path("cancel").post(handler::order::cancel_order))
                        .push(Router::with_path("status").get(handler::order::check_order_status))
                    )
                )
        )

        // 支付相关（无需 API Key 认证）
        .push(
            Router::with_path("api/v1/payment")
                .push(Router::with_path("usdt/<network>").get(handler::payment::get_usdt_address))
                .push(Router::with_path("paypal/success").get(handler::payment::paypal_success))
                .push(Router::with_path("paypal/cancel").get(handler::payment::paypal_cancel))
        )

        // Webhook（无需 API Key 认证，有独立的签名验证）
        .push(
            Router::with_path("api/v1/webhook")
                .push(Router::with_path("paypal").post(handler::payment::paypal_webhook))
                .push(Router::with_path("usdt").post(handler::payment::usdt_webhook))
        );

    // OpenAPI 文档生成
    let doc = OpenApi::new("RSWS API", "0.1.0")
        .merge_router(&api_routes);

    Router::new()
        .hoop(affix_state::inject(state))
        // Swagger UI（访问 /swagger-ui 查看）
        .push(doc.into_router("/api-doc/openapi.json"))
        .push(SwaggerUi::new("/swagger-ui").into_router("/api-doc/openapi.json"))
        // 业务路由
        .push(api_routes)
}
