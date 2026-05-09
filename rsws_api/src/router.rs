//! 路由定义

use salvo::prelude::*;
use salvo::affix_state;
use salvo::oapi::OpenApi;
use salvo_cors::Cors;
use salvo_cors::Any;
use salvo::http::Method;
use salvo_oapi::swagger_ui::SwaggerUi;
use crate::handler;
use crate::middleware::auth::{api_key_auth, rate_limit};
use crate::state::AppState;

/// 创建路由（带 State 注入 + OpenAPI 文档）
pub fn create_router(state: AppState) -> Router {
    let api_routes = Router::new()
        // 健康检查（无需认证）
        .push(Router::with_path("health").get(handler::health))

        // API v1（统一 API Key 认证 + 速率限制）
        .push(
            Router::with_path("api/v1")
                .hoop(api_key_auth)
                .hoop(rate_limit)
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

                // 资源相关（无认证的查询部分）
                .push(Router::with_path("resource")
                    .push(Router::new()
                        .get(handler::resource::list_resources)
                        .post(handler::resource::create_resource)
                    )
                    .push(Router::with_path("<id>")
                        .get(handler::resource::get_resource)
                        .put(handler::resource::update_resource)
                        .delete(handler::resource::delete_resource)
                        .push(Router::with_path("purchase-check")
                            .hoop(api_key_auth)
                            .get(handler::order::check_purchase)
                        )
                        .push(Router::with_path("download")
                            .hoop(api_key_auth)
                            .get(handler::order::get_resource_download)
                        )
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

                // 管理后台（同样使用 API Key 认证，handler 内检查 is_admin）
                .push(Router::with_path("admin")
                    .push(Router::new()
                        .get(handler::admin::get_current_admin)
                        .push(Router::with_path("list").get(handler::admin::list_admins))
                        .push(Router::with_path("create").post(handler::admin::create_admin))
                        .push(Router::with_path("api-keys")
                            .get(handler::admin::list_api_keys)
                            .post(handler::admin::create_api_key)
                        )
                        // 日志配置管理
                        .push(Router::with_path("log-configs")
                            .get(handler::admin::list_log_configs)
                            .post(handler::admin::create_log_config)
                        )
                        // 日志查询
                        .push(Router::with_path("logs/system").get(handler::admin::query_system_logs))
                        // Dashboard 统计
                        .push(Router::with_path("dashboard/stats").get(handler::admin::dashboard_stats))
                    )
                    .push(Router::with_path("<id>")
                        .get(handler::admin::get_admin)
                        .push(Router::with_path("deactivate").post(handler::admin::deactivate_admin))
                        .push(Router::with_path("api-keys/<key_id>").delete(handler::admin::delete_api_key))
                    )
                    // 日志配置详情/更新/删除
                    .push(Router::with_path("log-configs/<key>")
                        .get(handler::admin::get_log_config)
                        .put(handler::admin::update_log_config)
                        .delete(handler::admin::delete_log_config)
                    )
                )
                // USDT 钱包配置
                .push(Router::with_path("usdt-wallets")
                    .get(handler::admin::list_usdt_wallets)
                    .push(Router::with_path("<network>").put(handler::admin::update_usdt_wallet))
                )
        )

        // 管理员登录（无需 API Key，使用邮箱+密码）
        .push(
            Router::with_path("api/v1/admin/login")
                .post(handler::admin::login)
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

    // CORS 中间件 — 允许前端跨域访问
    let cors = Cors::new()
        .allow_origin(Any)
        .allow_methods(vec![Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
        .allow_headers(vec!["Content-Type", "Authorization", "X-Api-Key", "X-Signature"])
        .allow_credentials(true)
        .max_age(3600);

    Router::new()
        .hoop(cors.into_handler())
        .hoop(affix_state::inject(state))
        // Swagger UI
        .push(doc.into_router("/api-doc/openapi.json"))
        .push(SwaggerUi::new("/swagger-ui").into_router("/api-doc/openapi.json"))
        // 业务路由
        .push(api_routes)
}
