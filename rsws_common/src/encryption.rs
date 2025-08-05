use crate::error::CommonError;
use aes_gcm::aead::{Aead, NewAead};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use base64::{engine::general_purpose, Engine as _};
use rand::Rng;

pub struct EncryptionService {
    cipher: Aes256Gcm,
}

impl EncryptionService {
    pub fn new(key: &[u8; 32]) -> Self {
        let key = Key::from_slice(key);
        let cipher = Aes256Gcm::new(key);
        Self { cipher }
    }

    pub fn encrypt(&self, plaintext: &str) -> Result<String, CommonError> {
        let mut rng = rand::thread_rng();
        let nonce_bytes: [u8; 12] = rng.gen();
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = self
            .cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|_| CommonError::EncryptionError("Failed to encrypt".to_string()))?;

        let mut result = nonce_bytes.to_vec();
        result.extend_from_slice(&ciphertext);

        Ok(general_purpose::STANDARD.encode(&result))
    }

    pub fn decrypt(&self, encrypted: &str) -> Result<String, CommonError> {
        let data = general_purpose::STANDARD
            .decode(encrypted)
            .map_err(|_| CommonError::EncryptionError("Invalid base64".to_string()))?;

        if data.len() < 12 {
            return Err(CommonError::EncryptionError(
                "Invalid encrypted data".to_string(),
            ));
        }

        let (nonce_bytes, ciphertext) = data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        let plaintext = self
            .cipher
            .decrypt(nonce, ciphertext)
            .map_err(|_| CommonError::EncryptionError("Failed to decrypt".to_string()))?;

        String::from_utf8(plaintext)
            .map_err(|_| CommonError::EncryptionError("Invalid UTF-8".to_string()))
    }
}
