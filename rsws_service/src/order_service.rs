use rsws_db::{OrderRepository, ResourceRepository};
use rsws_model::payment::*;
use rsws_model::resource::Resource;
use rsws_common::error::ServiceError;
use rsws_common::snowflake;
use chrono::{Duration, Utc};
use rust_decimal::Decimal;
use std::sync::Arc;

pub struct OrderService {
    order_repo: Arc<OrderRepository>,
    resource_repo: Arc<ResourceRepository>,
}

impl OrderService {
    pub fn new(
        order_repo: Arc<OrderRepository>,
        resource_repo: Arc<ResourceRepository>,
    ) -> Self {
        Self {
            order_repo,
            resource_repo,
        }
    }
    
    // 创建订单
    pub async fn create_order(
        &self,
        user_id: i64,
        request: CreateOrderRequest,
    ) -> Result<OrderResponse, ServiceError> {
        // 检查资源是否存在
        let resource = self.resource_repo
            .get_by_id(request.resource_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("资源不存在".to_string()))?;
            
        // 检查用户是否已经购买过该资源
        if self.order_repo
            .check_user_purchased(user_id, request.resource_id)
            .await? {
            return Err(ServiceError::BusinessError("您已经购买过该资源".to_string()));
        }
        
        // 创建订单
        let order_id = snowflake::next_id();
        let expires_at = Utc::now() + Duration::minutes(30); // 30分钟过期
        
        let order = Order {
            id: order_id,
            user_id,
            resource_id: request.resource_id,
            amount: resource.price,
            status: OrderStatus::Pending,
            payment_method: request.payment_method,
            payment_id: None,
            transaction_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            completed_at: None,
            expired_at: Some(expires_at),
            notes: None,
        };
        
        self.order_repo.create_order(&order).await?;
        
        Ok(OrderResponse {
            id: order.id,
            user_id: order.user_id,
            resource_id: order.resource_id,
            resource_title: resource.title,
            amount: order.amount,
            status: order.status,
            payment_method: order.payment_method,
            created_at: order.created_at,
            expired_at: order.expired_at,
        })
    }
    
    // 获取订单详情
    pub async fn get_order_by_id(
        &self,
        order_id: i64,
        user_id: Option<i64>,
    ) -> Result<OrderResponse, ServiceError> {
        let order = self.order_repo
            .get_by_id(order_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("订单不存在".to_string()))?;
            
        // 检查权限
        if let Some(uid) = user_id {
            if order.user_id != uid {
                return Err(ServiceError::Forbidden("无权访问该订单".to_string()));
            }
        }
        
        let resource = self.resource_repo
            .get_by_id(order.resource_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("关联资源不存在".to_string()))?;
            
        Ok(OrderResponse {
            id: order.id,
            user_id: order.user_id,
            resource_id: order.resource_id,
            resource_title: resource.title,
            amount: order.amount,
            status: order.status,
            payment_method: order.payment_method,
            created_at: order.created_at,
            expired_at: order.expired_at,
        })
    }
    
    // 取消订单
    pub async fn cancel_order(
        &self,
        order_id: i64,
        user_id: i64,
    ) -> Result<(), ServiceError> {
        let order = self.order_repo
            .get_by_id(order_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("订单不存在".to_string()))?;
            
        if order.user_id != user_id {
            return Err(ServiceError::Forbidden("无权操作该订单".to_string()));
        }
        
        if order.status != OrderStatus::Pending {
            return Err(ServiceError::BusinessError("只能取消待支付的订单".to_string()));
        }
        
        self.order_repo
            .update_order_status(order_id, OrderStatus::Cancelled)
            .await?;
            
        Ok(())
    }
    
    // 获取用户订单列表
    pub async fn get_user_orders(
        &self,
        user_id: i64,
        request: OrderListRequest,
    ) -> Result<PaginatedResponse<OrderResponse>, ServiceError> {
        let page = request.page.unwrap_or(1);
        let page_size = request.page_size.unwrap_or(10);
        
        let (orders, total) = self.order_repo
            .get_user_orders(
                user_id,
                request.status,
                request.start_date,
                request.end_date,
                page,
                page_size,
            )
            .await?;
            
        let mut order_responses = Vec::new();
        for order in orders {
            let resource = self.resource_repo
                .get_by_id(order.resource_id)
                .await?
                .unwrap_or_else(|| Resource {
                    id: order.resource_id,
                    title: "已删除的资源".to_string(),
                    // ... 其他默认字段
                });
                
            order_responses.push(OrderResponse {
                id: order.id,
                user_id: order.user_id,
                resource_id: order.resource_id,
                resource_title: resource.title,
                amount: order.amount,
                status: order.status,
                payment_method: order.payment_method,
                created_at: order.created_at,
                expired_at: order.expired_at,
            });
        }
        
        Ok(PaginatedResponse {
            data: order_responses,
            total,
            page,
            page_size,
            total_pages: (total + page_size as i64 - 1) / page_size as i64,
        })
    }
    
    // 更新订单状态
    pub async fn update_order_status(
        &self,
        order_id: i64,
        status: OrderStatus,
    ) -> Result<(), ServiceError> {
        self.order_repo
            .update_order_status(order_id, status)
            .await?;
            
        // 如果订单完成，更新完成时间
        if status == OrderStatus::Completed {
            self.order_repo
                .update_completed_at(order_id, Utc::now())
                .await?;
        }
        
        Ok(())
    }
}