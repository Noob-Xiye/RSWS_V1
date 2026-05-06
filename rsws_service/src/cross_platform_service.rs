//! 跨平台服务

use chrono::{DateTime, Utc};
use rsws_common::error::RswsError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;
use tracing::{info, warn, error};

/// 平台配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformConfig {
    pub platform_name: String,
    pub api_endpoint: String,
    pub api_key: Option<String>,
    pub api_secret: Option<String>,
    pub webhook_url: Option<String>,
    pub is_active: bool,
}

/// 同步记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncRecord {
    pub id: i64,
    pub platform_name: String,
    pub operation_type: String,
    pub local_id: String,
    pub remote_id: Option<String>,
    pub sync_status: String,
    pub sync_data: Value,
    pub error_message: Option<String>,
    pub last_sync_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// 跨平台服务
pub struct CrossPlatformService {
    configs: HashMap<String, PlatformConfig>,
    client: reqwest::Client,
}

impl CrossPlatformService {
    /// 创建跨平台服务实例
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_default();
        Self {
            configs: HashMap::new(),
            client,
        }
    }

    /// 注册平台
    pub fn register_platform(&mut self, config: PlatformConfig) {
        info!("Registering platform: {}", config.platform_name);
        self.configs.insert(config.platform_name.clone(), config);
    }

    /// 获取平台配置
    pub fn get_platform_config(&self, platform_name: &str) -> Option<&PlatformConfig> {
        self.configs.get(platform_name)
    }

    /// 同步数据到远程平台
    ///
    /// 向目标平台的 API 发送 POST 请求，附带签名认证。
    /// 重试策略：最多 3 次，指数退避（1s/2s/4s）。
    pub async fn sync_data(
        &self,
        platform_name: &str,
        operation_type: &str,
        local_id: &str,
        data: Value,
    ) -> Result<SyncRecord, RswsError> {
        let config = self.configs.get(platform_name)
            .ok_or_else(|| RswsError::bad_request(format!("Platform not found: {}", platform_name)))?;

        if !config.is_active {
            return Err(RswsError::bad_request("Platform is not active"));
        }

        info!(
            "Syncing data to platform: {}, operation: {}, local_id: {}",
            platform_name, operation_type, local_id
        );

        // 构造同步 URL: {api_endpoint}/{operation_type}
        let url = format!("{}/{}", config.api_endpoint.trim_end_matches('/'), operation_type);

        // 构造请求体
        let mut body = serde_json::json!({
            "local_id": local_id,
            "operation": operation_type,
            "data": data,
            "timestamp": Utc::now().to_rfc3339(),
        });

        // 签名：HMAC-SHA256(api_secret, body_json) — 如果配置了 api_secret
        let mut request = self.client.post(&url)
            .header("Content-Type", "application/json");

        if let Some(ref api_key) = config.api_key {
            request = request.header("X-Api-Key", api_key);
        }

        if let Some(ref api_secret) = config.api_secret {
            let body_str = serde_json::to_string(&body)
                .map_err(|e| RswsError::internal(format!("Failed to serialize sync body: {}", e)))?;
            let signature = hmac_sha256(api_secret.as_bytes(), body_str.as_bytes());
            request = request.header("X-Signature", signature.clone());
            body["signature"] = Value::String(signature);
        }

        // 带重试的请求
        let mut last_error = None;
        for attempt in 0..3 {
            if attempt > 0 {
                let delay = Duration::from_secs(1 << attempt); // 2s, 4s
                info!("Retry attempt {} for {} sync, waiting {:?}", attempt + 1, platform_name, delay);
                tokio::time::sleep(delay).await;
            }

            match request
                .try_clone()
                .ok_or_else(|| RswsError::internal("Failed to clone request"))?
                .json(&body)
                .send()
                .await
            {
                Ok(resp) => {
                    let status = resp.status();
                    if status.is_success() {
                        let resp_data: Value = resp.json().await
                            .unwrap_or_else(|_| serde_json::json!({"status": "ok"}));
                        let remote_id = resp_data.get("id")
                            .or_else(|| resp_data.get("remote_id"))
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());

                        info!(
                            "Sync succeeded: platform={}, local_id={}, remote_id={:?}",
                            platform_name, local_id, remote_id
                        );

                        return Ok(SyncRecord {
                            id: rsws_common::snowflake::next_id(),
                            platform_name: platform_name.to_string(),
                            operation_type: operation_type.to_string(),
                            local_id: local_id.to_string(),
                            remote_id,
                            sync_status: "success".to_string(),
                            sync_data: data,
                            error_message: None,
                            last_sync_at: Some(Utc::now()),
                            created_at: Utc::now(),
                        });
                    } else {
                        let error_body = resp.text().await.unwrap_or_default();
                        let err_msg = format!("HTTP {} from {}: {}", status, url, error_body);
                        warn!("Sync failed (attempt {}): {}", attempt + 1, err_msg);
                        last_error = Some(err_msg);

                        // 4xx 不重试
                        if status.as_u16() >= 400 && status.as_u16() < 500 {
                            break;
                        }
                    }
                }
                Err(e) => {
                    let err_msg = format!("Request failed: {}", e);
                    warn!("Sync request error (attempt {}): {}", attempt + 1, err_msg);
                    last_error = Some(err_msg);
                }
            }
        }

        let error_message = last_error.unwrap_or_else(|| "Unknown sync error".to_string());
        error!("Sync ultimately failed: platform={}, local_id={}, error={}", platform_name, local_id, error_message);

        Ok(SyncRecord {
            id: rsws_common::snowflake::next_id(),
            platform_name: platform_name.to_string(),
            operation_type: operation_type.to_string(),
            local_id: local_id.to_string(),
            remote_id: None,
            sync_status: "failed".to_string(),
            sync_data: data,
            error_message: Some(error_message),
            last_sync_at: Some(Utc::now()),
            created_at: Utc::now(),
        })
    }
}

impl Default for CrossPlatformService {
    fn default() -> Self {
        Self::new()
    }
}

/// HMAC-SHA256 签名（hex 编码）
fn hmac_sha256(key: &[u8], message: &[u8]) -> String {
    use hmac::Mac;
    use std::fmt::Write;
    let mut hmac = hmac::Hmac::<sha2::Sha256>::new_from_slice(key)
        .expect("HMAC can take key of any size");
    hmac.update(message);
    let result = hmac.finalize();
    let code_bytes = result.into_bytes();
    code_bytes.iter().fold(String::with_capacity(64), |mut s: String, b: &u8| {
        write!(s, "{:02x}", b).unwrap();
        s
    })
}