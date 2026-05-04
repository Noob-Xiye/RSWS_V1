//! 资源处理器

use salvo::prelude::*;
use rsws_common::response::ApiResponse;
use serde::{Deserialize, Serialize};

/// 资源列表查询参数
#[derive(Debug, Deserialize)]
pub struct ResourceQuery {
    pub page: Option<i32>,
    pub limit: Option<i32>,
    pub category: Option<String>,
    pub search: Option<String>,
}

/// 资源信息
#[derive(Debug, Serialize)]
pub struct ResourceInfo {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub price: String,
    pub category: String,
    pub cover_image: Option<String>,
    pub download_count: i64,
    pub created_at: String,
}

/// 获取资源列表
#[handler]
pub async fn list_resources(req: &mut Request, res: &mut Response) {
    let query: ResourceQuery = req.parse_queries().unwrap_or(ResourceQuery {
        page: Some(1),
        limit: Some(20),
        category: None,
        search: None,
    });

    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(20);

    // TODO: 从数据库查询资源列表
    // let resources = resource_service.list(page, limit, query.category, query.search).await?;

    res.render(Json(ApiResponse::success(serde_json::json!({
        "items": [
            {
                "id": 1,
                "title": "Minecraft MOD Pack - Tech Edition",
                "description": "A comprehensive tech modpack with 200+ mods",
                "price": "9.99",
                "category": "modpack",
                "cover_image": "https://example.com/cover1.jpg",
                "download_count": 1500,
                "created_at": "2026-05-01T00:00:00Z"
            },
            {
                "id": 2,
                "title": "Minecraft MOD Pack - Magic Edition",
                "description": "A magical modpack with 150+ magic-themed mods",
                "price": "7.99",
                "category": "modpack",
                "cover_image": "https://example.com/cover2.jpg",
                "download_count": 800,
                "created_at": "2026-05-02T00:00:00Z"
            }
        ],
        "total": 2,
        "page": page,
        "limit": limit,
        "total_pages": 1
    }))));
}

/// 获取资源详情
#[handler]
pub async fn get_resource(req: &mut Request, res: &mut Response) {
    let id: i64 = req.param("id").unwrap_or(0);

    if id <= 0 {
        res.render(Json(ApiResponse::<()>::error_with_message(
            rsws_common::error_code::ErrorCode::INVALID_PARAMETER,
            "Invalid resource ID"
        )));
        return;
    }

    // TODO: 从数据库查询资源详情
    // let resource = resource_service.get(id).await?;

    res.render(Json(ApiResponse::success(serde_json::json!({
        "id": id,
        "title": "Minecraft MOD Pack - Tech Edition",
        "description": "A comprehensive tech modpack with 200+ mods including IndustrialCraft, BuildCraft, and more.",
        "price": "9.99",
        "category": "modpack",
        "cover_image": "https://example.com/cover1.jpg",
        "download_count": 1500,
        "file_size": "524288000",
        "version": "1.0.0",
        "minecraft_version": "1.20.1",
        "author": {
            "id": 1,
            "username": "admin"
        },
        "created_at": "2026-05-01T00:00:00Z",
        "updated_at": "2026-05-01T00:00:00Z"
    }))));
}

/// 创建资源 (C2C 模式)
#[handler]
pub async fn create_resource(req: &mut Request, res: &mut Response) {
    let body = req.parse_json::<serde_json::Value>().await;

    match body {
        Ok(data) => {
            // TODO: 验证用户权限并创建资源

            res.render(Json(ApiResponse::success(serde_json::json!({
                "id": 3,
                "title": data.get("title").and_then(|v| v.as_str()).unwrap_or("New Resource"),
                "message": "Resource created successfully"
            }))));
        }
        Err(e) => {
            res.render(Json(ApiResponse::<()>::error_with_message(
                rsws_common::error_code::ErrorCode::INVALID_REQUEST_FORMAT,
                format!("Invalid request: {}", e)
            )));
        }
    }
}

/// 更新资源
#[handler]
pub async fn update_resource(req: &mut Request, res: &mut Response) {
    let id: i64 = req.param("id").unwrap_or(0);
    let body = req.parse_json::<serde_json::Value>().await;

    match body {
        Ok(_data) => {
            // TODO: 验证权限并更新资源

            res.render(Json(ApiResponse::success(serde_json::json!({
                "id": id,
                "message": "Resource updated successfully"
            }))));
        }
        Err(e) => {
            res.render(Json(ApiResponse::<()>::error_with_message(
                rsws_common::error_code::ErrorCode::INVALID_REQUEST_FORMAT,
                format!("Invalid request: {}", e)
            )));
        }
    }
}

/// 删除资源
#[handler]
pub async fn delete_resource(req: &mut Request, res: &mut Response) {
    let id: i64 = req.param("id").unwrap_or(0);

    // TODO: 验证权限并删除资源

    res.render(Json(ApiResponse::success(serde_json::json!({
        "id": id,
        "message": "Resource deleted successfully"
    }))));
}
