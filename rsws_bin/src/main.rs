//! RSWS 主程序

use salvo::prelude::*;
use salvo::Server;
use rsws_api::router;
use rsws_api::state::AppState;
use rsws_common::config::load_config;
use rsws_common::error::RswsError;
use sqlx::postgres::PgPoolOptions;
use tracing::{info, error};
use rsws_db::RedisPool;

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
    let pool = PgPoolOptions::new()
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
    let redis_pool = RedisPool::new(&config.redis.url)?;
    info!("Redis connected");

    // 创建所有 service
    let user_service = rsws_service::create_user_service(pool.clone(), Some(redis_pool));
    let order_service = rsws_service::create_order_service(pool.clone());
    let resource_service = rsws_service::create_resource_service(pool.clone());
    let api_key_service = rsws_service::create_api_key_service(pool.clone());
    let config_service = rsws_service::create_config_service(pool.clone());
    let wallet_repo = rsws_db::WalletRepository::new(pool.clone());

    // PayPal/区块链/webhook/跨平台服务
    let paypal_config = config.paypal();
    let usdt_config = config.usdt();
    let paypal_service = rsws_service::create_paypal_service(paypal_config);
    let blockchain_service = rsws_service::create_blockchain_service(usdt_config, wallet_repo);
    let webhook_service = rsws_service::create_webhook_service();
    let cross_platform_service = rsws_service::create_cross_platform_service();

    info!("Services initialized");

    // 创建 AppState
    let app_state = AppState::new(
        user_service,
        order_service,
        resource_service,
        api_key_service,
        paypal_service,
        blockchain_service,
        webhook_service,
        cross_platform_service,
        config_service,
    );

    // 启动 USDT 监听服务
    let usdt_cfg = config.usdt();
    let tron_listener_config = if !usdt_cfg.trc20_address.is_empty() {
        Some(rsws_usdt::UsdtConfig {
            network: "tron".to_string(),
            api_url: "https://api.trongrid.io".to_string(),
            api_key: usdt_cfg.trongrid_api_key.clone(),
            usdt_contract: "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t".to_string(),
            poll_interval_seconds: 10,
            min_confirmations: usdt_cfg.confirmations_required as i32,
            is_active: true,
        })
    } else {
        None
    };
    let eth_listener_config = if !usdt_cfg.erc20_address.is_empty() {
        Some(rsws_usdt::UsdtConfig {
            network: "ethereum".to_string(),
            api_url: "https://api.etherscan.io".to_string(),
            api_key: usdt_cfg.etherscan_api_key.clone(),
            usdt_contract: "0xdAC17F958D2ee523a2206206994597C13D831ec7".to_string(),
            poll_interval_seconds: 15,
            min_confirmations: 12,
            is_active: true,
        })
    } else {
        None
    };

    if tron_listener_config.is_some() || eth_listener_config.is_some() {
        let listener = rsws_usdt::UsdtListener::new(
            pool.clone(),
            tron_listener_config,
            eth_listener_config,
        );
        listener.start().await;
        info!("USDT listener started (Tron + Ethereum)");
    } else {
        info!("USDT listener disabled (no addresses configured)");
    }

    // 创建路由
    let router = router::create_router(app_state);

    // 启动服务器
    let addr = format!("{}:{}", config.server.host, config.server.port);
    info!("Server listening on http://{}", addr);

    // 绑定端口并启动服务
    let acceptor = TcpListener::new(&addr).bind().await;
    Server::new(acceptor).serve(router).await;

    Ok(())
}
