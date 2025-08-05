use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
    pub confirm_password: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ChangePasswordResponse {
    pub success: bool,
    pub message: String,
}
