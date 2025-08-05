use sqlx::PgPool;

pub mod api_key;
pub mod redis;
pub mod user;
pub mod order;
pub mod payment;
pub mod resource;

pub use user::UserRepository;
pub use order::OrderRepository;
pub use payment::PaymentRepository;
pub use resource::ResourceRepository;

pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPool::connect(database_url).await
}