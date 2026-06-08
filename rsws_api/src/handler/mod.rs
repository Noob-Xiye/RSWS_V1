//! 处理器

pub mod admin;
pub mod common;
pub mod custom;

use salvo::prelude::*;

/// 健康检查
#[salvo_oapi::endpoint]
pub async fn health(res: &mut Response) {
    res.render(Json(rsws_common::response::ApiResponse::success("OK")));
}
