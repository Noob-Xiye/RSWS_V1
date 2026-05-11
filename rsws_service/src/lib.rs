//! 服务层
//!
//! 提供各业务模块的服务逻辑

pub mod admin_service;
pub mod api_key_service;
pub mod auth_service;
pub mod blockchain_service;
pub mod commission_service;
pub mod config_service;
pub mod cross_platform_service;
pub mod log_service;
pub mod order_service;
pub mod payment_service;
pub mod paypal_service;
pub mod request_service;
pub mod resource_service;
pub mod user_payment_service;
pub mod user_service;
pub mod webhook_service;

// 导出主要服务
pub use admin_service::AdminService;
pub use api_key_service::ApiKeyService;
pub use auth_service::AuthService;
pub use blockchain_service::BlockchainService;
pub use commission_service::CommissionService;
pub use config_service::ConfigService;
pub use config_service::{BlockchainDbConfig, EmailDbConfig, PayPalDbConfig, UsdtListenDbConfig};
pub use cross_platform_service::CrossPlatformService;
pub use log_service::LogService;
pub use log_service::{LogConfig, UpdateLogConfigRequest};
pub use order_service::OrderService;
pub use payment_service::PaymentService;
pub use paypal_service::PayPalService;
pub use request_service::RequestService;
pub use resource_service::ResourceService;
pub use rsws_db::admin::AdminRepository;
pub use user_payment_service::UserPaymentService;
pub use user_service::UserService;
pub use webhook_service::WebhookService;

use rsws_db::{
    ApiKeyRepository, OrderRepository, PaymentRepository, RedisService, ResourceRepository,
    UserRepository, WalletRepository,
};
use std::sync::Arc;

/// 创建 PayPal 服务（配置从数据库读取）
pub fn create_paypal_service(
    config: Option<crate::config_service::PayPalDbConfig>,
) -> PayPalService {
    PayPalService::new(config)
}

/// 创建区块链服务（配置从数据库读取）
pub fn create_blockchain_service(wallet_repo: WalletRepository) -> BlockchainService {
    BlockchainService::new(wallet_repo)
}

/// 创建 Webhook 服务
pub fn create_webhook_service(paypal_service: Arc<PayPalService>) -> WebhookService {
    WebhookService::new(paypal_service)
}

/// 创建跨平台服务
pub fn create_cross_platform_service() -> CrossPlatformService {
    CrossPlatformService::new()
}

/// 创建 API Key 服务（已自动使用 Arc<ApiKeyRepository>）
pub fn create_api_key_service(pool: sqlx::PgPool) -> ApiKeyService {
    ApiKeyService::new(Arc::new(ApiKeyRepository::new(pool)))
}

/// 创建用户服务
pub fn create_user_service(pool: sqlx::PgPool, redis: Option<RedisService>) -> UserService {
    let user_repo = UserRepository::new(pool);

    if let Some(redis) = redis {
        UserService::with_redis(user_repo, redis)
    } else {
        UserService::new(user_repo)
    }
}

/// 创建用户服务（带 Email）
pub fn create_user_service_with_email(
    pool: sqlx::PgPool,
    redis: Option<RedisService>,
    email_service: rsws_common::email::EmailService,
) -> UserService {
    let user_repo = UserRepository::new(pool);
    if let Some(redis) = redis {
        UserService::with_services(user_repo, redis, email_service)
    } else {
        // 没有 Redis 的情况，用 with_email
        UserService::with_redis_and_email(user_repo, email_service)
    }
}

/// 创建订单服务
pub fn create_order_service(pool: sqlx::PgPool) -> OrderService {
    OrderService::new(Arc::new(OrderRepository::new(pool)))
}

/// 创建资源服务
pub fn create_resource_service(pool: sqlx::PgPool) -> ResourceService {
    ResourceService::new(Arc::new(ResourceRepository::new(pool)))
}

/// 创建配置服务
pub fn create_config_service(pool: sqlx::PgPool, redis: RedisService) -> ConfigService {
    ConfigService::new(pool, redis)
}

/// 创建支付服务
pub fn create_payment_service(pool: sqlx::PgPool) -> PaymentService {
    PaymentService::new(Arc::new(PaymentRepository::new(pool)))
}

/// 创建管理员服务
pub fn create_admin_service(pool: sqlx::PgPool, redis: Option<RedisService>) -> AdminService {
    if let Some(redis) = redis {
        AdminService::with_redis(AdminRepository::new(pool), redis)
    } else {
        AdminService::new(AdminRepository::new(pool))
    }
}
