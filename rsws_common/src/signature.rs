use crate::error::CommonError;
use base64::{engine::general_purpose, Engine as _};
use chrono::Utc;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

pub struct SignatureService;

impl SignatureService {
    pub fn generate_signature(
        secret: &str,
        method: &str,
        path: &str,
        timestamp: u64,
        nonce: &str,
        body: &str,
    ) -> Result<String, CommonError> {
        let message = format!(
            "{}\n{}\n{}\n{}\n{}",
            method.to_uppercase(),
            path,
            timestamp,
            nonce,
            body
        );

        let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
            .map_err(|e| CommonError::InternalServerError(format!("HMAC key error: {}", e)))?;

        mac.update(message.as_bytes());
        let result = mac.finalize();

        Ok(general_purpose::STANDARD.encode(result.into_bytes()))
    }

    pub fn verify_signature(
        secret: &str,
        method: &str,
        path: &str,
        timestamp: u64,
        nonce: &str,
        body: &str,
        signature: &str,
    ) -> Result<bool, CommonError> {
        let expected_signature =
            Self::generate_signature(secret, method, path, timestamp, nonce, body)?;

        Ok(expected_signature == signature)
    }

    pub fn is_timestamp_valid(timestamp: u64, tolerance_seconds: u64) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let diff = if now > timestamp {
            now - timestamp
        } else {
            timestamp - now
        };

        diff <= tolerance_seconds
    }
}

pub struct ClientSignatureHelper {
    _api_key: String, // 添加下划线前缀避免未使用警告
    secret: String,
}

impl ClientSignatureHelper {
    pub fn new(api_key: String, secret: String) -> Self {
        Self {
            _api_key: api_key,
            secret,
        }
    }

    pub fn sign_request(
        &self,
        method: &str,
        path: &str,
        body: &str,
    ) -> Result<(String, u64, String), CommonError> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let nonce = uuid::Uuid::new_v4().to_string();

        let signature = SignatureService::generate_signature(
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
