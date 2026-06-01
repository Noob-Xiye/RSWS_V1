//! 静态配置
//!
//! 只保留启动时必需的配置（server/database/redis/encryption）。
//! 所有业务配置（PayPal/区块链/Email/USDT监听等）从数据库读取，
//! 通过 ConfigService 提供。
//!
//! 优先级：环境变量 > config.toml
//! 环境变量前缀：RSWS_，分隔符：_
//! 例如：RSWS_DATABASE_URL 覆盖 database.url

use config::{Config, ConfigError, Environment};
use serde::Deserialize;

/// TLS 配置（可选，启用后支持 HTTPS + HTTP/3）
#[derive(Debug, Deserialize, Clone, Default)]
pub struct TlsConfig {
    /// 是否启用 TLS
    #[serde(default)]
    pub enabled: bool,
    /// TLS 证书路径（PEM 格式）
    #[serde(default)]
    pub cert_path: String,
    /// TLS 私钥路径（PEM 格式）
    #[serde(default)]
    pub key_path: String,
    /// 是否启用 HTTP/3（需要 quinn）
    #[serde(default)]
    pub http3: bool,
    /// HTTP/3 监听端口（默认与 server.port 相同）
    #[serde(default)]
    pub http3_port: Option<u16>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub cors_origins: Vec<String>,
    /// 信任代理 IP 列表（用于获取真实客户端 IP）
    /// 生产环境应配置 Nginx/Cloudflare 等 CDN 的 IP
    /// 开发环境可配置为空或 ["127.0.0.1"]
    #[serde(default)]
    pub trusted_proxies: Vec<String>,
    /// TLS 配置（可选）
    #[serde(default)]
    pub tls: TlsConfig,
    /// 文件上传目录（默认 uploads）
    #[serde(default = "default_upload_dir")]
    pub upload_dir: String,
}

fn default_upload_dir() -> String {
    "uploads".to_string()
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RedisConfig {
    pub url: String,
    pub pool_size: usize,
}

#[derive(Debug, Deserialize, Clone)]
pub struct EncryptionConfig {
    pub key: String,
}

/// 应用静态配置（从 config.toml 中的连接配置）
#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub encryption: EncryptionConfig,
}

pub fn load_config() -> Result<AppConfig, ConfigError> {
    let config = Config::builder()
        // 1. 先设置默认值（优先级最低）
        .add_source(config::File::from_str(
            r#"
[server]
host = "0.0.0.0"
port = 5170
cors_origins = ["*"]
trusted_proxies = []

[server.tls]
enabled = false
http3 = false
upload_dir = "uploads"

[database]
url = ""
max_connections = 10
min_connections = 1

[redis]
url = ""
pool_size = 10

[encryption]
key = "change-this-32-byte-encryption-k"
"#,
            config::FileFormat::Toml,
        ))
        // 2. 从 config.toml 读取基础配置（覆盖默认值）
        .add_source(config::File::with_name("config.toml").required(false))
        // 3. 环境变量覆盖（最高优先级，覆盖 config.toml 和默认值）
        .add_source(
            Environment::with_prefix("RSWS")
                .prefix_separator("_")
                .separator("_"),
        )
        .build()?;

    config.try_deserialize::<AppConfig>()
}
