use crate::error::CommonError;
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose, Engine as _};
use rand::RngCore;

pub struct EncryptionService {
    cipher: Aes256Gcm,
}

impl EncryptionService {
    pub fn new(key: &[u8; 32]) -> Self {
        let cipher = Aes256Gcm::new_from_slice(key).expect("Invalid key length");
        Self { cipher }
    }

    pub fn encrypt(&self, plaintext: &str) -> Result<String, CommonError> {
        let mut nonce_bytes = [0u8; 12];
        rand::rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = self
            .cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| CommonError::InternalServerError(format!("Encryption failed: {}", e)))?;

        let mut result = nonce_bytes.to_vec();
        result.extend_from_slice(&ciphertext);

        Ok(general_purpose::STANDARD.encode(result))
    }

    pub fn decrypt(&self, encrypted_data: &str) -> Result<String, CommonError> {
        let data = general_purpose::STANDARD
            .decode(encrypted_data)
            .map_err(|e| {
                CommonError::InternalServerError(format!("Base64 decode failed: {}", e))
            })?;

        if data.len() < 12 {
            return Err(CommonError::InternalServerError(
                "Invalid encrypted data".to_string(),
            ));
        }

        let (nonce_bytes, ciphertext) = data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        let plaintext = self
            .cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| CommonError::InternalServerError(format!("Decryption failed: {}", e)))?;

        String::from_utf8(plaintext)
            .map_err(|e| CommonError::InternalServerError(format!("UTF-8 decode failed: {}", e)))
    }
}
