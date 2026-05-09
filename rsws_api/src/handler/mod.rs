//! 处理器

pub mod user;
pub mod resource;
pub mod order;
pub mod payment;
pub mod admin;
pub mod admin_paypal;
pub mod category;

use salvo::prelude::*;
use salvo_oapi::endpoint;

/// 健康检查
#[endpoint]
pub async fn health(res: &mut Response) {
    res.render(Json(rsws_common::response::ApiResponse::success("OK")));
}
