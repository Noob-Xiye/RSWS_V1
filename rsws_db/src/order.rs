//! 订单仓储层

use rsws_common::error::RswsError;
use rsws_common::error_code::ErrorCode;
use rsws_common::snowflake;
use rsws_model::payment::{Order, OrderDetail};
use sqlx::PgPool;

/// 订单仓储
pub struct OrderRepository {
    pool: PgPool,
}

impl OrderRepository {
    /// 创建订单仓储实例
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 创建订单
    pub async fn create(
        &self,
        user_id: i64,
        resource_id: i64,
        amount: i64,
        payment_method: &str,
        expire_minutes: i32,
    ) -> Result<Order, RswsError> {
        let order_id = snowflake::next_id();

        let order = sqlx::query_as::<_, Order>(
            r#"
            INSERT INTO orders (id, user_id, resource_id, amount, status, payment_method, created_at, updated_at, expired_at)
            VALUES ($1, $2, $3, $4, 'pending', $5, NOW(), NOW(), NOW() + INTERVAL '1 minute' * $6)
            RETURNING id, user_id, resource_id, amount, status, payment_method, created_at, updated_at, expired_at
            "#,
        )
        .bind(order_id)
        .bind(user_id)
        .bind(resource_id)
        .bind(amount)
        .bind(payment_method)
        .bind(expire_minutes)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            if e.to_string().contains("duplicate key") {
                RswsError::business(ErrorCode::ORDER_ALREADY_EXISTS)
            } else {
                RswsError::internal(format!("Failed to create order: {}", e))
            }
        })?;

        Ok(order)
    }

    /// 根据 ID 获取订单
    pub async fn get_by_id(&self, id: i64) -> Result<Option<Order>, RswsError> {
        let order = sqlx::query_as::<_, Order>(
            "SELECT id, user_id, resource_id, amount, status, payment_method, created_at, updated_at, expired_at FROM orders WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to get order: {}", e)))?;

        Ok(order)
    }

    /// 获取用户订单列表
    pub async fn get_user_orders(
        &self,
        user_id: i64,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<Order>, i64), RswsError> {
        let offset = (page - 1) * page_size;

        // 获取订单列表
        let orders = sqlx::query_as::<_, Order>(
            r#"
            SELECT id, user_id, resource_id, amount, status, payment_method, created_at, updated_at, expired_at
            FROM orders
            WHERE user_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(user_id)
        .bind(page_size)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to get orders: {}", e)))?;

        // 获取总数
        let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM orders WHERE user_id = $1")
            .bind(user_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| RswsError::internal(format!("Failed to count orders: {}", e)))?;

        Ok((orders, total.0))
    }

    /// 更新订单状态
    pub async fn update_status(&self, order_id: i64, status: &str) -> Result<(), RswsError> {
        sqlx::query(
            "UPDATE orders SET status = $1::order_status, updated_at = NOW() WHERE id = $2",
        )
        .bind(status)
        .bind(order_id)
        .execute(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to update order status: {}", e)))?;

        Ok(())
    }

    /// 获取用户订单列表
    pub async fn list_by_user(
        &self,
        user_id: i64,
        page: i32,
        limit: i32,
    ) -> Result<(Vec<Order>, i64), RswsError> {
        self.get_user_orders(user_id, page as i64, limit as i64)
            .await
    }

    /// 获取用户订单列表（包含资源标题）
    pub async fn list_detail_by_user(
        &self,
        user_id: i64,
        page: i32,
        limit: i32,
    ) -> Result<(Vec<OrderDetail>, i64), RswsError> {
        let offset = (page as i64 - 1) * limit as i64;

        // JOIN resources 表获取标题
        let orders = sqlx::query_as::<_, OrderDetail>(
            r#"
            SELECT o.id, o.user_id, o.resource_id, o.amount, o.status, o.payment_method,
                   o.created_at, o.updated_at, o.expired_at, r.title as resource_title
            FROM orders o
            LEFT JOIN resources r ON o.resource_id = r.id
            WHERE o.user_id = $1
            ORDER BY o.created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(user_id)
        .bind(limit as i64)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to get orders with details: {}", e)))?;

        // 获取总数
        let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM orders WHERE user_id = $1")
            .bind(user_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| RswsError::internal(format!("Failed to count orders: {}", e)))?;

        Ok((orders, total.0))
    }

    /// 检查用户是否已购买资源
    pub async fn check_user_purchased(
        &self,
        user_id: i64,
        resource_id: i64,
    ) -> Result<bool, RswsError> {
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM orders WHERE user_id = $1 AND resource_id = $2 AND status IN ('paid', 'completed')",
        )
        .bind(user_id)
        .bind(resource_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to check purchase: {}", e)))?;

        Ok(count.0 > 0)
    }

    /// 清理过期订单
    pub async fn cleanup_expired(&self) -> Result<u64, RswsError> {
        let result = sqlx::query(
            "UPDATE orders SET status = 'cancelled'::order_status WHERE status = 'pending'::order_status AND expired_at < NOW()",
        )
        .execute(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to cleanup orders: {}", e)))?;

        Ok(result.rows_affected())
    }

    /// 获取基础统计（订单总数 + 已完成订单数 + 总收入 + 过去30天订单数 + 过去30天收入）
    pub async fn get_basic_stats(&self) -> Result<(i64, i64, i64, i64, i64), RswsError> {
        let total_orders: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM orders")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| RswsError::internal(format!("Failed to count orders: {}", e)))?;

        let completed_orders: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM orders WHERE status IN ('paid', 'completed')")
                .fetch_one(&self.pool)
                .await
                .map_err(|e| {
                    RswsError::internal(format!("Failed to count completed orders: {}", e))
                })?;

        let total_revenue: (i64,) = sqlx::query_as(
            "SELECT COALESCE(SUM(amount), 0) FROM orders WHERE status IN ('paid', 'completed')",
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to sum revenue: {}", e)))?;

        let orders_30d: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM orders WHERE created_at >= NOW() - INTERVAL '30 days'",
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to count recent orders: {}", e)))?;

        let revenue_30d: (i64,) = sqlx::query_as(
            "SELECT COALESCE(SUM(amount), 0) FROM orders WHERE status IN ('paid', 'completed') AND created_at >= NOW() - INTERVAL '30 days'"
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to sum recent revenue: {}", e)))?;

        Ok((
            total_orders.0,
            completed_orders.0,
            total_revenue.0,
            orders_30d.0,
            revenue_30d.0,
        ))
    }
}

// ==================== 单元测试 ====================

#[cfg(test)]
mod tests {

    #[test]
    fn test_order_repository_new() {
        // 仅测试构造函数
    }
}
