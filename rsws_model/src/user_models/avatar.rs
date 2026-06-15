//! 头像模型

use serde::Serialize;

/// 上传头像响应
#[derive(Debug, Serialize)]
pub struct UploadAvatarResponse {
    pub success: bool,
    pub message: String,
    pub avatar_url: Option<String>,
}

// ==================== 单元测试 ====================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_upload_avatar_response() {
        let res = UploadAvatarResponse {
            success: true,
            message: "上传成功".to_string(),
            avatar_url: Some("https://example.com/avatar.png".to_string()),
        };

        assert!(res.success);
        assert!(res.avatar_url.is_some());
    }
}
