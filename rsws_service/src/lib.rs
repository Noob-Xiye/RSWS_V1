//! 服务层
//!
//! 提供各业务模块的服务逻辑

pub mod admin_service;
pub mod api_key_manager;
pub mod audit_log_service;
pub mod blockchain_service;
pub mod commission_service;
pub mod config_service;
pub mod cross_platform_service;
pub mod email_verification_service;
pub mod error_log_service;
pub mod log_service;
pub mod login_log_service;
pub mod order_service;
pub mod oss_service;
pub mod payment_service;
pub mod paypal_service;
pub mod request_service;
pub mod resource_service;
pub mod user_payment_service;
pub mod user_service;
pub mod webhook_service;

// 导出主要服务
pub use admin_service::AdminService;
pub use api_key_manager::ApiKeyManager;
pub use audit_log_service::{
    AuditAction, AuditLog, AuditLogPage, AuditLogQuery, AuditLogService, AuditStats,
    CreateAuditLogRequest, ResourceType, RiskLevel, VerificationMethod,
};
pub use blockchain_service::BlockchainService;
pub use commission_service::CommissionService;
pub use config_service::ConfigService;
pub use config_service::{BlockchainDbConfig, EmailDbConfig, PayPalDbConfig, UsdtListenDbConfig};
pub use cross_platform_service::CrossPlatformService;
pub use email_verification_service::EmailVerificationService;
pub use error_log_service::{
    CreateErrorLogRequest, ErrorLog, ErrorLogPage, ErrorLogQuery, ErrorLogService, ErrorStats,
    ErrorType, ResolveErrorRequest,
};
pub use log_service::LogService;
pub use log_service::{LogConfig, UpdateLogConfigRequest};
pub use login_log_service::{
    CreateLoginLogRequest, LoginLog, LoginLogPage, LoginLogQuery, LoginLogService, LoginStatus,
    LoginType,
};
pub use order_service::OrderService;
pub use oss_service::{FileMetadata, StorageBackend, StorageError, StorageService, UploadResult};
pub use payment_service::PaymentService;
pub use paypal_service::PayPalService;
pub use request_service::RequestService;
pub use resource_service::ResourceService;
pub use rsws_db::admin::AdminRepository;
pub use user_payment_service::UserPaymentService;
pub use user_service::UserService;
pub use webhook_service::WebhookService;

use rsws_db::{
    OrderRepository, PaymentRepository, RedisService, ResourceRepository, UserRepository,
    WalletRepository,
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

/// 创建 Admin API Key 管理器
pub fn create_admin_api_key_manager(redis: RedisService) -> ApiKeyManager {
    ApiKeyManager::for_admin(Arc::new(redis))
}

/// 创建 User API Key 管理器
pub fn create_user_api_key_manager(redis: RedisService) -> ApiKeyManager {
    ApiKeyManager::for_user(Arc::new(redis))
}

/// 创建用户服务
pub fn create_user_service(
    pool: sqlx::PgPool,
    redis: Option<RedisService>,
    email_config: Option<&EmailDbConfig>,
) -> UserService {
    let user_repo = UserRepository::new(pool);

    if let Some(redis) = redis {
        UserService::with_email_verification_service(user_repo, redis, email_config)
    } else {
        UserService::new(user_repo)
    }
}

/// 创建订单服务
pub fn create_order_service(pool: sqlx::PgPool) -> OrderService {
    OrderService::new(Arc::new(OrderRepository::new(pool)))
}

/// 创建资源服务
pub fn create_resource_service(
    pool: sqlx::PgPool,
    config_service: Option<ConfigService>,
    order_service: Option<Arc<OrderService>>,
) -> ResourceService {
    let mut service = if let Some(cfg) = config_service {
        ResourceService::with_oss(Arc::new(ResourceRepository::new(pool)), cfg)
    } else {
        ResourceService::new(Arc::new(ResourceRepository::new(pool)))
    };
    if let Some(os) = order_service {
        service.set_order_service(os);
    }
    service
}

/// 创建配置服务
pub fn create_config_service(pool: sqlx::PgPool, redis: RedisService) -> ConfigService {
    ConfigService::new(pool, redis)
}

/// 创建支付服务
pub fn create_payment_service(pool: sqlx::PgPool) -> PaymentService {
    PaymentService::new(Arc::new(PaymentRepository::new(pool)))
}

/// 创建管理员服务（不再持有 Redis，API Key 管理已迁移至 api_key_manager）
pub fn create_admin_service(pool: sqlx::PgPool) -> AdminService {
    AdminService::new(AdminRepository::new(pool))
}
