use crate::error::CommonError;
use config::EmailConfig;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};

pub struct EmailService {
    smtp_transport: SmtpTransport,
    from_email: String,
}

impl EmailService {
    pub async fn new(config: &EmailConfig) -> Result<Self, CommonError> {
        // TODO: 从配置中读取SMTP设置
        let smtp_server = config.smtp_server.as_str();
        let smtp_username = config.smtp_username.as_str();
        let smtp_password = config.smtp_password.as_str();
        let from_email = config.from_email.as_str();

        let creds = Credentials::new(smtp_username.to_string(), smtp_password.to_string());

        let smtp_transport = SmtpTransport::relay(smtp_server)
            .map_err(|e| CommonError::Email(format!("SMTP relay error: {}", e)))?
            .credentials(creds)
            .build();

        Ok(Self {
            smtp_transport,
            from_email,
        })
    }

    pub async fn send_email(&self, to: &str, subject: &str, body: &str) -> Result<(), CommonError> {
        let email = Message::builder()
            .from(
                self.from_email
                    .parse()
                    .map_err(|e| CommonError::Email(format!("Invalid from email: {}", e)))?,
            )
            .to(to
                .parse()
                .map_err(|e| CommonError::Email(format!("Invalid to email: {}", e)))?)
            .subject(subject)
            .body(body.to_string())
            .map_err(|e| CommonError::Email(format!("Email build error: {}", e)))?;

        self.smtp_transport
            .send(&email)
            .map_err(|e| CommonError::Email(format!("Email send error: {}", e)))?;

        Ok(())
    }
}
