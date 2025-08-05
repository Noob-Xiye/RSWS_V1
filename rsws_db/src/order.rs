use sqlx::{PgPool, Row};
use rsws_model::payment::{Order, OrderStatus, OrderResponse};
use rsws_common::error::ServiceError;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

pub struct OrderRepository {
    pool: PgPool,
}

impl OrderRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // 创建订单
    pub async fn create(&self, order: &Order) -> Result<Order, ServiceError> {
        let result = sqlx::query_as!(
            Order,
            r#"
            INSERT INTO orders (id, user_id, resource_id, amount, status, payment_method, created_at, updated_at, expired_at)
            VALUES ($1, $2, $3, $4, $5::order_status, $6, $7, $8, $9)
            RETURNING id, user_id, resource_id, amount, status as "status: OrderStatus", payment_method, created_at, updated_at, expired_at
            "#,
            order.id,
            order.user_id,
            order.resource_id,
            order.amount,
            order.status as OrderStatus,
            order.payment_method,
            order.created_at,
            order.updated_at,
            order.expired_at
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            if e.to_string().contains("duplicate key") {
                ServiceError::BusinessError("您已经购买过该资源".to_string())
            } else {
                ServiceError::DatabaseError(e.to_string())
            }
        })?;

        Ok(result)
    }

    // 根据ID获取订单
    pub async fn get_by_id(&self, id: i64) -> Result<Option<Order>, ServiceError> {
        let result = sqlx::query_as!(
            Order,
            r#"
            SELECT id, user_id, resource_id, amount, status as "status: OrderStatus", payment_method, created_at, updated_at, expired_at
            FROM orders 
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        Ok(result)
    }

    // 获取用户订单列表
    pub async fn get_user_orders(
        &self,
        user_id: i64,
        status: Option<OrderStatus>,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<Order>, i64), ServiceError> {
        let offset = (page - 1) * page_size;
        
        let mut query = "SELECT id, user_id, resource_id, amount, status, payment_method, created_at, updated_at, expired_at FROM orders WHERE user_id = $1".to_string();
        let mut params: Vec<Box<dyn sqlx::Encode<'_, sqlx::Postgres> + Send + Sync>> = vec![Box::new(user_id)];
        let mut param_count = 1;

        if let Some(status) = status {
            param_count += 1;
            query.push_str(&format!(" AND status = ${}", param_count));
            params.push(Box::new(status));
        }

        if let Some(start_date) = start_date {
            param_count += 1;
            query.push_str(&format!(" AND created_at >= ${}", param_count));
            params.push(Box::new(start_date));
        }

        if let Some(end_date) = end_date {
            param_count += 1;
            query.push_str(&format!(" AND created_at <= ${}", param_count));
            params.push(Box::new(end_date));
        }

        query.push_str(" ORDER BY created_at DESC");
        query.push_str(&format!(" LIMIT {} OFFSET {}", page_size, offset));

        // 获取订单列表
        let orders = sqlx::query_as::<_, Order>(&query)
            .bind(user_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        // 获取总数
        let mut count_query = "SELECT COUNT(*) FROM orders WHERE user_id = $1".to_string();
        let mut count_params = 1;

        if status.is_some() {
            count_params += 1;
            count_query.push_str(&format!(" AND status = ${}", count_params));
        }
        if start_date.is_some() {
            count_params += 1;
            count_query.push_str(&format!(" AND created_at >= ${}", count_params));
        }
        if end_date.is_some() {
            count_params += 1;
            count_query.push_str(&format!(" AND created_at <= ${}", count_params));
        }

        let total: i64 = sqlx::query_scalar(&count_query)
            .bind(user_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        Ok((orders, total))
    }

    // 更新订单状态
    pub async fn update_order_status(
        &self,
        order_id: i64,
        status: OrderStatus,
    ) -> Result<(), ServiceError> {
        sqlx::query!(
            "UPDATE orders SET status = $1::order_status, updated_at = NOW() WHERE id = $2",
            status as OrderStatus,
            order_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    // 检查用户是否已购买资源
    pub async fn check_user_purchased(
        &self,
        user_id: i64,
        resource_id: i64,
    ) -> Result<bool, ServiceError> {
        let count: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM orders WHERE user_id = $1 AND resource_id = $2 AND status IN ('paid', 'completed')",
            user_id,
            resource_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        Ok(count > 0)
    }

    // 清理过期订单
    pub async fn cleanup_expired_orders(&self) -> Result<i64, ServiceError> {
        let result = sqlx::query!(
            "UPDATE orders SET status = 'cancelled'::order_status WHERE status = 'pending'::order_status AND expired_at < NOW()"
        )
        .execute(&self.pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        Ok(result.rows_affected() as i64)
    }
}