use base64::{engine::general_purpose, Engine as _};
use chrono::Utc;
use hmac::{Hmac, Mac};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

type HmacSha256 = Hmac<Sha256>;

pub struct SignatureService;

impl SignatureService {
    // 生成签名（客户端使用）
    pub fn generate_signature(
        api_secret: &str,
        method: &str,
        path: &str,
        timestamp: i64,
        nonce: &str,
        body: Option<&str>,
        query_params: Option<&BTreeMap<String, String>>,
    ) -> Result<String, String> {
        let string_to_sign =
            Self::build_string_to_sign(method, path, timestamp, nonce, body, query_params)?;

        let mut mac =
            HmacSha256::new_from_slice(api_secret.as_bytes()).map_err(|_| "Invalid secret key")?;
        mac.update(string_to_sign.as_bytes());

        let signature = mac.finalize().into_bytes();
        Ok(general_purpose::STANDARD.encode(signature))
    }

    // 验证签名（服务端使用）
    pub fn verify_signature(
        api_secret: &str,
        method: &str,
        path: &str,
        timestamp: i64,
        nonce: &str,
        signature: &str,
        body: Option<&str>,
        query_params: Option<&BTreeMap<String, String>>,
    ) -> Result<bool, String> {
        // 检查时间戳（防重放攻击）
        let current_timestamp = Utc::now().timestamp();
        let time_diff = (current_timestamp - timestamp).abs();
        if time_diff > 300 {
            // 5分钟有效期
            return Ok(false);
        }

        let expected_signature = Self::generate_signature(
            api_secret,
            method,
            path,
            timestamp,
            nonce,
            body,
            query_params,
        )?;

        Ok(signature == expected_signature)
    }

    // 构建待签名字符串
    fn build_string_to_sign(
        method: &str,
        path: &str,
        timestamp: i64,
        nonce: &str,
        body: Option<&str>,
        query_params: Option<&BTreeMap<String, String>>,
    ) -> Result<String, String> {
        let mut parts = vec![
            method.to_uppercase(),
            path.to_string(),
            timestamp.to_string(),
            nonce.to_string(),
        ];

        // 添加查询参数（按字母顺序排序）
        if let Some(params) = query_params {
            let query_string = params
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join("&");
            if !query_string.is_empty() {
                parts.push(query_string);
            }
        }

        // 添加请求体（如果有）
        if let Some(body_str) = body {
            if !body_str.is_empty() {
                // 对JSON进行标准化处理
                let normalized_body = Self::normalize_json(body_str)?;
                parts.push(normalized_body);
            }
        }

        Ok(parts.join("\n"))
    }

    // JSON标准化（确保签名一致性）
    fn normalize_json(json_str: &str) -> Result<String, String> {
        if json_str.trim().is_empty() {
            return Ok(String::new());
        }

        let value: Value = serde_json::from_str(json_str).map_err(|_| "Invalid JSON format")?;

        // 使用紧凑格式，确保键按字母顺序排列
        serde_json::to_string(&value).map_err(|_| "Failed to serialize JSON")
    }

    // 生成随机nonce
    pub fn generate_nonce() -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let nonce_bytes: [u8; 16] = rng.gen();
        general_purpose::URL_SAFE_NO_PAD.encode(nonce_bytes)
    }
}

// 客户端签名助手
pub struct ClientSignatureHelper {
    api_key: String,
    api_secret: String,
}

impl ClientSignatureHelper {
    pub fn new(api_key: String, api_secret: String) -> Self {
        Self {
            api_key,
            api_secret,
        }
    }

    pub fn sign_request(
        &self,
        method: &str,
        path: &str,
        body: Option<&str>,
        query_params: Option<&BTreeMap<String, String>>,
    ) -> Result<(i64, String, String), String> {
        let timestamp = Utc::now().timestamp();
        let nonce = SignatureService::generate_nonce();

        let signature = SignatureService::generate_signature(
            &self.api_secret,
            method,
            path,
            timestamp,
            &nonce,
            body,
            query_params,
        )?;

        Ok((timestamp, nonce, signature))
    }
}
