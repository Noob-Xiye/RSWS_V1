//! 路由定义

use crate::handler;
use crate::middleware::auth::{api_key_auth, rate_limit, require_admin};
use crate::middleware::request_id::request_id_middleware;
use crate::state::AppState;
use salvo::affix_state;
use salvo::http::Method;
use salvo::oapi::OpenApi;
use salvo::prelude::*;
use salvo_cors::Any;
use salvo_cors::Cors;
use salvo_oapi::swagger_ui::SwaggerUi;

/// 创建路由（带 State 注入 + OpenAPI 文档）
pub fn create_router(state: AppState) -> Router {
    let api_routes = Router::new()
        // 健康检查（无需认证）
        .push(Router::with_path("health").get(handler::health))
        // 分类列表（无需认证）
        .push(Router::with_path("api/v1/categories").get(handler::category::list_categories))
        // API v1（统一 API Key 认证 + 速率限制）
        .push(
            Router::with_path("api/v1")
                .hoop(api_key_auth)
                .hoop(rate_limit)
                // 用户相关
                .push(
                    Router::with_path("user")
                        .push(
                            Router::new()
                                .get(handler::user::get_current_user) // GET /api/v1/user
                                .push(
                                    Router::with_path("info").get(handler::user::get_current_user),
                                ) // GET /api/v1/user/info (前端对齐)
                                .push(Router::with_path("register").post(handler::user::register))
                                .push(Router::with_path("login").post(handler::user::login))
                                .push(
                                    Router::with_path("profile").put(handler::user::update_profile),
                                )
                                .push(
                                    Router::with_path("password")
                                        .put(handler::user::change_password),
                                ) // PUT /api/v1/user/password
                                .push(
                                    Router::with_path("change-password")
                                        .post(handler::user::change_password),
                                ) // POST /api/v1/user/change-password (前端对齐)
                                .push(
                                    Router::with_path("send-code").post(handler::user::send_code),
                                ), // POST /api/v1/user/send-code
                        )
                        .push(Router::with_path("<id>").get(handler::user::get_user)),
                )
                // 资源相关（无认证的查询部分）
                .push(
                    Router::with_path("resource")
                        .push(
                            Router::new()
                                .get(handler::resource::list_resources)
                                .post(handler::resource::create_resource),
                        )
                        .push(
                            Router::with_path("<id>")
                                .get(handler::resource::get_resource)
                                .put(handler::resource::update_resource)
                                .delete(handler::resource::delete_resource)
                                .push(
                                    Router::with_path("purchase-check")
                                        .hoop(api_key_auth)
                                        .get(handler::order::check_purchase),
                                )
                                .push(
                                    Router::with_path("download")
                                        .hoop(api_key_auth)
                                        .get(handler::order::get_resource_download),
                                ),
                        ),
                )
                // 订单相关
                .push(
                    Router::with_path("order")
                        .push(
                            Router::new()
                                .get(handler::order::list_orders)
                                .post(handler::order::create_order),
                        )
                        .push(
                            Router::with_path("<id>")
                                .get(handler::order::get_order)
                                .push(
                                    Router::with_path("cancel").post(handler::order::cancel_order),
                                )
                                .push(
                                    Router::with_path("status")
                                        .get(handler::order::check_order_status),
                                )
                                .push(
                                    Router::with_path("refund").post(handler::order::refund_order),
                                )
                                .push(
                                    Router::with_path("complete")
                                        .post(handler::order::complete_order),
                                ),
                        ),
                )
                // 管理后台（需要 Admin 权限）
                .push(
                    Router::with_path("admin")
                        .hoop(require_admin) // 统一 Admin 权限检查
                        // Dashboard
                        .push(
                            Router::with_path("dashboard/stats")
                                .get(handler::admin::dashboard_stats),
                        )
                        .push(
                            Router::with_path("dashboard/revenue-chart")
                                .get(handler::admin::revenue_chart),
                        )
                        // 管理员管理
                        .push(
                            Router::new()
                                .get(handler::admin::get_current_admin)
                                .push(Router::with_path("list").get(handler::admin::list_admins))
                                .push(
                                    Router::with_path("create").post(handler::admin::create_admin),
                                )
                                .push(
                                    Router::with_path("api-keys")
                                        .get(handler::admin::list_api_keys)
                                        .post(handler::admin::create_api_key),
                                )
                                .push(
                                    Router::with_path("<key_id>/api-keys")
                                        .delete(handler::admin::delete_api_key),
                                ),
                        )
                        .push(
                            Router::with_path("<id>")
                                .get(handler::admin::get_admin)
                                .push(
                                    Router::with_path("deactivate")
                                        .post(handler::admin::deactivate_admin),
                                )
                                .push(
                                    Router::with_path("activate")
                                        .post(handler::admin::activate_admin),
                                )
                                .push(
                                    Router::with_path("reset-password")
                                        .post(handler::admin::reset_admin_password),
                                )
                                .push(
                                    Router::with_path("api-keys/<key_id>")
                                        .put(handler::admin::toggle_api_key_status),
                                ),
                        )
                        // 用户管理
                        .push(
                            Router::with_path("user")
                                .get(handler::admin::list_users)
                                .push(
                                    Router::with_path("<id>/deactivate")
                                        .post(handler::admin::deactivate_user),
                                )
                                .push(
                                    Router::with_path("<id>/activate")
                                        .post(handler::admin::activate_user),
                                ),
                        )
                        // 日志配置管理
                        .push(
                            Router::with_path("log-configs")
                                .get(handler::admin::list_log_configs)
                                .post(handler::admin::create_log_config),
                        )
                        .push(
                            Router::with_path("log-configs/<key>")
                                .get(handler::admin::get_log_config)
                                .put(handler::admin::update_log_config)
                                .delete(handler::admin::delete_log_config),
                        )
                        // 日志查询
                        .push(
                            Router::with_path("logs/system").get(handler::admin::query_system_logs),
                        )
                        // USDT 钱包配置
                        .push(
                            Router::with_path("usdt-wallets")
                                .get(handler::admin::list_usdt_wallets)
                                .push(
                                    Router::with_path("<network>")
                                        .put(handler::admin::update_usdt_wallet),
                                ),
                        )
                        // PayPal 配置管理
                        .push(
                            Router::with_path("paypal-configs")
                                .get(handler::admin_paypal::list_paypal_configs)
                                .push(
                                    Router::with_path("<id>")
                                        .get(handler::admin_paypal::get_paypal_config)
                                        .put(handler::admin_paypal::update_paypal_config)
                                        .push(
                                            Router::with_path("active/<active>").post(
                                                handler::admin_paypal::set_paypal_config_active,
                                            ),
                                        ),
                                ),
                        ),
                ),
        )
        // 管理员登录（无需 API Key，使用邮箱+密码）
        .push(Router::with_path("api/v1/admin/login").post(handler::admin::login))
        // 支付相关（无需 API Key 认证）
        .push(
            Router::with_path("api/v1/payment")
                .push(Router::with_path("usdt/<network>").get(handler::payment::get_usdt_address))
                .push(Router::with_path("paypal/success").get(handler::payment::paypal_success))
                .push(Router::with_path("paypal/cancel").get(handler::payment::paypal_cancel)),
        )
        // Webhook（无需 API Key 认证，有独立的签名验证）
        .push(
            Router::with_path("api/v1/webhook")
                .push(Router::with_path("paypal").post(handler::payment::paypal_webhook))
                .push(Router::with_path("usdt").post(handler::payment::usdt_webhook)),
        );

    // OpenAPI 文档生成
    let doc = OpenApi::new("RSWS API", "0.1.0").merge_router(&api_routes);

    // CORS 中间件 — 从配置读取允许的域名
    let cors_origins = &state.config.server.cors_origins;
    let cors = if cors_origins.contains(&"*".to_string()) || cors_origins.is_empty() {
        // 开发模式：允许所有来源（不建议在生产环境使用）
        tracing::warn!("CORS allow_origin set to '*' — not recommended for production");
        Cors::new()
            .allow_origin(Any)
            .allow_methods(vec![
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::OPTIONS,
            ])
            .allow_headers(vec![
                "Content-Type",
                "Authorization",
                "X-Api-Key",
                "X-Signature",
            ])
            .allow_credentials(false) // ⚠️ 当 allow_origin 为 * 时，必须为 false
            .max_age(3600)
    } else {
        // 生产模式：仅允许配置的域名（取第一个，salvo-cors 单域名支持更好）
        let allowed_origin = &cors_origins[0];
        tracing::info!("CORS allow_origin set to: {}", allowed_origin);
        Cors::new()
            .allow_origin(allowed_origin.as_str())
            .allow_methods(vec![
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::OPTIONS,
            ])
            .allow_headers(vec![
                "Content-Type",
                "Authorization",
                "X-Api-Key",
                "X-Signature",
            ])
            .allow_credentials(true)
            .max_age(3600)
    };

    Router::new()
        // Request ID 追踪（所有请求）
        .hoop(request_id_middleware)
        .hoop(cors.into_handler())
        .hoop(affix_state::inject(state))
        // Swagger UI
        .push(doc.into_router("/api-doc/openapi.json"))
        .push(SwaggerUi::new("/swagger-ui").into_router("/api-doc/openapi.json"))
        // 业务路由
        .push(api_routes)
}
