//! 请求服务

use rsws_common::error::RswsError;
use rsws_common::snowflake;
use rsws_model::request::{CreateRequestData, RequestResponse};
use tracing::info;

/// 请求服务
pub struct RequestService;

impl RequestService {
    /// 创建请求
    pub async fn create_request(
        &self,
        _request_data: CreateRequestData,
    ) -> Result<RequestResponse, RswsError> {
        let request_id = snowflake::next_id().to_string();

        info!("Creating request with ID: {}", request_id);

        Ok(RequestResponse { id: request_id })
    }
}

// ==================== 单元测试 ====================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_request() {
        let service = RequestService;
        let data = CreateRequestData {
            method: "GET".to_string(),
            path: "/test".to_string(),
        };

        let result = service.create_request(data).await;
        assert!(result.is_ok());
    }
}
