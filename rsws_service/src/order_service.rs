//! 订单服务

use rsws_common::error::RswsError;
use rsws_db::OrderRepository;
use std::sync::Arc;
use tracing::info;

/// 订单服务
pub struct OrderService {
    order_repo: Arc<OrderRepository>,
}

impl OrderService {
    /// 创建订单服务实例
    pub fn new(order_repo: Arc<OrderRepository>) -> Self {
        Self { order_repo }
    }

    /// 创建订单
    pub async fn create(
        &self,
        user_id: i64,
        resource_id: i64,
        amount: i64,
        payment_method: &str,
    ) -> Result<i64, RswsError> {
        let order = self.order_repo
            .create(user_id, resource_id, amount, payment_method, 30)
            .await?;

        info!("Order created: {}", order.id);

        Ok(order.id)
    }

    /// 获取订单
    pub async fn get(&self, order_id: i64) -> Result<Option<rsws_model::payment::Order>, RswsError> {
        self.order_repo.get_by_id(order_id).await
    }

    /// 更新订单状态
    pub async fn update_status(&self, order_id: i64, status: &str) -> Result<(), RswsError> {
        self.order_repo.update_status(order_id, status).await
    }
}
