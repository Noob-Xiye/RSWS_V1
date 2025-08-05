use crate::error::CommonError;
use argon2::password_hash::{rand_core::OsRng, SaltString};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};

pub struct PasswordService;

impl PasswordService {
    /// 使用Argon2加密密码
    pub fn hash_password(password: &str) -> Result<String, CommonError> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| CommonError::HashError(format!("Failed to hash password: {}", e)))?;

        Ok(password_hash.to_string())
    }

    /// 验证密码
    pub fn verify_password(password: &str, hash: &str) -> Result<bool, CommonError> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| CommonError::HashError(format!("Invalid password hash: {}", e)))?;

        let argon2 = Argon2::default();

        match argon2.verify_password(password.as_bytes(), &parsed_hash) {
            Ok(()) => Ok(true),
            Err(argon2::password_hash::Error::Password) => Ok(false),
            Err(e) => Err(CommonError::HashError(format!(
                "Password verification failed: {}",
                e
            ))),
        }
    }
}
