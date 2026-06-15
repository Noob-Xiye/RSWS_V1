//! 请求模型

use serde::{Deserialize, Serialize};

/// 创建请求数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRequestData {
    pub method: String,
    pub path: String,
}

/// 请求响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestResponse {
    pub id: String,
}
