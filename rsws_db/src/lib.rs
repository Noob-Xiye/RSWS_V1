//! 数据库层
//!
//! 提供各业务模块的数据库操作

use sqlx::PgPool;

pub mod api_key;
pub mod user;
pub mod order;
pub mod payment;
pub mod resource;
pub mod redis;
pub mod wallet;
pub mod admin;

pub use user::UserRepository;
pub use order::OrderRepository;
pub use payment::PaymentRepository;
pub use resource::ResourceRepository;
pub use api_key::ApiKeyRepository;
pub use redis::RedisService;
pub use wallet::WalletRepository;
pub use admin::AdminRepository;

/// Redis 连接池类型别名
pub type RedisPool = RedisService;

/// 创建数据库连接池
pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPool::connect(database_url).await
}

/// 创建数据库连接池（带配置）
pub async fn create_pool_with_config(
    database_url: &str,
    max_connections: u32,
) -> Result<PgPool, sqlx::Error> {
    sqlx::postgres::PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(database_url)
        .await
}
