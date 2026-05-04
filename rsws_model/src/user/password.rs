//! 密码变更模型

use serde::{Deserialize, Serialize};

/// 修改密码请求
#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
    pub confirm_password: String,
}

/// 修改密码响应
#[derive(Debug, Serialize)]
pub struct ChangePasswordResponse {
    pub success: bool,
    pub message: String,
}

// ==================== 单元测试 ====================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_change_password_request() {
        let req = ChangePasswordRequest {
            current_password: "OldPass123".to_string(),
            new_password: "NewPass456".to_string(),
            confirm_password: "NewPass456".to_string(),
        };

        assert_eq!(req.new_password, req.confirm_password);
    }
}
