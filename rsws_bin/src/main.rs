//! RSWS 主程序
//!
//! 启动流程:
//! 1. 从 config.toml 加载静态配置（server/database/redis/encryption）
//! 2. 初始化 DB 和 Redis 连接
//! 3. 从 DB 读取动态配置（PayPal/区块链/Email/USDT监听等）
//! 4. 构建服务并启动

use rsws_api::router;
use rsws_api::state::AppState;
use rsws_common::config::load_config;
use rsws_common::error::RswsError;
use rsws_db::RedisPool;
use salvo::conn::rustls::{Keycert, RustlsConfig};
use salvo::prelude::*;
use salvo::Server;
use sqlx::postgres::PgPoolOptions;
use std::convert::TryInto;
use std::sync::Arc;
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;

/// 初始化结构化日志
///
/// 支持环境变量控制:
/// - `RUST_LOG`: 日志级别过滤 (e.g., "info,rsws=debug")
/// - `LOG_FORMAT`: 输出格式，"json" (默认) 或 "pretty" (开发)
fn init_logging() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let format = std::env::var("LOG_FORMAT").unwrap_or_else(|_| "json".to_string());

    if format == "pretty" {
        // 开发环境：彩色可读格式
        tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .with_target(true)
            .with_thread_ids(true)
            .with_line_number(true)
            .pretty()
            .init();
    } else {
        // 生产环境：JSON 结构化格式
        tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .with_target(true)
            .with_thread_ids(true)
            .with_line_number(true)
            .json()
            .init();
    }
}

#[tokio::main]
async fn main() -> Result<(), RswsError> {
    // 初始化结构化日志
    init_logging();

    // 尝试从 .env 文件加载环境变量（开发环境使用，生产环境忽略）
    dotenvy::dotenv().ok();

    info!("Starting RSWS server...");

    // ========== 1. 加载静态配置（仅 server/database/redis/encryption） ==========
    let config = load_config().map_err(|e| {
        error!("Failed to load config: {}", e);
        RswsError::internal("Failed to load config")
    })?;

    info!(
        "Config loaded: {}:{}",
        config.server.host, config.server.port
    );

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

    // 运行数据库迁移
    // TODO: 迁移文件已清空，等业务代码完善后重新生成准确的迁移文件
    // #[cfg(not(debug_assertions))]
    // sqlx::migrate!("../migrations")
    //     .run(&pool)
    //     .await
    //     .map_err(|e| {
    //         error!("Database migration failed: {}", e);
    //         RswsError::internal("Database migration failed")
    //     })?;
    // #[cfg(debug_assertions)]
    // {
    //     warn!("Skipping migration checksum check in debug mode");
    //     let _ = sqlx::migrate!("../migrations").run(&pool).await;
    // }
    info!("Database migrations skipped (pending schema redesign)");

    let redis_pool = RedisPool::new(&config.redis.url)?;
    info!("Redis connected");

    // ========== 3. 初始化 ConfigService 并从 DB 读取动态配置 ==========
    let config_service = rsws_service::create_config_service(pool.clone(), redis_pool.clone());
    let config_service = std::sync::Arc::new(config_service);

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
    let blockchain_configs = config_service
        .get_blockchain_configs()
        .await
        .map_err(|e| {
            warn!("Failed to load blockchain configs from DB: {}", e);
            e
        })
        .unwrap_or_default();
    info!(
        "Blockchain configs loaded: {} networks",
        blockchain_configs.len()
    );

    // 读取 USDT 监听配置
    let usdt_listen_configs = config_service
        .get_usdt_listen_configs()
        .await
        .map_err(|e| {
            warn!("Failed to load USDT listen configs from DB: {}", e);
            e
        })
        .unwrap_or_default();
    info!(
        "USDT listen configs loaded: {} networks",
        usdt_listen_configs.len()
    );

    // 读取 Email 配置
    let email_db_config = config_service
        .get_email_config()
        .await
        .map_err(|e| warn!("Failed to load email config from DB: {}", e))
        .ok()
        .flatten(); // Result<Option<EmailDbConfig>> → Option<EmailDbConfig>
    if email_db_config.is_some() {
        info!("Email config loaded from database");
    } else {
        warn!("No active email config found in database — email will be disabled");
    }

    // ========== 4. 创建所有 service ==========
    // EmailVerificationService 根据 email_configs.provider 自动切换 dev/prod 模式
    let user_service = rsws_service::create_user_service(
        pool.clone(),
        Some(redis_pool.clone()),
        email_db_config.as_ref(),
    );
    let order_service = rsws_service::create_order_service(pool.clone());
    let resource_service =
        rsws_service::create_resource_service(pool.clone(), Some(config_service.as_ref().clone()));
    let admin_api_key_manager = rsws_service::create_admin_api_key_manager(redis_pool.clone());
    let user_api_key_manager = rsws_service::create_user_api_key_manager(redis_pool.clone());
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
    let category_repo = rsws_db::CategoryRepository::new(pool.clone());
    let admin_service = rsws_service::create_admin_service(pool.clone());
    let log_service = rsws_service::LogService::new(pool.clone());
    let login_log_service = rsws_service::LoginLogService::new(pool.clone());
    let error_log_service = rsws_service::ErrorLogService::new(pool.clone());
    let audit_log_service = rsws_service::AuditLogService::new(pool.clone());

    info!("Services initialized");

    // 创建 AppState
    let app_state = AppState::new(
        pool.clone(),
        config.clone(),
        user_service,
        order_service,
        resource_service,
        admin_api_key_manager,
        user_api_key_manager,
        paypal_service,
        payment_service,
        blockchain_service,
        webhook_service,
        cross_platform_service,
        config_service.clone(),
        admin_service,
        log_service,
        login_log_service,
        error_log_service,
        audit_log_service,
        admin_repo,
        category_repo,
    );

    // ========== 5. 启动 USDT 监听服务（配置来自数据库） ==========
    let tron_listener_config = usdt_listen_configs
        .iter()
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

    let eth_listener_config = usdt_listen_configs
        .iter()
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
        let listener =
            rsws_usdt::UsdtListener::new(pool.clone(), tron_listener_config, eth_listener_config);
        listener.start().await;
        info!("USDT listener started (configs from database)");
    } else {
        warn!("USDT listener disabled (no active listen configs in database)");
    }

    // ========== 6. 启动 HTTP/HTTPS/HTTP3 服务 ==========
    let router = router::create_router(app_state);

    let addr = format!("{}:{}", config.server.host, config.server.port);

    if config.server.tls.enabled {
        let tls_config = &config.server.tls;

        // 验证证书文件存在
        if !std::path::Path::new(&tls_config.cert_path).exists() {
            error!("TLS cert file not found: {}", tls_config.cert_path);
            return Err(RswsError::internal("TLS cert file not found"));
        }
        if !std::path::Path::new(&tls_config.key_path).exists() {
            error!("TLS key file not found: {}", tls_config.key_path);
            return Err(RswsError::internal("TLS key file not found"));
        }

        // 加载 rustls 证书
        let rustls_config = RustlsConfig::new(
            Keycert::new()
                .key_from_path(&tls_config.key_path)
                .map_err(|e| {
                    error!("Failed to read TLS key: {}", e);
                    RswsError::internal("Failed to read TLS key file")
                })?
                .cert_from_path(&tls_config.cert_path)
                .map_err(|e| {
                    error!("Failed to read TLS cert: {}", e);
                    RswsError::internal("Failed to read TLS cert file")
                })?,
        );

        // HTTP/3 (QuinnListener) - 需要 quinn feature
        if tls_config.http3 {
            use salvo::conn::quinn::QuinnListener;

            let http3_port = tls_config.http3_port.unwrap_or(config.server.port);
            let http3_addr = format!("{}:{}", config.server.host, http3_port);

            // RustlsConfig 可以转换为 quinn::ServerConfig
            let quinn_server_config: salvo::conn::quinn::ServerConfig =
                rustls_config.clone().try_into().map_err(|e| {
                    error!("Failed to create Quinn server config: {:?}", e);
                    RswsError::internal("Failed to create HTTP/3 server config")
                })?;

            // 创建 QuinnListener (HTTP/3) - 注意：需要在 bind 之前 join
            let quinn_listener = QuinnListener::new(quinn_server_config, http3_addr.clone());

            // 先创建 HTTPS listener (未 bind)
            let https_listener = TcpListener::new(addr.clone()).rustls(rustls_config.clone());

            // 组合两个 listener，然后一起 bind
            let joined = https_listener.join(quinn_listener);
            let acceptor = joined.try_bind().await.map_err(|e| {
                error!("Failed to bind HTTPS/HTTP3 listeners: {:?}", e);
                RswsError::internal("Failed to bind HTTPS/HTTP3 listeners")
            })?;

            info!("HTTPS server listening on https://{}", addr);
            info!("HTTP/3 server listening on https://{} (QUIC)", http3_addr);

            Server::new(acceptor).serve(router).await;
        } else {
            // 仅 HTTPS
            let https_listener = TcpListener::new(addr.clone())
                .rustls(rustls_config)
                .bind()
                .await;
            info!("HTTPS server listening on https://{}", addr);
            Server::new(https_listener).serve(router).await;
        }
    } else {
        // 纯 HTTP 模式（开发环境）
        info!("Server listening on http://{} (TLS disabled)", addr);
        let acceptor = TcpListener::new(addr).bind().await;
        Server::new(acceptor).serve(router).await;
    }

    Ok(())
}
