use salvo::prelude::*;
use std::sync::Arc;

use rsws_common::config::load_config;
use rsws_db::postgres;
use rsws_db::redis as redis_db;
use rsws_api::router;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();
    
    // 加载配置
    let app_config = load_config().expect("Failed to load configuration");
    
    // 初始化PgPool
    let pg_pool = Arc::new(
        postgres::init_pg_pool(&app_config.database.url, app_config.database.max_connections)
            .await
            .expect("Failed to initialize PostgreSQL connection pool")
    );
    
    // 初始化RedisPool
    let redis_cfg = deadpool_redis::Config::from_url(app_config.redis.url);
    let redis_pool = Arc::new(
        redis_cfg.create_pool(Some(deadpool_redis::Runtime::Tokio1))
            .expect("Failed to initialize Redis connection pool")
    );

    // 路由
    let router = router::create_router();

    // 注入数据库连接池到Depot
    let pg_pool_clone = pg_pool.clone();
    let redis_pool_clone = redis_pool.clone();
    let handler = move |req: &mut Request, depot: &mut Depot| {
        depot.insert(pg_pool_clone.clone());
        depot.insert(redis_pool_clone.clone());
    };

    let acceptor = TcpListener::new(format!("{}:{}", app_config.server.host, app_config.server.port))
        .bind()
        .await;
        
    Server::new(acceptor)
        .hoop(handler)
        .serve(router)
        .await;
}