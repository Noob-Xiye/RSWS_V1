//! 密码服务
//!
//! 基于 Argon2 的密码哈希和验证服务

use crate::error::RswsError;
use crate::error_code::ErrorCode;
use argon2::password_hash::{rand_core::OsRng, SaltString};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};

/// 密码服务
pub struct PasswordService;

impl PasswordService {
    /// 使用 Argon2 哈希密码
    pub fn hash(password: &str) -> Result<String, RswsError> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| RswsError::internal(format!("Failed to hash password: {}", e)))?;

        Ok(password_hash.to_string())
    }

    /// 验证密码
    pub fn verify(password: &str, hash: &str) -> Result<bool, RswsError> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| RswsError::bad_request(format!("Invalid password hash: {}", e)))?;

        let argon2 = Argon2::default();

        match argon2.verify_password(password.as_bytes(), &parsed_hash) {
            Ok(()) => Ok(true),
            Err(argon2::password_hash::Error::Password) => Ok(false),
            Err(e) => Err(RswsError::internal(format!(
                "Password verification failed: {}",
                e
            ))),
        }
    }

    /// 验证密码强度
    /// 要求: 至少 8 个字符，包含大小写字母和数字
    pub fn validate_strength(password: &str) -> Result<(), RswsError> {
        if password.len() < 8 {
            return Err(RswsError::business_with_message(
                ErrorCode::AUTH_PASSWORD_TOO_WEAK,
                "Password must be at least 8 characters",
            ));
        }

        let has_upper = password.chars().any(|c| c.is_uppercase());
        let has_lower = password.chars().any(|c| c.is_lowercase());
        let has_digit = password.chars().any(|c| c.is_numeric());

        if !has_upper || !has_lower || !has_digit {
            return Err(RswsError::business_with_message(
                ErrorCode::AUTH_PASSWORD_TOO_WEAK,
                "Password must contain uppercase, lowercase and digit",
            ));
        }

        Ok(())
    }
}

// ==================== 单元测试 ====================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_and_verify() {
        let password = "TestPassword123";
        let hash = PasswordService::hash(password).expect("Hash failed");

        // 正确密码应该验证通过
        let result = PasswordService::verify(password, &hash).expect("Verify failed");
        assert!(result);

        // 错误密码应该验证失败
        let result = PasswordService::verify("WrongPassword", &hash).expect("Verify failed");
        assert!(!result);
    }

    #[test]
    fn test_hash_unique() {
        let password = "SamePassword123";

        let hash1 = PasswordService::hash(password).expect("Hash failed");
        let hash2 = PasswordService::hash(password).expect("Hash failed");

        // 相同密码应该产生不同的哈希（因为 salt 不同）
        assert_ne!(hash1, hash2);

        // 但都能验证通过
        assert!(PasswordService::verify(password, &hash1).expect("Verify failed"));
        assert!(PasswordService::verify(password, &hash2).expect("Verify failed"));
    }

    #[test]
    fn test_admin_password_hash() {
        let password = "Admin123!@#";
        let hash = PasswordService::hash(password).expect("Hash failed");
        println!("\nPassword: {}", password);
        println!("Argon2 Hash: {}", hash);
        
        // Verify it works
        let result = PasswordService::verify(password, &hash).expect("Verify failed");
        assert!(result);
    }

    #[test]
    fn test_validate_strength() {
        // 有效密码
        assert!(PasswordService::validate_strength("Password123").is_ok());
        assert!(PasswordService::validate_strength("MySecure1Pass").is_ok());

        // 无效密码
        assert!(PasswordService::validate_strength("short").is_err()); // 太短
        assert!(PasswordService::validate_strength("alllowercase1").is_err()); // 无大写
        assert!(PasswordService::validate_strength("ALLUPPERCASE1").is_err()); // 无小写
        assert!(PasswordService::validate_strength("NoNumbers").is_err()); // 无数字
    }
}
