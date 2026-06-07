//! 文件上传 Handler（支持 OSS 分块上传 + 单文件上传）

use crate::state::get_state;
use bytes::Bytes;
use futures_util::StreamExt;
use http_body_util::BodyStream;
use multer::Multipart;
use rand::Rng;
use rsws_common::{ResponseExt, RswsError};
use salvo::oapi::extract::JsonBody;
use salvo::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;
use tracing::{error, info};

// ==================== 统一错误类型 ====================

/// 上传相关错误（统一错误处理）
#[derive(Error, Debug)]
pub enum UploadError {
    #[error("缺少 content-type 头")]
    MissingContentType,
    
    #[error("无效的 content-type")]
    InvalidContentType,
    
    #[error("Multipart boundary 解析失败: {0}")]
    BoundaryParseFailed(String),
    
    #[error("Content-Type 不是 multipart/form-data")]
    NoMultipart,
    
    #[error("请求体读取失败: {0}")]
    BodyReadFailed(String),
    
    #[error("Multipart 解析错误: {0}")]
    MultipartParseError(String),
    
    #[error("读取文件数据失败: {0}")]
    FileReadError(String),
    
    #[error("未找到上传文件")]
    FileNotFound,
    
    #[error("文件大小超过限制: {0}")]
    FileSizeExceeded(String),
    
    #[error("未知上传错误: {0}")]
    Unknown(String),
}

impl From<multer::Error> for UploadError {
    fn from(err: multer::Error) -> Self {
        match err {
            multer::Error::NoMultipart => UploadError::NoMultipart,
            multer::Error::NoBoundary => UploadError::BoundaryParseFailed("未找到 boundary".to_string()),
            multer::Error::StreamReadFailed(e) => UploadError::BodyReadFailed(e.to_string()),
            multer::Error::FieldSizeExceeded { limit, field_name } => {
                let name = field_name.unwrap_or_else(|| "<unknown>".to_string());
                UploadError::FileSizeExceeded(format!("字段 {} 超过大小限制: {} bytes", name, limit))
            }
            multer::Error::StreamSizeExceeded { limit } => {
                UploadError::FileSizeExceeded(format!("流大小超过限制: {} bytes", limit))
            }
            multer::Error::IncompleteStream => UploadError::MultipartParseError("不完整的 multipart 流".to_string()),
            multer::Error::IncompleteHeaders => UploadError::MultipartParseError("不完整的字段头".to_string()),
            multer::Error::UnknownField { field_name } => {
                let name = field_name.unwrap_or_else(|| "<unknown>".to_string());
                UploadError::MultipartParseError(format!("未知字段: {}", name))
            }
            multer::Error::IncompleteFieldData { field_name } => {
                let name = field_name.unwrap_or_else(|| "<unknown>".to_string());
                UploadError::FileReadError(format!("字段 {} 数据不完整", name))
            }
            multer::Error::ReadHeaderFailed(e) => UploadError::MultipartParseError(format!("读取头失败: {:?}", e)),
            multer::Error::DecodeHeaderName { name, .. } => UploadError::MultipartParseError(format!("解码头名失败: {:?}", name)),
            multer::Error::DecodeHeaderValue { .. } => UploadError::MultipartParseError("解码头值失败".to_string()),
            multer::Error::DecodeContentType(_) => UploadError::InvalidContentType,
            multer::Error::LockFailure => UploadError::MultipartParseError("锁定 multipart 状态失败".to_string()),
            // 处理未来可能添加的变体和 json feature
            _ => UploadError::Unknown(err.to_string()),
        }
    }
}

impl From<UploadError> for RswsError {
    fn from(err: UploadError) -> Self {
        RswsError::BadRequest(err.to_string())
    }
}

/// 初始化上传请求
#[derive(Debug, Deserialize, salvo_oapi::ToSchema)]
pub struct InitUploadRequest {
    pub filename: String,
    pub file_size: i64,
    pub chunk_size: Option<i64>,
    pub content_type: Option<String>,
}

/// 初始化上传响应
#[derive(Debug, Serialize, salvo_oapi::ToSchema)]
pub struct InitUploadResponse {
    pub upload_id: String,
    pub chunk_size: i64,
    pub total_chunks: i32,
    pub file_key: String,
}

/// 完成上传请求
#[derive(Debug, Deserialize, salvo_oapi::ToSchema)]
pub struct CompleteUploadRequest {
    pub upload_id: String,
    pub file_key: String,
}

/// 完成上传响应
#[derive(Debug, Serialize, salvo_oapi::ToSchema)]
pub struct CompleteUploadResponse {
    pub file_url: String,
    pub file_size: i64,
    pub etag: Option<String>,
}

/// 单文件上传响应
#[derive(Debug, Serialize, salvo_oapi::ToSchema)]
pub struct SingleUploadResponse {
    pub file_url: String,
    pub file_key: String,
    pub file_size: i64,
    pub content_type: Option<String>,
}

/// 上传会话（存储在 Redis）
#[derive(Debug, Serialize, Deserialize)]
struct UploadSession {
    file_key: String,
    filename: String,
    file_size: i64,
    chunk_size: i64,
    total_chunks: i32,
    content_type: Option<String>,
    uploaded_chunks: Vec<i32>,
    created_at: i64,
}

// ==================== Handler ====================

/// 初始化上传
#[endpoint]
pub async fn init_upload(
    _req: &mut Request,
    depot: &mut Depot,
    body: JsonBody<InitUploadRequest>,
    res: &mut Response,
) {
    let req = body.into_inner();
    let state = get_state(depot);

    if req.file_size <= 0 {
        return res.http_error(StatusCode::BAD_REQUEST, "文件大小必须大于 0");
    }
    if req.filename.is_empty() {
        return res.http_error(StatusCode::BAD_REQUEST, "文件名不能为空");
    }

    let upload_id = chrono::Utc::now().timestamp_millis().to_string();
    let file_key = generate_file_key(&req.filename);
    let chunk_size = req.chunk_size.unwrap_or(5 * 1024 * 1024);
    let total_chunks = ((req.file_size + chunk_size - 1) / chunk_size) as i32;

    let session = UploadSession {
        file_key: file_key.clone(),
        filename: req.filename,
        file_size: req.file_size,
        chunk_size,
        total_chunks,
        content_type: req.content_type,
        uploaded_chunks: Vec::new(),
        created_at: chrono::Utc::now().timestamp(),
    };

    let session_key = format!("upload_session:{}", upload_id);
    if let Err(e) = state.config_service.redis_client()
        .set_json(&session_key, &session, 3600)
        .await
    {
        error!("Failed to save session: {}", e);
        return res.error(RswsError::internal("保存上传会话失败"));
    }

    info!("Upload initialized: upload_id={}, file_key={}, chunks={}",
          upload_id, file_key, total_chunks);

    res.success(InitUploadResponse {
        upload_id,
        chunk_size,
        total_chunks,
        file_key,
    });
}

/// 上传分块（POST multipart/form-data，upload_id 和 chunk_index 通过查询参数传入）
#[endpoint]
pub async fn upload_chunk(
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
) {
    let state = get_state(depot);

    // 解析查询参数
    let upload_id: String = req.parse_queries()
        .map_err(|_| StatusCode::BAD_REQUEST)
        .and_then(|p: UploadChunkQuery| Ok(p.upload_id))
        .unwrap_or_default();
    let chunk_index: i32 = req.parse_queries()
        .map_err(|_| StatusCode::BAD_REQUEST)
        .and_then(|p: UploadChunkQuery| Ok(p.chunk_index))
        .unwrap_or(-1);

    if upload_id.is_empty() {
        return res.http_error(StatusCode::BAD_REQUEST, "缺少 upload_id");
    }
    if chunk_index < 0 {
        return res.http_error(StatusCode::BAD_REQUEST, "缺少 chunk_index");
    }

    let session_key = format!("upload_session:{}", upload_id);
    let mut session: UploadSession = match state.config_service.redis_client()
        .get_json(&session_key)
        .await
    {
        Ok(Some(s)) => s,
        Ok(None) => return res.http_error(StatusCode::NOT_FOUND, "上传会话不存在或已过期"),
        Err(e) => {
            error!("Redis get error: {}", e);
            return res.error(RswsError::internal("读取上传会话失败"));
        }
    };

    if chunk_index < 0 || chunk_index >= session.total_chunks {
        return res.http_error(StatusCode::BAD_REQUEST, "无效的分块索引");
    }

    // 读取分块数据（只取 Bytes 部分）
    let chunk_data = match read_multipart_file(req).await {
        Ok((data, _, _)) => data,
        Err(e) => return res.http_error(StatusCode::BAD_REQUEST, e.to_string()),
    };

    // 获取 OSS 配置并上传
    let oss_config = match state.config_service.get_storage_config().await {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to get storage config: {}", e);
            return res.error(RswsError::internal("获取存储配置失败"));
        }
    };

    let storage_service = match rsws_service::oss_service::StorageService::new(&oss_config).await {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to create storage service: {}", e);
            return res.error(RswsError::internal("创建存储服务失败"));
        }
    };

    let chunk_key = format!("{}.chunk_{}", session.file_key, chunk_index);
    if let Err(e) = storage_service.upload(&chunk_key, chunk_data, session.content_type.as_deref()).await {
        error!("Failed to upload chunk: {}", e);
        return res.error(RswsError::internal("上传分块失败"));
    }

    session.uploaded_chunks.push(chunk_index);
    session.uploaded_chunks.sort();

    if let Err(e) = state.config_service.redis_client()
        .set_json(&session_key, &session, 3600)
        .await
    {
        error!("Failed to save session: {}", e);
        return res.error(RswsError::internal("保存上传会话失败"));
    }

    info!("Chunk uploaded: upload_id={}, chunk={}/{}",
          upload_id, chunk_index + 1, session.total_chunks);

    res.success_msg((), format!("分块 {}/{} 上传成功", chunk_index + 1, session.total_chunks));
}

/// 完成上传
#[endpoint]
pub async fn complete_upload(
    _req: &mut Request,
    depot: &mut Depot,
    body: JsonBody<CompleteUploadRequest>,
    res: &mut Response,
) {
    let req = body.into_inner();
    let state = get_state(depot);

    let session_key = format!("upload_session:{}", req.upload_id);
    let session: UploadSession = match state.config_service.redis_client()
        .get_json(&session_key)
        .await
    {
        Ok(Some(s)) => s,
        Ok(None) => return res.http_error(StatusCode::NOT_FOUND, "上传会话不存在或已过期"),
        Err(e) => {
            error!("Redis get error: {}", e);
            return res.error(RswsError::internal("读取上传会话失败"));
        }
    };

    if session.uploaded_chunks.len() != session.total_chunks as usize {
        return res.http_error(StatusCode::BAD_REQUEST, format!(
            "还有 {} 个分块未上传",
            session.total_chunks - session.uploaded_chunks.len() as i32
        ));
    }

    let oss_config = match state.config_service.get_storage_config().await {
        Ok(c) => c,
        Err(_e) => return res.error(RswsError::internal("获取存储配置失败")),
    };

    let storage_service = match rsws_service::oss_service::StorageService::new(&oss_config).await {
        Ok(s) => s,
        Err(_e) => return res.error(RswsError::internal("创建存储服务失败")),
    };

    let file_url = if session.total_chunks == 1 {
        let chunk_key = format!("{}.chunk_0", session.file_key);
        let data = match storage_service.download(&chunk_key).await {
            Ok(d) => d,
            Err(e) => return res.error(RswsError::internal(format!("下载分块失败: {}", e))),
        };
        let result = match storage_service.upload(&session.file_key, data, session.content_type.as_deref()).await {
            Ok(r) => r,
            Err(e) => return res.error(RswsError::internal(format!("上传最终文件失败: {}", e))),
        };
        let _ = storage_service.delete(&chunk_key).await;
        result.url
    } else {
        let mut complete_data = Vec::new();
        for i in 0..session.total_chunks {
            let chunk_key = format!("{}.chunk_{}", session.file_key, i);
            let chunk_data = match storage_service.download(&chunk_key).await {
                Ok(d) => d,
                Err(e) => return res.error(RswsError::internal(format!("下载分块 {} 失败: {}", i, e))),
            };
            complete_data.extend_from_slice(&chunk_data);
            let _ = storage_service.delete(&chunk_key).await;
        }
        let result = match storage_service.upload(&session.file_key, Bytes::from(complete_data), session.content_type.as_deref()).await {
            Ok(r) => r,
            Err(e) => return res.error(RswsError::internal(format!("上传最终文件失败: {}", e))),
        };
        result.url
    };

    if let Err(e) = state.config_service.redis_client().del(&session_key).await {
        error!("Failed to delete session: {}", e);
    }

    info!("Upload completed: upload_id={}, file_url={}", req.upload_id, file_url);

    res.success(CompleteUploadResponse {
        file_url,
        file_size: session.file_size,
        etag: None,
    });
}

/// 单文件上传
#[endpoint]
pub async fn upload_single(
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
) {
    let state = get_state(depot);

    let (file_data, filename, content_type) = match read_multipart_file(req).await {
        Ok((data, name, ct)) => (data, name, ct),
        Err(e) => return res.error(RswsError::from(e)),
    };

    let file_key = generate_file_key(&filename);

    let oss_config = match state.config_service.get_storage_config().await {
        Ok(c) => c,
        Err(_e) => return res.error(RswsError::internal("获取存储配置失败")),
    };

    let storage_service = match rsws_service::oss_service::StorageService::new(&oss_config).await {
        Ok(s) => s,
        Err(_e) => return res.error(RswsError::internal("创建存储服务失败")),
    };

    let result = match storage_service.upload(&file_key, file_data.clone(), content_type.as_deref()).await {
        Ok(r) => r,
        Err(e) => return res.error(RswsError::internal(format!("上传文件失败: {}", e))),
    };

    info!("Single file uploaded: file_key={}, url={}", file_key, result.url);

    res.success(SingleUploadResponse {
        file_url: result.url,
        file_key,
        file_size: file_data.len() as i64,
        content_type,
    });
}

// ==================== 辅助函数 ====================

/// 解析查询参数
#[derive(Debug, Deserialize, Default)]
struct UploadChunkQuery {
    upload_id: String,
    chunk_index: i32,
}

/// 从 multipart 中读取第一个文件字段
async fn read_multipart_file(req: &mut Request) -> Result<(Bytes, String, Option<String>), UploadError> {
    // 获取 content-type 头
    let content_type = req.headers().get("content-type")
        .ok_or(UploadError::MissingContentType)?
        .to_str()
        .map_err(|_| UploadError::InvalidContentType)?;
    
    // 解析 boundary
    let boundary = multer::parse_boundary(content_type)
        .map_err(|e| UploadError::BoundaryParseFailed(e.to_string()))?;
    
    // 获取请求体并转换为 multer 所需的流类型
    let body = req.take_body();
    let body_stream = BodyStream::new(body);
    
    // 转换 Stream<Item = Result<Frame<Bytes>, salvo::Error>> 
    // 到 Stream<Item = Result<Bytes, multer::Error>>
    let stream = body_stream.map(|result| {
        match result {
            Ok(frame) => {
                let bytes = frame.into_data().unwrap_or_default();
                Ok(bytes)
            }
            Err(err) => {
                // 将 salvo::Error 转换为 multer::Error
                Err(multer::Error::StreamReadFailed(
                    Box::new(std::io::Error::new(std::io::ErrorKind::Other, err.to_string()))
                ))
            }
        }
    });
    
    // 创建 multipart 解析器
    let mut multipart = Multipart::new(stream, &boundary);
    
    // 查找文件字段
    while let Some(field) = multipart.next_field().await.map_err(UploadError::from)? {
        let name = field.name().unwrap_or("").to_string();
        if name == "file" {
            let filename = field.file_name().map(|f| f.to_string());
            let content_type = field.content_type().map(|ct| ct.to_string());
            let data = field.bytes().await.map_err(UploadError::from)?;
            return Ok((data, filename.unwrap_or_else(|| "unknown".to_string()), content_type));
        }
    }
    
    Err(UploadError::FileNotFound)
}

fn generate_file_key(filename: &str) -> String {
    let path_buf = PathBuf::from(filename);
    let ext = path_buf.extension().and_then(|e| e.to_str()).unwrap_or("");
    let timestamp = chrono::Utc::now().timestamp();
    let random: String = rand::rng()
        .sample_iter(rand::distr::Alphanumeric)
        .take(8)
        .map(char::from)
        .collect();

    if ext.is_empty() {
        format!("resources/{}/{}_{}", chrono::Utc::now().format("%Y%m%d"), timestamp, random)
    } else {
        format!("resources/{}/{}_{}.{}", chrono::Utc::now().format("%Y%m%d"), timestamp, random, ext)
    }
}
