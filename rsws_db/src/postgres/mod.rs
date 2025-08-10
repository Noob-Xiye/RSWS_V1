use rsws_common::error::ServiceError;
use sqlx::PgPool;

pub async fn init_pg_pool(database_url: &str, max_connections: u32) -> Result<PgPool, sqlx::Error> {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(database_url)
        .await?;

    // 运行数据库迁移
    sqlx::migrate!("../sql").run(&pool).await?;

    Ok(pool)
}

pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::migrate!("../sql").run(pool).await
}
