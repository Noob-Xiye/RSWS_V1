//! 用户资料模型

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 用户资料
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: i64,
    pub nickname: String,
    pub email: String,
    pub avatar: Option<String>,
    pub is_email_verified: bool,
    pub created_at: DateTime<Utc>,
}

/// 更新资料请求
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateProfileRequest {
    pub nickname: Option<String>,
    pub avatar: Option<String>,
}

/// 资料完成度响应
#[derive(Debug, Serialize)]
pub struct ProfileCompletionResponse {
    pub completion_percentage: f32,
    pub missing_fields: Vec<String>,
    pub suggestions: Vec<String>,
}

// ==================== 单元测试 ====================

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_user_profile() {
        let profile = UserProfile {
            id: 1,
            nickname: "test".to_string(),
            email: "test@example.com".to_string(),
            avatar: None,
            is_email_verified: true,
            created_at: Utc::now(),
        };

        assert_eq!(profile.nickname, "test");
        assert!(profile.is_email_verified);
    }

    #[test]
    fn test_update_profile_request() {
        let req = UpdateProfileRequest {
            nickname: Some("new_name".to_string()),
            avatar: Some("https://example.com/avatar.png".to_string()),
        };

        assert!(req.nickname.is_some());
    }
}
