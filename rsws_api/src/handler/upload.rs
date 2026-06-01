//! 分块文件上传处理器
//!
//! 支持大文件分块上传（几十MB到几GB），流程：
//! 1. POST /api/v1/upload/init — 初始化上传（返回 upload_id）
//! 2. POST /api/v1/upload/chunk — 上传单个分块（JSON body: base64 data）
//! 3. POST /api/v1/upload/complete — 合并所有分块，返回文件路径

use crate::state::get_state;
use base64::Engine;
use rsws_common::{error_code::ErrorCode, AuthHandler, ResponseExt, RswsError};
use salvo::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;
use tracing::{error, info};

/// 分块大小：8MB
pub const CHUNK_SIZE: usize = 8 * 1024 * 1024;

/// 初始化上传请求
#[derive(Debug, Deserialize, Serialize, salvo_oapi::ToSchema)]
pub struct InitUploadRequest {
    pub filename: String,
    pub total_size: u64,
    pub file_md5: Option<String>,
}

/// 初始化上传响应
#[derive(Debug, Serialize, salvo_oapi::ToSchema)]
pub struct InitUploadResponse {
    pub upload_id: String,
    pub chunk_size: usize,
    pub total_chunks: u32,
}

/// 上传分块请求
#[derive(Debug, Deserialize, salvo_oapi::ToSchema)]
pub struct ChunkUploadRequest {
    pub upload_id: String,
    pub chunk_index: u32,
    /// Base64 编码的分块数据
    pub data: String,
}

/// 完成上传请求
#[derive(Debug, Deserialize, Serialize, salvo_oapi::ToSchema)]
pub struct CompleteUploadRequest {
    pub upload_id: String,
    pub filename: String,
}

/// 完成上传响应
#[derive(Debug, Serialize, salvo_oapi::ToSchema)]
pub struct CompleteUploadResponse {
    pub file_url: String,
    pub file_size: u64,
}

/// 获取分块临时目录
fn get_chunk_dir(upload_id: &str) -> PathBuf {
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("uploads")
        .join("_chunks")
        .join(upload_id)
}

/// 初始化上传
#[salvo_oapi::endpoint(
    request_body = InitUploadRequest,
    responses(
        (status_code = 200, description = "初始化成功"),
        (status_code = 401, description = "未认证"),
    )
)]
pub async fn init_upload(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let _user_id = match res.auth_require_user_id(depot) {
        Some(id) => id,
        None => return,
    };

    let body = match req.parse_json::<InitUploadRequest>().await {
        Ok(b) => b,
        Err(e) => {
            res.error_msg(
                RswsError::from(ErrorCode::INVALID_REQUEST_FORMAT),
                format!("Invalid request: {}", e),
            );
            return;
        }
    };

    let filename = body.filename.trim();
    if filename.is_empty() {
        res.error_msg(
            RswsError::from(ErrorCode::INVALID_PARAMETER),
            "Filename cannot be empty",
        );
        return;
    }

    const MAX_SIZE: u64 = 5 * 1024 * 1024 * 1024;
    if body.total_size == 0 || body.total_size > MAX_SIZE {
        res.error_msg(
            RswsError::from(ErrorCode::INVALID_PARAMETER),
            format!("File size must be between 1 and {} bytes", MAX_SIZE),
        );
        return;
    }

    let upload_id = uuid::Uuid::new_v4().to_string();
    let total_chunks = (body.total_size as usize).div_ceil(CHUNK_SIZE) as u32;

    let chunk_dir = get_chunk_dir(&upload_id);
    if let Err(e) = fs::create_dir_all(&chunk_dir) {
        error!("Failed to create chunk dir: {}", e);
        res.error(RswsError::internal("Failed to initialize upload"));
        return;
    }

    let meta = serde_json::json!({
        "filename": body.filename,
        "total_size": body.total_size,
        "file_md5": body.file_md5,
        "total_chunks": total_chunks,
    });
    let meta_path = chunk_dir.join("_meta.json");
    if let Err(e) = fs::write(&meta_path, serde_json::to_string(&meta).unwrap()) {
        error!("Failed to write chunk meta: {}", e);
        let _ = fs::remove_dir_all(&chunk_dir);
        res.error(RswsError::internal("Failed to initialize upload"));
        return;
    }

    info!(
        "Upload initialized: {} file={} size={} chunks={}",
        upload_id, body.filename, body.total_size, total_chunks
    );

    res.success(InitUploadResponse {
        upload_id,
        chunk_size: CHUNK_SIZE,
        total_chunks,
    });
}

/// 上传分块（base64 JSON body）
#[salvo_oapi::endpoint(
    request_body = ChunkUploadRequest,
    responses(
        (status_code = 200, description = "分块上传成功"),
        (status_code = 401, description = "未认证"),
    )
)]
pub async fn upload_chunk(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let _user_id = match res.auth_require_user_id(depot) {
        Some(id) => id,
        None => return,
    };

    let body = match req.parse_json::<ChunkUploadRequest>().await {
        Ok(b) => b,
        Err(e) => {
            res.error_msg(
                RswsError::from(ErrorCode::INVALID_REQUEST_FORMAT),
                format!("Invalid request: {}", e),
            );
            return;
        }
    };

    if body.upload_id.is_empty() {
        res.error_msg(
            RswsError::from(ErrorCode::INVALID_PARAMETER),
            "Missing upload_id",
        );
        return;
    }

    let chunk_dir = get_chunk_dir(&body.upload_id);
    if !chunk_dir.exists() {
        res.error_msg(
            RswsError::from(ErrorCode::INVALID_PARAMETER),
            "Invalid or expired upload_id",
        );
        return;
    }

    // 解码 base64
    let chunk_data = match base64::engine::general_purpose::STANDARD.decode(&body.data) {
        Ok(d) => d,
        Err(e) => {
            res.error_msg(
                RswsError::from(ErrorCode::INVALID_REQUEST_FORMAT),
                format!("Invalid base64 data: {}", e),
            );
            return;
        }
    };

    let received = chunk_data.len();
    let chunk_path = chunk_dir.join(format!("chunk_{:06}", body.chunk_index));
    let mut f = match tokio::fs::File::create(&chunk_path).await {
        Ok(f) => f,
        Err(e) => {
            error!("Failed to create chunk file: {}", e);
            res.error(RswsError::internal("Failed to save chunk"));
            return;
        }
    };

    if let Err(e) = f.write_all(&chunk_data).await {
        error!("Failed to write chunk: {}", e);
        res.error(RswsError::internal("Failed to save chunk"));
        return;
    }

    info!(
        "Chunk uploaded: {} index={} size={}",
        body.upload_id, body.chunk_index, received
    );

    res.success(serde_json::json!({
        "chunk_index": body.chunk_index,
        "received": received,
        "message": "Chunk uploaded"
    }));
}

/// 完成上传（合并分块）
#[salvo_oapi::endpoint(
    request_body = CompleteUploadRequest,
    responses(
        (status_code = 200, description = "合并完成"),
        (status_code = 401, description = "未认证"),
    )
)]
pub async fn complete_upload(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let _user_id = match res.auth_require_user_id(depot) {
        Some(id) => id,
        None => return,
    };

    let body = match req.parse_json::<CompleteUploadRequest>().await {
        Ok(b) => b,
        Err(e) => {
            res.error_msg(
                RswsError::from(ErrorCode::INVALID_REQUEST_FORMAT),
                format!("Invalid request: {}", e),
            );
            return;
        }
    };

    let chunk_dir = get_chunk_dir(&body.upload_id);
    if !chunk_dir.exists() {
        res.error_msg(
            RswsError::from(ErrorCode::INVALID_PARAMETER),
            "Invalid or expired upload_id",
        );
        return;
    }

    let meta_path = chunk_dir.join("_meta.json");
    let meta_str = match fs::read_to_string(&meta_path) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to read meta: {}", e);
            res.error(RswsError::internal("Upload metadata lost"));
            return;
        }
    };
    let meta: serde_json::Value = match serde_json::from_str(&meta_str) {
        Ok(m) => m,
        Err(_) => {
            res.error(RswsError::internal("Upload metadata corrupted"));
            return;
        }
    };
    let total_chunks: u32 = meta["total_chunks"].as_u64().unwrap_or(0) as u32;
    let original_filename = meta["filename"].as_str().unwrap_or(&body.filename);

    let chunk_files: Vec<_> = (0..total_chunks)
        .map(|i| chunk_dir.join(format!("chunk_{:06}", i)))
        .collect();
    for cf in &chunk_files {
        if !cf.exists() {
            res.error_msg(
                RswsError::from(ErrorCode::INVALID_PARAMETER),
                format!("Missing chunk {}", cf.display()),
            );
            return;
        }
    }

    let state = get_state(depot);
    let now = chrono::Utc::now();
    let relative_path = format!(
        "resources/{}/{}/{}-{}",
        now.format("%Y"),
        now.format("%m"),
        uuid::Uuid::new_v4(),
        sanitize_filename(original_filename)
    );
    let dest_path = PathBuf::from(&state.config.server.upload_dir).join(&relative_path);

    if let Err(e) = fs::create_dir_all(dest_path.parent().unwrap()) {
        error!("Failed to create dest dir: {}", e);
        res.error(RswsError::internal("Failed to complete upload"));
        return;
    }

    let mut dest_file = match tokio::fs::File::create(&dest_path).await {
        Ok(f) => f,
        Err(e) => {
            error!("Failed to create dest file: {}", e);
            res.error(RswsError::internal("Failed to complete upload"));
            return;
        }
    };

    let mut total_written: u64 = 0;
    for cf in &chunk_files {
        let data = match fs::read(cf) {
            Ok(d) => d,
            Err(e) => {
                error!("Failed to read chunk {}: {}", cf.display(), e);
                let _ = tokio::fs::remove_file(&dest_path).await;
                res.error(RswsError::internal("Failed to merge chunks"));
                return;
            }
        };
        if let Err(e) = dest_file.write_all(&data).await {
            error!("Failed to write chunk to dest: {}", e);
            let _ = tokio::fs::remove_file(&dest_path).await;
            res.error(RswsError::internal("Failed to merge chunks"));
            return;
        }
        total_written += data.len() as u64;
    }
    dest_file.flush().await.ok();

    let _ = fs::remove_dir_all(&chunk_dir);

    // 根据存储配置决定文件 URL
    let file_url = match state.config_service.get_storage_config().await {
        Ok(storage_config) if storage_config.is_active && !storage_config.is_local() => {
            if let Some(ref domain) = storage_config.custom_domain {
                format!(
                    "https://{}/{}{}",
                    domain, storage_config.prefix, relative_path
                )
            } else {
                format!(
                    "https://{}.s3.{}.amazonaws.com/{}{}",
                    storage_config.bucket,
                    storage_config.region,
                    storage_config.prefix,
                    relative_path
                )
            }
        }
        Ok(_) => format!("/uploads/{}", relative_path),
        Err(e) => {
            tracing::warn!("Failed to get storage config, using local: {}", e);
            format!("/uploads/{}", relative_path)
        }
    };

    info!(
        "Upload completed: {} file={} size={} url={}",
        body.upload_id, relative_path, total_written, file_url
    );

    res.success(CompleteUploadResponse {
        file_url,
        file_size: total_written,
    });
}

fn sanitize_filename(filename: &str) -> String {
    let name = filename.rsplit(['/', '\\', ':']).next().unwrap_or(filename);
    name.chars()
        .filter(|c| !c.is_control() && *c != '\0')
        .collect()
}
