use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct UserProfile {
    pub id: i64,
    pub nickname: String,
    pub email: String,
    pub avatar: Option<String>,
    pub is_email_verified: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, schemars::JsonSchema)]
pub struct UpdateProfileRequest {
    pub nickname: Option<String>,
    pub avatar: Option<String>,
}

// 在现有代码后添加

#[derive(Debug, Serialize, JsonSchema)]
pub struct ProfileCompletionResponse {
    pub completion_percentage: f32,
    pub missing_fields: Vec<String>,
    pub suggestions: Vec<String>,
}
