//! RSWS 主程序
//!
//! 启动流程:
//! 1. 从 config.toml 加载静态配置（server/database/redis/encryption）
//! 2. 初始化 DB 和 Redis 连接
//! 3. 从 DB 读取动态配置（PayPal/区块链/Email/USDT监听等）
//! 4. 构建服务并启动

use salvo::prelude::*;
use std::sync::Arc;
use salvo::Server;
use rsws_api::router;
use rsws_api::state::AppState;
use rsws_common::config::load_config;
use rsws_common::error::RswsError;
use sqlx::postgres::PgPoolOptions;
use tracing::{info, error, warn};
use rsws_db::RedisPool;

#[tokio::main]
async fn main() -> Result<(), RswsError> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    info!("Starting RSWS server...");

    // ========== 1. 加载静态配置（仅 server/database/redis/encryption） ==========
    let config = load_config().map_err(|e| {
        error!("Failed to load config: {}", e);
        RswsError::internal("Failed to load config")
    })?;

    info!("Config loaded: {}:{}", config.server.host, config.server.port);

    // ========== 2. 初始化数据库和 Redis 连接 ==========
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

    let redis_pool = RedisPool::new(&config.redis.url)?;
    info!("Redis connected");

    // ========== 3. 初始化 ConfigService 并从 DB 读取动态配置 ==========
    let config_service = rsws_service::create_config_service(pool.clone(), redis_pool.clone());

    // 读取 PayPal 配置
    let paypal_db_config = match config_service.get_paypal_config().await {
        Ok(Some(c)) => {
            info!("PayPal config loaded from database");
            Some(c)
        }
        Ok(None) => {
            warn!("No active PayPal config found in database — PayPal will run in mock mode");
            None
        }
        Err(e) => {
            warn!("Failed to load PayPal config from DB: {}, using None", e);
            None
        }
    };

    // 读取区块链配置
    let blockchain_configs = config_service.get_blockchain_configs().await
        .map_err(|e| {
            warn!("Failed to load blockchain configs from DB: {}", e);
            e
        })
        .unwrap_or_default();
    info!("Blockchain configs loaded: {} networks", blockchain_configs.len());

    // 读取 USDT 监听配置
    let usdt_listen_configs = config_service.get_usdt_listen_configs().await
        .map_err(|e| {
            warn!("Failed to load USDT listen configs from DB: {}", e);
            e
        })
        .unwrap_or_default();
    info!("USDT listen configs loaded: {} networks", usdt_listen_configs.len());

    // 读取 Email 配置
    let email_db_config = config_service.get_email_config().await
        .map_err(|e| warn!("Failed to load email config from DB: {}", e))
        .ok()
        .flatten(); // Result<Option<EmailDbConfig>> → Option<EmailDbConfig>
    if email_db_config.is_some() {
        info!("Email config loaded from database");
    } else {
        warn!("No active email config found in database — email will be disabled");
    }

    // ========== 4. 创建所有 service ==========
    let user_service = match email_db_config {
        Some(ref email_cfg) => {
            let email_config = rsws_common::email::EmailConfig {
                smtp_server: email_cfg.host.clone(),
                smtp_username: email_cfg.username.clone(),
                smtp_password: email_cfg.password.clone(),
                from_email: email_cfg.from_email.clone(),
            };
            match rsws_common::email::EmailService::new(&email_config) {
                Ok(email_service) => {
                    info!("Email service initialized");
                    rsws_service::create_user_service_with_email(pool.clone(), Some(redis_pool.clone()), email_service)
                }
                Err(e) => {
                    warn!("Failed to initialize email service: {}, continuing without email", e);
                    rsws_service::create_user_service(pool.clone(), Some(redis_pool.clone()))
                }
            }
        }
        None => rsws_service::create_user_service(pool.clone(), Some(redis_pool.clone())),
    };
    let order_service = rsws_service::create_order_service(pool.clone());
    let resource_service = rsws_service::create_resource_service(pool.clone());
    let api_key_service = rsws_service::ApiKeyService::with_redis(
        std::sync::Arc::new(rsws_db::ApiKeyRepository::new(pool.clone())),
        redis_pool.clone(),
    );
    let wallet_repo = rsws_db::WalletRepository::new(pool.clone());

    // PayPal 服务 — 配置从 DB 读取
    let paypal_service = Arc::new(rsws_service::create_paypal_service(paypal_db_config));
    let payment_service = rsws_service::create_payment_service(pool.clone());

    // 区块链服务 — 不再依赖 config.toml
    let blockchain_service = rsws_service::create_blockchain_service(wallet_repo);
    let webhook_service = rsws_service::create_webhook_service(paypal_service.clone());
    let cross_platform_service = rsws_service::create_cross_platform_service();

    // Admin 服务
    let admin_repo = rsws_db::AdminRepository::new(pool.clone());
    let admin_service = rsws_service::create_admin_service(pool.clone(), Some(redis_pool.clone()));
    let log_service = rsws_service::LogService::new(pool.clone());

    info!("Services initialized");

    // 创建 AppState
    let app_state = AppState::new(
        user_service,
        order_service,
        resource_service,
        api_key_service,
        paypal_service,
        payment_service,
        blockchain_service,
        webhook_service,
        cross_platform_service,
        config_service,
        admin_service,
        log_service,
        admin_repo,
    );

    // ========== 5. 启动 USDT 监听服务（配置来自数据库） ==========
    let tron_listener_config = usdt_listen_configs.iter()
        .find(|c| c.network == "tron" && c.is_active)
        .map(|c| rsws_usdt::UsdtConfig {
            network: c.network.clone(),
            api_url: c.api_url.clone(),
            api_key: c.api_key.clone(),
            usdt_contract: c.usdt_contract.clone(),
            poll_interval_seconds: c.poll_interval_seconds,
            min_confirmations: c.min_confirmations,
            is_active: c.is_active,
        });

    let eth_listener_config = usdt_listen_configs.iter()
        .find(|c| c.network == "ethereum" && c.is_active)
        .map(|c| rsws_usdt::UsdtConfig {
            network: c.network.clone(),
            api_url: c.api_url.clone(),
            api_key: c.api_key.clone(),
            usdt_contract: c.usdt_contract.clone(),
            poll_interval_seconds: c.poll_interval_seconds,
            min_confirmations: c.min_confirmations,
            is_active: c.is_active,
        });

    if tron_listener_config.is_some() || eth_listener_config.is_some() {
        let listener = rsws_usdt::UsdtListener::new(
            pool.clone(),
            tron_listener_config,
            eth_listener_config,
        );
        listener.start().await;
        info!("USDT listener started (configs from database)");
    } else {
        warn!("USDT listener disabled (no active listen configs in database)");
    }

    // ========== 6. 启动 HTTP 服务 ==========
    let router = router::create_router(app_state);

    let addr = format!("{}:{}", config.server.host, config.server.port);
    info!("Server listening on http://{}", addr);

    let acceptor = TcpListener::new(&addr).bind().await;
    Server::new(acceptor).serve(router).await;

    Ok(())
}
