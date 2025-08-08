use rustls::{Certificate, PrivateKey, ResolvesServerCertUsingSni, ServerConfig};
use rustls_pemfile::{certs, pkcs8_private_keys, rsa_private_keys};
use salvo::conn::rustls::{Keycert, RustlsConfig};
use salvo::prelude::*;
use std::io::Cursor;
use std::{
    sync::{Arc, RwLock},
    time::Duration,
};
use tokio::time;

use rsws_api::router;
use rsws_common::config::load_config;
use rsws_db::postgres;
use rsws_db::redis as redis_db;

type SharedResolver = Arc<RwLock<ResolvesServerCertUsingSni>>;

// 解析证书链
fn parse_certificates(pem: &str) -> anyhow::Result<Vec<Certificate>> {
    let mut reader = Cursor::new(pem);
    let certs = certs(&mut reader)?.into_iter().map(Certificate).collect();
    Ok(certs)
}

// 解析私钥（PKCS8 或 RSA）
fn parse_private_key(pem: &str) -> anyhow::Result<PrivateKey> {
    let mut reader = Cursor::new(pem);
    let mut keys = pkcs8_private_keys(&mut reader)?;
    if !keys.is_empty() {
        return Ok(PrivateKey(keys.remove(0)));
    }
    reader.set_position(0);
    let mut keys = rsa_private_keys(&mut reader)?;
    if !keys.is_empty() {
        return Ok(PrivateKey(keys.remove(0)));
    }
    anyhow::bail!("No valid private key found in PEM");
}

// 从数据库加载所有域名证书到resolver
async fn update_resolver_from_db(
    pg_pool: &deadpool_postgres::Pool,
    resolver: SharedResolver,
) -> anyhow::Result<()> {
    let client = pg_pool.get().await?;
    // 假设表 domains(domain TEXT, cert_pem TEXT, key_pem TEXT)
    let rows = client
        .query("SELECT domain, cert_pem, key_pem FROM domains", &[])
        .await?;

    let mut new_resolver = ResolvesServerCertUsingSni::new();

    for row in rows {
        let domain: String = row.get("domain");
        let cert_pem: String = row.get("cert_pem");
        let key_pem: String = row.get("key_pem");

        let cert_chain = parse_certificates(&cert_pem)?;
        let priv_key = parse_private_key(&key_pem)?;

        let signing_key = rustls::sign::any_supported_type(&priv_key)
            .map_err(|e| anyhow::anyhow!("Failed to parse key for {}: {:?}", domain, e))?;
        let certified_key = rustls::CertifiedKey::new(cert_chain, Arc::new(signing_key));

        new_resolver.add(&domain, certified_key)?;
    }

    let mut write_guard = resolver.write().unwrap();
    *write_guard = new_resolver;

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();

    // 加载配置
    let app_config = load_config().expect("Failed to load configuration");

    // 初始化PgPool
    let pg_pool = Arc::new(
        postgres::init_pg_pool(
            &app_config.database.url,
            app_config.database.max_connections,
        )
        .await
        .expect("Failed to initialize PostgreSQL connection pool"),
    );

    // 初始化RedisPool
    let redis_cfg = deadpool_redis::Config::from_url(app_config.redis.url);
    let redis_pool = Arc::new(
        redis_cfg
            .create_pool(Some(deadpool_redis::Runtime::Tokio1))
            .expect("Failed to initialize Redis connection pool"),
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

    // 初始化动态SNI解析器
    let resolver = Arc::new(RwLock::new(ResolvesServerCertUsingSni::new()));

    // 首次加载证书
    update_resolver_from_db(&pg_pool, Arc::clone(&resolver)).await?;

    // 定时任务刷新证书（每10分钟刷新一次）
    {
        let pg_pool = pg_pool.clone();
        let resolver = Arc::clone(&resolver);
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(600));
            loop {
                interval.tick().await;
                if let Err(e) = update_resolver_from_db(&pg_pool, Arc::clone(&resolver)).await {
                    tracing::error!("Failed to refresh TLS certs: {:?}", e);
                }
            }
        });
    }

    // 用动态resolver构建 ServerConfig
    let server_config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_cert_resolver(resolver);

    let rustls_config = RustlsConfig::from_rustls_config(Arc::new(server_config));

    // 监听地址
    let listen_addr = format!("{}:{}", app_config.server.host, app_config.server.port);

    // 创建带 TLS 的监听器
    let tcp_listener = TcpListener::new(listen_addr.clone()).rustls(rustls_config.clone());

    let quinn_listener =
        QuinnListener::new(rustls_config.build_quinn_config()?, listen_addr.clone());

    // 合并监听器
    let acceptor = quinn_listener.join(tcp_listener).bind().await?;

    // 启动服务器
    Server::new(acceptor).hoop(handler).serve(router).await;

    Ok(())
}
