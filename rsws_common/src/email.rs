use crate::error::CommonError;
use config::config::Config;
use lettre::{
    message::header::ContentType, transport::smtp::authentication::Credentials, AsyncSmtpTransport,
    AsyncTransport, Message, Tokio1Executor,
};

pub struct EmailService {
    mailer: AsyncSmtpTransport<Tokio1Executor>,
    from_email: String,
}

impl EmailService {
    pub async fn new(config: &Config) -> Result<Self, CommonError> {
        // 从数据库动态配置中获取邮件配置
        let smtp_server = "smtp.gmail.com"; // 这里应该从数据库配置获取
        let smtp_username = "your-email@gmail.com"; // 从数据库配置获取
        let smtp_password = "your-app-password"; // 从数据库配置获取

        let creds = Credentials::new(smtp_username.to_string(), smtp_password.to_string());

        let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(smtp_server)
            .map_err(|e| CommonError::EmailError(format!("SMTP relay error: {}", e)))?
            .credentials(creds)
            .build();

        Ok(Self {
            mailer,
            from_email: smtp_username.to_string(),
        })
    }

    pub async fn send_verification_code(
        &self,
        to_email: &str,
        code: &str,
        code_type: &str,
    ) -> Result<(), CommonError> {
        let subject = match code_type {
            "registration" => "邮箱验证码 - 注册账户",
            "password_reset" => "邮箱验证码 - 重置密码",
            _ => "邮箱验证码",
        };

        let body = format!(
            r#"
            <html>
            <body>
                <h2>验证码</h2>
                <p>您的验证码是：<strong style="font-size: 24px; color: #007bff;">{}</strong></p>
                <p>验证码有效期为10分钟，请及时使用。</p>
                <p>如果这不是您的操作，请忽略此邮件。</p>
            </body>
            </html>
            "#,
            code
        );

        let email = Message::builder()
            .from(
                self.from_email
                    .parse()
                    .map_err(|e| CommonError::EmailError(format!("Invalid from email: {}", e)))?,
            )
            .to(to_email
                .parse()
                .map_err(|e| CommonError::EmailError(format!("Invalid to email: {}", e)))?)
            .subject(subject)
            .header(ContentType::TEXT_HTML)
            .body(body)
            .map_err(|e| CommonError::EmailError(format!("Failed to build email: {}", e)))?;

        self.mailer
            .send(email)
            .await
            .map_err(|e| CommonError::EmailError(format!("Failed to send email: {}", e)))?;

        Ok(())
    }
}
