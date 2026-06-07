//! Dashboard 统计面板
//!
//! 控制台统计数据 + 收入图表

use crate::state::get_state;
use rsws_common::ResponseExt;
use rsws_common::RswsError;
use rsws_db::{order::OrderRepository, resource::ResourceRepository, user::UserRepository};
use rsws_model::user_models::admin::{DailyOrderCount, DashboardStats};
use salvo::prelude::*;
use salvo_oapi::endpoint;
use sqlx::PgPool;

/// 获取 Dashboard 统计面板数据
#[endpoint(
    responses(
        (status_code = 200, description = "成功"),
        (status_code = 403, description = "非管理员"),
    )
)]
pub async fn dashboard_stats(_req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let state = get_state(depot);
    let pool: PgPool = state.pool();

    let user_repo = UserRepository::new(pool.clone());
    let order_repo = OrderRepository::new(pool.clone());
    let resource_repo = ResourceRepository::new(pool.clone());

    // 并行查询所有统计数据
    let (user_result, order_result, resource_result) = tokio::join!(
        user_repo.get_basic_stats(),
        order_repo.get_basic_stats(),
        resource_repo.get_basic_stats(),
    );

    let (total_users, new_users_30d) = match user_result {
        Ok(v) => v,
        Err(e) => {
            res.error(e);
            return;
        }
    };

    let (total_orders, completed_orders, total_revenue, _orders_30d, revenue_30d) =
        match order_result {
            Ok(v) => v,
            Err(e) => {
                res.error(e);
                return;
            }
        };

    let (total_resources, active_resources, new_resources_30d) = match resource_result {
        Ok(v) => v,
        Err(e) => {
            res.error(e);
            return;
        }
    };

    // 查询过去30天订单趋势
    let orders_trend: Vec<DailyOrderCount> = match sqlx::query_as(
        r#"
        SELECT DATE(created_at AT TIME ZONE 'UTC')::text AS date, COUNT(*)::bigint AS count
        FROM orders
        WHERE created_at >= NOW() - INTERVAL '30 days'
        GROUP BY DATE(created_at AT TIME ZONE 'UTC')
        ORDER BY date ASC
        "#,
    )
    .fetch_all(&pool)
    .await
    {
        Ok(v) => v,
        Err(e) => {
            res.error(RswsError::internal(format!(
                "Failed to query orders trend: {}",
                e
            )));
            return;
        }
    };

    let stats = DashboardStats {
        total_users,
        new_users_30d,
        total_orders,
        completed_orders,
        total_revenue, // 单位：分，前端除以100转元
        revenue_30d,   // 单位：分，前端除以100转元
        total_resources,
        active_resources,
        new_resources_30d,
        orders_trend,
    };

    res.success(stats);
}

/// 收入图表
#[endpoint(
    parameters(
        ("days", Query, description = "天数，默认30天"),
    ),
    responses(
        (status_code = 200, description = "获取成功"),
    )
)]
pub async fn revenue_chart(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let state = get_state(depot);
    let pool = &state.pool;

    // 解析参数
    let days: i64 = req.query("days").unwrap_or(30).clamp(1, 365);

    // 查询每日收入
    // 使用 make_interval() 避免字符串拼接（SQL 注入风险）
    let rows: Vec<(String, i64)> = match sqlx::query_as(
        r#"
        SELECT DATE(paid_at AT TIME ZONE 'UTC')::text AS date, COALESCE(SUM(amount), 0)::bigint AS revenue
        FROM orders
        WHERE status IN ('paid', 'completed')
          AND paid_at >= NOW() - make_interval(days := $1::int)
        GROUP BY DATE(paid_at AT TIME ZONE 'UTC')
        ORDER BY date ASC
        "#,
    )
    .bind(days as i32)
    .fetch_all(pool)
    .await
    {
        Ok(v) => v,
        Err(e) => {
            eprintln!("REVENUE_CHART_ERROR: {}", e);
            res.error(RswsError::internal(format!("Failed to query revenue chart: {}", e)));
            return;
        }
    };

    let dates: Vec<String> = rows.iter().map(|(d, _)| d.clone()).collect();
    let revenues: Vec<i64> = rows.iter().map(|(_, r)| *r).collect();

    let chart = serde_json::json!({
        "dates": dates,
        "revenues": revenues,
    });

    res.success(chart);
}
