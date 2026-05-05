//! 服务层
//!
//! 提供各业务模块的服务逻辑

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
pub use api_key_service::ApiKeyService;
pub use auth_service::AuthService;
pub use blockchain_service::BlockchainService;
pub use commission_service::CommissionService;
pub use config_service::ConfigService;
pub use cross_platform_service::CrossPlatformService;
pub use log_service::LogService;
pub use order_service::OrderService;
pub use payment_service::PaymentService;
pub use paypal_service::PayPalService;
pub use request_service::RequestService;
pub use resource_service::ResourceService;
pub use user_payment_service::UserPaymentService;
pub use user_service::UserService;
pub use webhook_service::WebhookService;

use std::sync::Arc;
use rsws_db::{ApiKeyRepository, UserRepository, OrderRepository, ResourceRepository};
use rsws_db::RedisPool;
use rsws_common::config::{PayPalConfig, USDTConfig};

/// 创建 PayPal 服务
pub fn create_paypal_service(config: PayPalConfig) -> PayPalService {
    PayPalService::new(config)
}

/// 创建区块链服务
pub fn create_blockchain_service(config: USDTConfig) -> BlockchainService {
    BlockchainService::new(config)
}

/// 创建 Webhook 服务
pub fn create_webhook_service() -> WebhookService {
    WebhookService::new()
}

/// 创建跨平台服务
pub fn create_cross_platform_service() -> CrossPlatformService {
    CrossPlatformService::new()
}

/// 创建 API Key 服务
pub fn create_api_key_service(pool: sqlx::PgPool) -> ApiKeyService {
    // TODO: check ApiKeyService::new signature, may need Arc wrapping
    ApiKeyService::new(Arc::new(ApiKeyRepository::new(pool)))
}

/// 创建用户服务
pub fn create_user_service(
    pool: sqlx::PgPool,
    redis: Option<RedisPool>,
) -> UserService {
    let user_repo = UserRepository::new(pool);
    
    if let Some(redis) = redis {
        UserService::with_redis(user_repo, redis)
    } else {
        UserService::new(user_repo)
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
pub fn create_config_service(pool: sqlx::PgPool) -> ConfigService {
    ConfigService::new(pool)
}