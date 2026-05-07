//! 签名服务
//!
//! 基于 HMAC-SHA256 的请求签名服务

use crate::error::RswsError;
use base64::{engine::general_purpose, Engine as _};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::time::{SystemTime, UNIX_EPOCH};

type HmacSha256 = Hmac<Sha256>;

/// 签名服务
pub struct SignatureService;

impl SignatureService {
    /// 生成签名
    ///
    /// 签名消息格式: `METHOD\nPATH\nTIMESTAMP\nNONCE\nBODY`
    pub fn generate(
        secret: &str,
        method: &str,
        path: &str,
        timestamp: u64,
        nonce: &str,
        body: &str,
    ) -> Result<String, RswsError> {
        let message = format!(
            "{}\n{}\n{}\n{}\n{}",
            method.to_uppercase(),
            path,
            timestamp,
            nonce,
            body
        );

        let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
            .map_err(|e| RswsError::internal(format!("HMAC key error: {}", e)))?;

        mac.update(message.as_bytes());
        let result = mac.finalize();

        Ok(general_purpose::STANDARD.encode(result.into_bytes()))
    }

    /// 验证签名
    pub fn verify(
        secret: &str,
        method: &str,
        path: &str,
        timestamp: u64,
        nonce: &str,
        body: &str,
        signature: &str,
    ) -> Result<bool, RswsError> {
        let expected = Self::generate(secret, method, path, timestamp, nonce, body)?;
        Ok(expected == signature)
    }

    /// 检查时间戳是否在有效范围内
    pub fn is_timestamp_valid(timestamp: u64, tolerance_seconds: u64) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let diff = now.abs_diff(timestamp);

        diff <= tolerance_seconds
    }
}

/// 客户端签名助手
pub struct ClientSignature {
    api_key: String,
    secret: String,
}

impl ClientSignature {
    /// 创建客户端签名实例
    pub fn new(api_key: String, secret: String) -> Self {
        Self { api_key, secret }
    }

    /// 获取 API Key
    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    /// 对请求进行签名
    ///
    /// 返回: (signature, timestamp, nonce)
    pub fn sign(
        &self,
        method: &str,
        path: &str,
        body: &str,
    ) -> Result<(String, u64, String), RswsError> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let nonce = uuid::Uuid::new_v4().to_string();

        let signature = SignatureService::generate(
            &self.secret,
            method,
            path,
            timestamp,
            &nonce,
            body,
        )?;

        Ok((signature, timestamp, nonce))
    }
}

// ==================== 单元测试 ====================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signature_generate_and_verify() {
        let secret = "test-secret-key";
        let method = "POST";
        let path = "/api/users";
        let timestamp = 1714848000;
        let nonce = "test-nonce-123";
        let body = r#"{"name":"test"}"#;

        let signature = SignatureService::generate(
            secret, method, path, timestamp, nonce, body
        ).expect("Generate failed");

        let valid = SignatureService::verify(
            secret, method, path, timestamp, nonce, body, &signature
        ).expect("Verify failed");

        assert!(valid);
    }

    #[test]
    fn test_signature_invalid_secret() {
        let secret = "correct-secret";
        let wrong_secret = "wrong-secret";
        let method = "GET";
        let path = "/api/test";
        let timestamp = 1714848000;
        let nonce = "nonce";
        let body = "";

        let signature = SignatureService::generate(
            secret, method, path, timestamp, nonce, body
        ).expect("Generate failed");

        let valid = SignatureService::verify(
            wrong_secret, method, path, timestamp, nonce, body, &signature
        ).expect("Verify failed");

        assert!(!valid);
    }

    #[test]
    fn test_timestamp_valid() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // 当前时间应该有效
        assert!(SignatureService::is_timestamp_valid(now, 300));

        // 1 分钟前应该有效
        assert!(SignatureService::is_timestamp_valid(now - 60, 300));

        // 10 分钟前应该无效（容差 5 分钟）
        assert!(!SignatureService::is_timestamp_valid(now - 600, 300));
    }

    #[test]
    fn test_client_signature() {
        let client = ClientSignature::new(
            "ak_test123".to_string(),
            "secret123".to_string()
        );

        assert_eq!(client.api_key(), "ak_test123");

        let (signature, timestamp, nonce) = client
            .sign("POST", "/api/orders", r#"{"amount":100}"#)
            .expect("Sign failed");

        assert!(!signature.is_empty());
        assert!(timestamp > 0);
        assert!(!nonce.is_empty());
    }
}
