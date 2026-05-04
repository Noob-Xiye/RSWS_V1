//! 路由定义

use salvo::prelude::*;
use crate::handler;

/// 创建路由
pub fn create_router() -> Router {
    Router::new()
        // 健康检查
        .push(Router::with_path("health").get(handler::health))

        // API v1
        .push(Router::with_path("api/v1")
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

            // 支付相关
            .push(Router::with_path("payment")
                .push(Router::with_path("usdt/<network>").get(handler::payment::get_usdt_address))
                .push(Router::with_path("paypal/success").get(handler::payment::paypal_success))
                .push(Router::with_path("paypal/cancel").get(handler::payment::paypal_cancel))
            )

            // Webhook
            .push(Router::with_path("webhook")
                .push(Router::with_path("paypal").post(handler::payment::paypal_webhook))
                .push(Router::with_path("usdt").post(handler::payment::usdt_webhook))
            )
        )
}
