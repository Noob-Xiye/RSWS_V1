//! 邮件服务
//!
//! 基于 lettre 的 SMTP 邮件发送服务

use crate::error::RswsError;
use crate::error_code::ErrorCode;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};

/// 邮件配置
#[derive(Debug, Clone)]
pub struct EmailConfig {
    pub smtp_server: String,
    pub smtp_username: String,
    pub smtp_password: String,
    pub from_email: String,
}

/// 邮件服务
pub struct EmailService {
    smtp_transport: SmtpTransport,
    from_email: String,
}

impl EmailService {
    /// 创建邮件服务实例
    pub fn new(config: &EmailConfig) -> Result<Self, RswsError> {
        let smtp_server = config.smtp_server.as_str();
        let smtp_username = config.smtp_username.as_str();
        let smtp_password = config.smtp_password.as_str();

        let creds = Credentials::new(smtp_username.to_string(), smtp_password.to_string());

        let smtp_transport = SmtpTransport::relay(smtp_server)
            .map_err(|e| RswsError::business_with_message(
                ErrorCode::CONFIG_INVALID_VALUE,
                format!("SMTP relay error: {}", e)
            ))?
            .credentials(creds)
            .build();

        Ok(Self {
            smtp_transport,
            from_email: config.from_email.clone(),
        })
    }

    /// 发送邮件
    pub fn send(&self, to: &str, subject: &str, body: &str) -> Result<(), RswsError> {
        let email = Message::builder()
            .from(
                self.from_email
                    .parse()
                    .map_err(|e| RswsError::bad_request(format!("Invalid from email: {}", e)))?,
            )
            .to(to
                .parse()
                .map_err(|e| RswsError::bad_request(format!("Invalid to email: {}", e)))?)
            .subject(subject)
            .body(body.to_string())
            .map_err(|e| RswsError::internal(format!("Email build error: {}", e)))?;

        self.smtp_transport
            .send(&email)
            .map_err(|e| RswsError::internal(format!("Email send error: {}", e)))?;

        Ok(())
    }

    /// 发送验证码
    pub async fn send_verification_code(
        &self,
        to: &str,
        code: &str,
        code_type: &str,
    ) -> Result<(), RswsError> {
        let subject = match code_type {
            "login" => "Your Login Verification Code",
            "register" => "Your Registration Verification Code",
            "reset_password" => "Your Password Reset Code",
            _ => "Your Verification Code",
        };

        let body = format!(
            r#"Your verification code is: {}

This code will expire in 5 minutes.

If you did not request this code, please ignore this email."#,
            code
        );

        self.send(to, subject, &body)
    }
}

// ==================== 单元测试 ====================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_config_creation() {
        let config = EmailConfig {
            smtp_server: "smtp.example.com".to_string(),
            smtp_username: "user@example.com".to_string(),
            smtp_password: "password".to_string(),
            from_email: "noreply@example.com".to_string(),
        };

        assert_eq!(config.smtp_server, "smtp.example.com");
        assert_eq!(config.from_email, "noreply@example.com");
    }
}
