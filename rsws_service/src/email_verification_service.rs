//! Email verification service
//!
//! Generates, stores, and verifies email verification codes
//! - Production: generate 6-digit random code, send via SMTP
//! - Development: generate fixed code "123456", print to log

use rand::Rng;
use rsws_common::email::EmailService;
use rsws_common::error::RswsError;
use rsws_common::error_code::ErrorCode;
use rsws_db::RedisService;
use std::sync::Arc;
use tracing::{info, warn};

use crate::config_service::EmailDbConfig;

/// Email verification service
#[derive(Clone)]
pub struct EmailVerificationService {
    redis: RedisService,
    email_service: Option<Arc<EmailService>>,
    email_mode: String, // "development" | "production"
}

impl EmailVerificationService {
    /// Create service instance
    pub fn new(redis: RedisService, email_config: Option<&EmailDbConfig>) -> Self {
        let email_mode = email_config
            .map(|c| {
                // Parse mode from provider field
                // "development" / "dev" / "mock" -> dev mode
                // otherwise -> production mode (real SMTP)
                if c.provider.to_lowercase() == "development"
                    || c.provider.to_lowercase() == "dev"
                    || c.provider.to_lowercase() == "mock"
                {
                    "development".to_string()
                } else {
                    "production".to_string()
                }
            })
            .unwrap_or_else(|| "development".to_string());

        let email_service = email_config.and_then(|cfg| {
            if email_mode == "development" {
                // Dev mode: no real SMTP needed
                None
            } else {
                // Production mode: init SMTP
                let email_config = rsws_common::email::EmailConfig {
                    smtp_server: cfg.host.clone(),
                    smtp_username: cfg.username.clone(),
                    smtp_password: cfg.password.clone(),
                    from_email: cfg.from_email.clone(),
                };
                EmailService::new(&email_config).ok().map(Arc::new)
            }
        });

        Self {
            redis,
            email_service,
            email_mode,
        }
    }

    /// Send verification code
    pub async fn send_code(&self, email: &str, code_type: &str) -> Result<(), RswsError> {
        // Check if unexpired code already exists
        if self.redis.has_verification_code(email, code_type).await? {
            return Err(RswsError::business_with_message(
                ErrorCode::RATE_LIMIT_EXCEEDED,
                "Verification code already sent, please try again later",
            ));
        }

        // Generate code
        let code = self.generate_code();

        // Store code in Redis
        self.redis
            .set_verification_code(email, code_type, &code)
            .await?;

        // Send email or mock
        self.send_email(email, &code, code_type).await?;

        info!(
            "Verification code sent to {} (type: {}, mode: {})",
            email, code_type, self.email_mode
        );
        Ok(())
    }

    /// Verify code
    pub async fn verify_code(
        &self,
        email: &str,
        code_type: &str,
        code: &str,
    ) -> Result<bool, RswsError> {
        let (valid, _remaining) = self.redis.verify_code(email, code_type, code).await?;
        Ok(valid)
    }

    /// Generate code
    fn generate_code(&self) -> String {
        if self.email_mode == "development" {
            // Dev mode: fixed code
            "123456".to_string()
        } else {
            // Production mode: 6-digit random code
            let mut rng = rand::rng();
            format!("{:06}", rng.random::<u32>() % 1_000_000)
        }
    }

    /// Send email
    async fn send_email(&self, email: &str, code: &str, code_type: &str) -> Result<(), RswsError> {
        if self.email_mode == "development" {
            // Dev mode: print to log
            warn!(
                "========================================================\n\
                 EMAIL VERIFICATION CODE [DEV MODE]\n\
                 -------------------------------------------------------\n\
                 To: {}\n\
                 Type: {}\n\
                 Code: {}\n\
                 -------------------------------------------------------\n\
                 NOTE: This code is only valid in development mode!\n\
                 ========================================================",
                email, code_type, code
            );
            Ok(())
        } else {
            // Production mode: send via SMTP
            if let Some(ref svc) = self.email_service {
                svc.send_verification_code(email, code, code_type).await?;
                Ok(())
            } else {
                Err(RswsError::internal("Email service not configured"))
            }
        }
    }

    /// Get current mode
    pub fn mode(&self) -> &str {
        &self.email_mode
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_generate_code_dev_mode() {
        // Fixed code in dev mode
    }
}
