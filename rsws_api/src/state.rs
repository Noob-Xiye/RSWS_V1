//! 应用全局状态
//!
//! 持有所有 service 实例，通过 Salvo Depot 注入到 handler

use rsws_common::config::AppConfig;
use rsws_db::CategoryRepository;
use rsws_service::{
    AdminRepository, AdminService, ApiKeyService, BlockchainService, ConfigService,
    CrossPlatformService, LogService, OrderService, PayPalService, PaymentService, ResourceService,
    UserService, WebhookService,
};
use salvo::prelude::*;
use sqlx::PgPool;
use std::sync::Arc;

/// 应用全局状态
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: AppConfig,
    pub user_service: Arc<UserService>,
    pub order_service: Arc<OrderService>,
    pub resource_service: Arc<ResourceService>,
    pub api_key_service: Arc<ApiKeyService>,
    pub paypal_service: Arc<PayPalService>,
    pub payment_service: Arc<PaymentService>,
    pub blockchain_service: Arc<BlockchainService>,
    pub webhook_service: Arc<WebhookService>,
    pub cross_platform_service: Arc<CrossPlatformService>,
    pub config_service: Arc<ConfigService>,
    pub admin_service: Arc<AdminService>,
    pub log_service: Arc<LogService>,
    admin_repo: Arc<AdminRepository>,
    pub category_service: Arc<CategoryRepository>,
}

impl AppState {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        pool: PgPool,
        config: AppConfig,
        user_service: UserService,
        order_service: OrderService,
        resource_service: ResourceService,
        api_key_service: ApiKeyService,
        paypal_service: Arc<PayPalService>,
        payment_service: PaymentService,
        blockchain_service: BlockchainService,
        webhook_service: WebhookService,
        cross_platform_service: CrossPlatformService,
        config_service: ConfigService,
        admin_service: AdminService,
        log_service: LogService,
        admin_repo: AdminRepository,
        category_service: CategoryRepository,
    ) -> Self {
        Self {
            pool: pool.clone(),
            config,
            user_service: Arc::new(user_service),
            order_service: Arc::new(order_service),
            resource_service: Arc::new(resource_service),
            api_key_service: Arc::new(api_key_service),
            paypal_service,
            payment_service: Arc::new(payment_service),
            blockchain_service: Arc::new(blockchain_service),
            webhook_service: Arc::new(webhook_service),
            cross_platform_service: Arc::new(cross_platform_service),
            config_service: Arc::new(config_service),
            admin_service: Arc::new(admin_service),
            log_service: Arc::new(log_service),
            admin_repo: Arc::new(admin_repo),
            category_service: Arc::new(category_service),
        }
    }

    /// 返回数据库连接池（用于创建临时仓储实例）
    pub fn pool(&self) -> PgPool {
        self.pool.clone()
    }

    /// 克隆 AdminRepository 用于中间件更新 last_used
    pub fn admin_repo_clone(&self) -> Arc<AdminRepository> {
        self.admin_repo.clone()
    }
}

// ==================== Depot 辅助方法 ====================

/// 从 Depot 获取 AppState
pub fn get_state(depot: &Depot) -> AppState {
    depot
        .obtain::<AppState>()
        .cloned()
        .expect("AppState not found in Depot")
}

/// 从 Depot 获取已认证的用户 ID
pub fn get_user_id(depot: &Depot) -> Option<i64> {
    depot.get::<i64>("user_id").ok().copied()
}

/// 从 Depot 获取已认证的用户 ID（必须）
pub fn require_user_id(depot: &Depot) -> Result<i64, StatusCode> {
    get_user_id(depot).ok_or(StatusCode::UNAUTHORIZED)
}
