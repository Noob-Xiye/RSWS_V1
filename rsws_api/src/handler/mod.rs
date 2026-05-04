//! 处理器

pub mod user;
pub mod resource;
pub mod order;
pub mod payment;

use salvo::prelude::*;

/// 健康检查
#[handler]
pub async fn health(res: &mut Response) {
    res.render(Json(rsws_common::response::ApiResponse::success("OK")));
}
