//! Database Repository
//!
//! Provides data access layer for each business domain

use sqlx::PgPool;

pub mod api_key;
pub mod user;
pub mod order;
pub mod payment;
pub mod resource;
pub mod redis;
pub mod wallet;
pub mod admin;
pub mod category;

pub use user::UserRepository;
pub use order::OrderRepository;
pub use payment::PaymentRepository;
pub use payment::PayPalConfigRepository;
pub use resource::ResourceRepository;
pub use api_key::ApiKeyRepository;
pub use redis::RedisService;
pub use wallet::WalletRepository;
pub use admin::AdminRepository;
pub use category::CategoryRepository;
pub use category::Category;

/// Redis connection pool alias
pub type RedisPool = RedisService;

/// Create database connection pool
pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPool::connect(database_url).await
}

/// Create database connection pool (with config)
pub async fn create_pool_with_config(
    database_url: &str,
    max_connections: u32,
) -> Result<PgPool, sqlx::Error> {
    sqlx::postgres::PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(database_url)
        .await
}
