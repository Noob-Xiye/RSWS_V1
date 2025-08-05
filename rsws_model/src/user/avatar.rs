use schemars::JsonSchema;
use serde::Serialize;

#[derive(Debug, Serialize, JsonSchema)]
pub struct UploadAvatarResponse {
    pub success: bool,
    pub message: String,
    pub avatar_url: Option<String>,
}
