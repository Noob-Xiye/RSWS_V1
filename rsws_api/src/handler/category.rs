//! Category handler

use crate::state::get_state;
use rsws_common::{ResponseExt, RswsError};
use rsws_db::category::CategoryRepository;
use salvo::prelude::*;
use salvo_oapi::endpoint;
use serde::Deserialize;

// ========== 公开端点（无需管理员权限）==========

/// Get category list (仅活跃分类)
#[endpoint(
    responses(
        (status_code = 200, description = "Category list"),
    )
)]
pub async fn list_categories(_req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let state = get_state(depot);
    let repo = CategoryRepository::new(state.pool());

    match repo.find_all().await {
        Ok(categories) => res.success(serde_json::json!({
            "categories": categories
        })),
        Err(e) => res.error(RswsError::Database(e)),
    }
}

// ========== 管理端点（需要 Admin 权限，由 router 层 require_admin 中间件保证）==========

#[derive(Debug, Deserialize)]
pub struct CreateCategoryRequest {
    pub name: String,
    pub description: Option<String>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCategoryRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub sort_order: Option<i32>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct BatchSortRequest {
    pub orders: Vec<SortItem>,
}

#[derive(Debug, Deserialize)]
pub struct SortItem {
    pub id: i64,
    pub sort_order: i32,
}

/// 管理员 - 获取所有分类（含已停用）
#[endpoint]
pub async fn admin_list_categories(_req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let state = get_state(depot);
    let repo = CategoryRepository::new(state.pool());

    match repo.find_all_with_inactive().await {
        Ok(categories) => {
            // 并行查询每个分类的资源数量
            let mut categories_with_count = Vec::with_capacity(categories.len());
            for cat in categories {
                let count = repo.count_resources(cat.id).await.unwrap_or(0);
                categories_with_count.push(serde_json::json!({
                    "id": cat.id,
                    "name": cat.name,
                    "description": cat.description,
                    "sort_order": cat.sort_order,
                    "is_active": cat.is_active,
                    "resource_count": count,
                    "created_at": cat.created_at,
                    "updated_at": cat.updated_at,
                }));
            }
            res.success(serde_json::json!({
                "categories": categories_with_count
            }))
        }
        Err(e) => res.error(RswsError::Database(e)),
    }
}

/// 管理员 - 创建分类
#[endpoint]
pub async fn create_category(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let state = get_state(depot);
    let repo = CategoryRepository::new(state.pool());

    let body: CreateCategoryRequest = match req.parse_json().await {
        Ok(b) => b,
        Err(_) => {
            res.http_error(salvo::http::StatusCode::BAD_REQUEST, "请求参数格式错误");
            return;
        }
    };

    if body.name.trim().is_empty() {
        res.http_error(salvo::http::StatusCode::BAD_REQUEST, "分类名称不能为空");
        return;
    }

    // 检查名称是否重复
    if let Ok(Some(_)) = repo.find_by_name(body.name.trim()).await {
        res.http_error(salvo::http::StatusCode::CONFLICT, "分类名称已存在");
        return;
    }

    let max_order = repo.max_sort_order().await.unwrap_or(0);
    let sort_order = body.sort_order.unwrap_or(max_order + 1);

    match repo.create(body.name.trim(), body.description.as_deref(), sort_order).await {
        Ok(category) => res.success(category),
        Err(e) => res.error(RswsError::Database(e)),
    }
}

/// 管理员 - 更新分类
#[endpoint]
pub async fn update_category(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let state = get_state(depot);
    let repo = CategoryRepository::new(state.pool());

    let id: i64 = match req.param("id").and_then(|v: &str| v.parse::<i64>().ok()) {
        Some(v) => v,
        None => {
            res.http_error(salvo::http::StatusCode::BAD_REQUEST, "无效的分类ID");
            return;
        }
    };

    let body: UpdateCategoryRequest = match req.parse_json().await {
        Ok(b) => b,
        Err(_) => {
            res.http_error(salvo::http::StatusCode::BAD_REQUEST, "请求参数格式错误");
            return;
        }
    };

    // 如果更新名称，检查重复
    if let Some(ref name) = body.name {
        if name.trim().is_empty() {
            res.http_error(salvo::http::StatusCode::BAD_REQUEST, "分类名称不能为空");
            return;
        }
        if let Ok(Some(existing)) = repo.find_by_name(name.trim()).await {
            if existing.id != id {
                res.http_error(salvo::http::StatusCode::CONFLICT, "分类名称已存在");
                return;
            }
        }
    }

    // 将 description 的 Option<String> 转为 Option<Option<&str>>
    let desc_ref = body.description.as_deref();

    match repo
        .update(
            id,
            body.name.as_deref().map(|s| s.trim()),
            Some(desc_ref),
            body.sort_order,
            body.is_active,
        )
        .await
    {
        Ok(Some(category)) => res.success(category),
        Ok(None) => res.http_error(salvo::http::StatusCode::NOT_FOUND, "分类不存在"),
        Err(e) => res.error(RswsError::Database(e)),
    }
}

/// 管理员 - 删除分类
#[endpoint]
pub async fn delete_category(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let state = get_state(depot);
    let repo = CategoryRepository::new(state.pool());

    let id: i64 = match req.param("id").and_then(|v: &str| v.parse::<i64>().ok()) {
        Some(v) => v,
        None => {
            res.http_error(salvo::http::StatusCode::BAD_REQUEST, "无效的分类ID");
            return;
        }
    };

    // 检查分类下是否有资源
    match repo.count_resources(id).await {
        Ok(count) if count > 0 => {
            res.http_error(
                salvo::http::StatusCode::CONFLICT,
                format!("该分类下还有 {} 个资源，无法删除。请先将资源移至其他分类或删除。", count),
            );
            return;
        }
        Err(e) => {
            res.error(RswsError::Database(e));
            return;
        }
        Ok(_) => {}
    }

    match repo.delete(id).await {
        Ok(true) => res.ok(),
        Ok(false) => res.http_error(salvo::http::StatusCode::NOT_FOUND, "分类不存在"),
        Err(e) => res.error(RswsError::Database(e)),
    }
}

/// 管理员 - 批量更新排序
#[endpoint]
pub async fn batch_update_sort(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let state = get_state(depot);
    let repo = CategoryRepository::new(state.pool());

    let body: BatchSortRequest = match req.parse_json().await {
        Ok(b) => b,
        Err(_) => {
            res.http_error(salvo::http::StatusCode::BAD_REQUEST, "请求参数格式错误");
            return;
        }
    };

    if body.orders.is_empty() {
        res.http_error(salvo::http::StatusCode::BAD_REQUEST, "排序数据不能为空");
        return;
    }

    let orders: Vec<(i64, i32)> = body.orders.iter().map(|item| (item.id, item.sort_order)).collect();

    match repo.batch_update_sort_order(&orders).await {
        Ok(()) => res.ok(),
        Err(e) => res.error(RswsError::Database(e)),
    }
}
