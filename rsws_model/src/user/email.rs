use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SendEmailChangeCodeRequest {
    pub new_email: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct SendEmailChangeCodeResponse {
    pub success: bool,
    pub message: String,
    pub expires_in: i64, // 过期时间（秒）
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct VerifyEmailChangeRequest {
    pub new_email: String,
    pub code: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct VerifyEmailChangeResponse {
    pub success: bool,
    pub message: String,
}
