//! 加密解密服务
//!
//! 基于 AES-256-GCM 的加密解密服务

use crate::error::RswsError;
use crate::error_code::ErrorCode;
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose, Engine as _};
use rand::RngCore;

/// 加密服务
pub struct EncryptionService {
    cipher: Aes256Gcm,
}

impl EncryptionService {
    /// 从 32 字节密钥创建加密服务
    pub fn new(key: &[u8; 32]) -> Self {
        let cipher = Aes256Gcm::new_from_slice(key).expect("Invalid key length");
        Self { cipher }
    }

    /// 从 Base64 编码的密钥创建加密服务
    pub fn from_base64(key_b64: &str) -> Result<Self, RswsError> {
        let key_bytes = general_purpose::STANDARD
            .decode(key_b64)
            .map_err(|e| RswsError::business_with_message(
                ErrorCode::CONFIG_DECRYPTION_FAILED,
                format!("Invalid key base64: {}", e)
            ))?;

        let key: [u8; 32] = key_bytes
            .try_into()
            .map_err(|_| RswsError::business(
                ErrorCode::CONFIG_INVALID_VALUE
            ))?;

        Ok(Self::new(&key))
    }

    /// 加密字符串
    pub fn encrypt(&self, plaintext: &str) -> Result<String, RswsError> {
        let mut nonce_bytes = [0u8; 12];
        rand::rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = self
            .cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| RswsError::business_with_message(
                ErrorCode::CONFIG_ENCRYPTION_FAILED,
                format!("Encryption failed: {}", e)
            ))?;

        // 格式: nonce (12 bytes) + ciphertext
        let mut result = nonce_bytes.to_vec();
        result.extend_from_slice(&ciphertext);

        Ok(general_purpose::STANDARD.encode(result))
    }

    /// 解密字符串
    pub fn decrypt(&self, encrypted_data: &str) -> Result<String, RswsError> {
        let data = general_purpose::STANDARD
            .decode(encrypted_data)
            .map_err(|e| RswsError::business_with_message(
                ErrorCode::CONFIG_DECRYPTION_FAILED,
                format!("Base64 decode failed: {}", e)
            ))?;

        if data.len() < 12 {
            return Err(RswsError::business(
                ErrorCode::CONFIG_DECRYPTION_FAILED
            ));
        }

        let (nonce_bytes, ciphertext) = data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        let plaintext = self
            .cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| RswsError::business_with_message(
                ErrorCode::CONFIG_DECRYPTION_FAILED,
                format!("Decryption failed: {}", e)
            ))?;

        String::from_utf8(plaintext)
            .map_err(|e| RswsError::internal(format!("UTF-8 decode failed: {}", e)))
    }
}

// ==================== 单元测试 ====================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_decryption() {
        let key = [0u8; 32]; // 测试用全零密钥
        let service = EncryptionService::new(&key);

        let plaintext = "Hello, World!";
        let encrypted = service.encrypt(plaintext).expect("Encryption failed");
        let decrypted = service.decrypt(&encrypted).expect("Decryption failed");

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_encrypt_different_nonce() {
        let key = [0u8; 32];
        let service = EncryptionService::new(&key);

        let plaintext = "Same text";
        let encrypted1 = service.encrypt(plaintext).expect("Encryption failed");
        let encrypted2 = service.encrypt(plaintext).expect("Encryption failed");

        // 每次加密应该产生不同的结果（因为 nonce 不同）
        assert_ne!(encrypted1, encrypted2);

        // 但都能正确解密
        let decrypted1 = service.decrypt(&encrypted1).expect("Decryption failed");
        let decrypted2 = service.decrypt(&encrypted2).expect("Decryption failed");
        assert_eq!(decrypted1, decrypted2);
    }

    #[test]
    fn test_decrypt_invalid_data() {
        let key = [0u8; 32];
        let service = EncryptionService::new(&key);

        let result = service.decrypt("invalid_base64!!!");
        assert!(result.is_err());
    }
}
