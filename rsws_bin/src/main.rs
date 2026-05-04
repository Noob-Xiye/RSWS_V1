//! RSWS 主程序

use salvo::prelude::*;
use salvo::Server;
use rsws_api::router;
use rsws_common::config::load_config;
use rsws_common::error::RswsError;
use sqlx::postgres::PgPoolOptions;
use tracing::{info, error};

#[tokio::main]
async fn main() -> Result<(), RswsError> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    info!("Starting RSWS server...");

    // 加载配置
    let config = load_config().map_err(|e| {
        error!("Failed to load config: {}", e);
        RswsError::internal("Failed to load config")
    })?;

    info!("Config loaded: {}:{}", config.server.host, config.server.port);

    // 初始化数据库连接池
    let _pool = PgPoolOptions::new()
        .max_connections(config.database.max_connections)
        .min_connections(config.database.min_connections)
        .connect(&config.database.url)
        .await
        .map_err(|e| {
            error!("Failed to connect to database: {}", e);
            RswsError::internal("Failed to connect to database")
        })?;

    info!("Database connected");

    // 初始化 Redis 连接池
    let _redis_pool = rsws_db::RedisPool::new(&config.redis.url)?;

    info!("Redis connected");

    // 创建路由
    let router = router::create_router();

    // 启动服务器
    let addr = format!("{}:{}", config.server.host, config.server.port);
    info!("Server listening on http://{}", addr);

    // 绑定端口并启动服务
    let acceptor = TcpListener::new(&addr).bind().await;
    Server::new(acceptor).serve(router).await;

    Ok(())
}
