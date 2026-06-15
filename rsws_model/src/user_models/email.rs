//! 邮箱变更模型

use serde::{Deserialize, Serialize};

/// 发送邮箱变更验证码请求
#[derive(Debug, Deserialize)]
pub struct SendEmailChangeCodeRequest {
    pub new_email: String,
}

/// 发送邮箱变更验证码响应
#[derive(Debug, Serialize)]
pub struct SendEmailChangeCodeResponse {
    pub success: bool,
    pub message: String,
    /// 过期时间（秒）
    pub expires_in: i64,
}

/// 验证邮箱变更请求
#[derive(Debug, Deserialize)]
pub struct VerifyEmailChangeRequest {
    pub new_email: String,
    pub code: String,
}

/// 验证邮箱变更响应
#[derive(Debug, Serialize)]
pub struct VerifyEmailChangeResponse {
    pub success: bool,
    pub message: String,
}

// ==================== 单元测试 ====================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_send_email_change_code_request() {
        let req = SendEmailChangeCodeRequest {
            new_email: "new@example.com".to_string(),
        };

        assert_eq!(req.new_email, "new@example.com");
    }

    #[test]
    fn test_verify_email_change_request() {
        let req = VerifyEmailChangeRequest {
            new_email: "new@example.com".to_string(),
            code: "123456".to_string(),
        };

        assert_eq!(req.code, "123456");
    }
}
