//! 订单服务

use rust_decimal::Decimal;

use rsws_common::error::RswsError;
use rsws_common::error_code::ErrorCode;
use rsws_db::OrderRepository;
use rsws_model::payment::{Order, OrderDetail};
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
        amount: Decimal,
        payment_method: &str,
    ) -> Result<Order, RswsError> {
        // 检查金额
        if amount < Decimal::ZERO {
            return Err(RswsError::business(ErrorCode::PAYMENT_AMOUNT_INVALID));
        }

        let order = self
            .order_repo
            .create(user_id, resource_id, amount, payment_method, 30)
            .await?;

        info!("Order created: {}", order.id);

        Ok(order)
    }

    /// 获取订单
    pub async fn get(&self, order_id: i64) -> Result<Option<Order>, RswsError> {
        self.order_repo.get_by_id(order_id).await
    }

    /// 获取用户的订单列表
    pub async fn list_by_user(
        &self,
        user_id: i64,
        page: i32,
        limit: i32,
    ) -> Result<(Vec<Order>, i64), RswsError> {
        self.order_repo.list_by_user(user_id, page, limit).await
    }

    /// 获取用户的订单列表（包含资源标题）
    pub async fn list_detail_by_user(
        &self,
        user_id: i64,
        page: i32,
        page_size: i32,
    ) -> Result<(Vec<OrderDetail>, i64), RswsError> {
        self.order_repo
            .list_detail_by_user(user_id, page, page_size)
            .await
    }

    /// 管理员获取全部订单列表（含用户名、资源标题、支持筛选）
    pub async fn admin_list_orders(
        &self,
        status: Option<&str>,
        user_id: Option<i64>,
        payment_method: Option<&str>,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<rsws_model::payment::AdminOrderDetail>, i64), RswsError> {
        self.order_repo
            .list_all_detail(status, user_id, payment_method, page, page_size)
            .await
    }

    /// 更新订单状态
    pub async fn update_status(&self, order_id: i64, status: &str) -> Result<(), RswsError> {
        // 验证状态值
        let valid_statuses = ["pending", "paid", "completed", "cancelled", "refunded"];
        if !valid_statuses.contains(&status.to_lowercase().as_str()) {
            return Err(RswsError::business(ErrorCode::ORDER_STATUS_INVALID));
        }

        self.order_repo.update_status(order_id, status).await
    }

    /// 取消订单
    pub async fn cancel(&self, order_id: i64, user_id: i64) -> Result<(), RswsError> {
        let order = self
            .order_repo
            .get_by_id(order_id)
            .await?
            .ok_or_else(|| RswsError::business(ErrorCode::ORDER_NOT_FOUND))?;

        // 验证订单所有权
        if order.user_id != user_id {
            return Err(RswsError::business(ErrorCode::AUTH_PERMISSION_DENIED));
        }

        // 检查订单状态
        if order.status != "pending" {
            return Err(RswsError::business(ErrorCode::ORDER_STATUS_INVALID));
        }

        self.order_repo.update_status(order_id, "cancelled").await
    }

    /// 确认订单已支付
    pub async fn mark_paid(&self, order_id: i64, _payment_method: &str) -> Result<(), RswsError> {
        self.order_repo.update_status(order_id, "paid").await
    }

    /// 完成订单
    pub async fn complete(&self, order_id: i64) -> Result<(), RswsError> {
        self.order_repo.update_status(order_id, "completed").await
    }

    /// 退款订单
    pub async fn refund(&self, order_id: i64) -> Result<(), RswsError> {
        self.order_repo.update_status(order_id, "refunded").await
    }

    /// 检查用户是否已购买某资源（通过已完成订单）
    pub async fn check_purchased(&self, user_id: i64, resource_id: i64) -> Result<bool, RswsError> {
        self.order_repo
            .check_user_purchased(user_id, resource_id)
            .await
    }
}
