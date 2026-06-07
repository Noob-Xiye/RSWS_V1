//! 管理员 OSS 配置 Handler

use crate::state::get_state;
use rsws_common::ResponseExt;
use rsws_service::config_service::OssStorageConfig;
use salvo::oapi::extract::JsonBody;
use salvo::prelude::*;
use tracing::error;

/// 获取 OSS 存储配置
#[endpoint]
pub async fn get_storage_config(_req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let state = get_state(depot);

    match state.config_service.get_storage_config().await {
        Ok(config) => res.success(config),
        Err(e) => {
            error!("Failed to get storage config: {}", e);
            res.error(e);
        }
    }
}

/// 保存 OSS 存储配置
#[endpoint]
pub async fn update_storage_config(
    _req: &mut Request,
    depot: &mut Depot,
    body: JsonBody<OssStorageConfig>,
    res: &mut Response,
) {
    let config = body.into_inner();
    let state = get_state(depot);

    // 参数验证
    if config.provider.is_empty() {
        return res.http_error(StatusCode::BAD_REQUEST, "存储提供商不能为空");
    }
    if config.provider != "local" {
        if config.endpoint.is_empty()
            || config.bucket.is_empty()
            || config.access_key.is_empty()
            || config.secret_key.is_empty()
        {
            return res.http_error(
                StatusCode::BAD_REQUEST,
                "Endpoint/Bucket/AccessKey/SecretKey 不能为空",
            );
        }
    } else if config.endpoint.is_empty() {
        return res.http_error(StatusCode::BAD_REQUEST, "本地存储路径不能为空");
    }

    match state.config_service.save_storage_config(&config).await {
        Ok(_) => res.success_msg((), "OSS 配置保存成功"),
        Err(e) => {
            error!("Failed to save storage config: {}", e);
            res.error(e);
        }
    }
}

/// 测试 OSS 连接
#[endpoint]
pub async fn test_storage_connection(
    _req: &mut Request,
    depot: &mut Depot,
    body: JsonBody<OssStorageConfig>,
    res: &mut Response,
) {
    let config = body.into_inner();
    let _state = get_state(depot);

    let storage_service = match rsws_service::oss_service::StorageService::new(&config).await {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to create storage service: {}", e);
            return res.http_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("创建存储服务失败: {}", e),
            );
        }
    };

    let test_key = format!(
        "{}/test_{}.txt",
        config.prefix.trim_end_matches('/'),
        chrono::Utc::now().timestamp()
    );
    let test_data = bytes::Bytes::from("RSWS OSS Connection Test");

    match storage_service
        .upload(&test_key, test_data, Some("text/plain"))
        .await
    {
        Ok(result) => {
            let _ = storage_service.delete(&test_key).await;
            res.success_msg(
                result.url.clone(),
                format!("连接测试成功！文件已上传到: {}", result.url),
            );
        }
        Err(e) => {
            error!("OSS connection test failed: {}", e);
            res.http_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("OSS 连接测试失败: {}", e),
            );
        }
    }
}
