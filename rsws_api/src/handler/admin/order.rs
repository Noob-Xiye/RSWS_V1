//! 管理员订单处理器

use crate::state::get_state;
use rsws_common::ResponseExt;
use salvo::prelude::*;
use salvo_oapi::endpoint;
use serde::Deserialize;

/// 管理员订单查询参数
#[derive(Debug, Deserialize)]
pub struct AdminOrderQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub status: Option<String>,
    pub user_id: Option<i64>,
    pub payment_method: Option<String>,
}

/// 管理员获取全部订单列表
#[endpoint]
pub async fn admin_list_orders(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let is_admin: bool = depot.get("is_admin").copied().unwrap_or(false);
    if !is_admin {
        res.http_error(StatusCode::FORBIDDEN, "Admin access required");
        return;
    }

    let query: AdminOrderQuery = req.parse_queries().unwrap_or(AdminOrderQuery {
        page: Some(1),
        page_size: Some(20),
        status: None,
        user_id: None,
        payment_method: None,
    });

    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).clamp(1, 100);

    let state = get_state(depot);

    match state
        .order_service
        .admin_list_orders(
            query.status.as_deref(),
            query.user_id,
            query.payment_method.as_deref(),
            page,
            page_size,
        )
        .await
    {
        Ok((orders, total)) => {
            let total_pages = if page_size > 0 {
                (total + page_size - 1) / page_size
            } else {
                0
            };
            res.success(serde_json::json!({
                "items": orders,
                "total": total,
                "page": page,
                "page_size": page_size,
                "total_pages": total_pages,
            }));
        }
        Err(e) => {
            res.error(e);
        }
    }
}
