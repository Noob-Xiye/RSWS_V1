use chrono::Utc;
use rsws_common::error::ServiceError;
use rsws_common::snowflake;
use rsws_model::resource::Resource;
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

// 定义请求和响应结构体
#[derive(Debug, Clone)]
pub struct UploadResourceRequest {
    pub title: String,
    pub description: Option<String>,
    pub detail_description: Option<String>,
    pub specifications: Option<serde_json::Value>,
    pub usage_guide: Option<String>,
    pub precautions: Option<String>,
    pub display_images: Option<Vec<String>>,
    pub file_name: String,
    pub file_size: u64,
    pub content_type: String,
    pub price: Decimal,
    pub category_id: Option<i64>,
    pub file: FileUpload,
}

#[derive(Debug, Clone)]
pub struct FileUpload {
    data: Vec<u8>,
}

impl FileUpload {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    pub fn contents(&self) -> &[u8] {
        &self.data
    }
}

#[derive(Debug, Clone)]
pub struct ResourceResponse {
    pub id: i64,
    pub title: String,
    pub description: Option<String>,
    pub detail_description: Option<String>,
    pub specifications: Option<serde_json::Value>,
    pub usage_guide: Option<String>,
    pub precautions: Option<String>,
    pub display_images: Option<Vec<String>>,
    pub file_name: String,
    pub file_size: i64,
    pub content_type: String,
    pub price: Decimal,
    pub status: String,
    pub created_at: chrono::DateTime<Utc>,
}

pub struct ResourceService {
    db_pool: PgPool,
    storage_path: String,
}

impl ResourceService {
    pub fn new(db_pool: PgPool, storage_path: String) -> Self {
        Self {
            db_pool,
            storage_path,
        }
    }

    pub async fn upload_resource(
        &self,
        user_id: i64,
        request: UploadResourceRequest,
    ) -> Result<ResourceResponse, ServiceError> {
        // 生成唯一文件名
        let file_id = snowflake::next_id();
        let extension = Path::new(&request.file_name)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        let storage_filename = format!("{}.{}", file_id, extension);
        let storage_path = Path::new(&self.storage_path).join(storage_filename.clone());

        // 保存文件
        let mut file = File::create(storage_path)
            .await
            .map_err(|e| ServiceError::IoError(format!("Failed to create file: {}", e)))?;

        let content = request.file.contents();
        file.write_all(content)
            .await
            .map_err(|e| ServiceError::IoError(format!("Failed to write file: {}", e)))?;

        // 创建资源记录
        let resource = sqlx::query_as::<_, Resource>(
            r#"
            INSERT INTO resources 
            (id, user_id, title, description, detail_description, specifications, 
             usage_guide, precautions, display_images, file_name, storage_filename, 
             file_size, content_type, price, category_id, provider_type, provider_id, 
             commission_rate, status)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, 'user', $2, 0.1, 'pending')
            RETURNING *
            "#
        )
        .bind(file_id)
        .bind(user_id)
        .bind(&request.title)
        .bind(&request.description)
        .bind(&request.detail_description)
        .bind(&request.specifications)
        .bind(&request.usage_guide)
        .bind(&request.precautions)
        .bind(&request.display_images)
        .bind(&request.file_name)
        .bind(&storage_filename)
        .bind(request.file_size as i64)
        .bind(&request.content_type)
        .bind(request.price)
        .bind(request.category_id)
        .fetch_one(&self.db_pool)
        .await
        .map_err(|e| ServiceError::DbError(format!("Failed to create resource record: {}", e)))?;

        // 构建响应
        Ok(ResourceResponse {
            id: resource.id,
            title: resource.title,
            description: resource.description,
            detail_description: resource.detail_description,
            specifications: resource.specifications,
            usage_guide: resource.usage_guide,
            precautions: resource.precautions,
            display_images: resource.display_images,
            file_name: request.file_name,
            file_size: request.file_size as i64,
            content_type: request.content_type,
            price: resource.price,
            status: "pending".to_string(),
            created_at: resource.created_at,
        })
    }
}
