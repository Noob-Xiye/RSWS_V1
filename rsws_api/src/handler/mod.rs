//! 处理器

pub mod admin;
pub mod admin_paypal;
pub mod category;
pub mod order;
pub mod payment;
pub mod resource;
pub mod upload;
pub mod user;

use salvo::prelude::*;
use salvo_oapi::endpoint;

/// 健康检查
#[endpoint]
pub async fn health(res: &mut Response) {
    res.render(Json(rsws_common::response::ApiResponse::success("OK")));
}
