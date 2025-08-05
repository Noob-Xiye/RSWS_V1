pub struct ResourceHandler {
    resource_service: Arc<ResourceService>,
}

impl ResourceHandler {
    pub fn new(resource_service: Arc<ResourceService>) -> Self {
        Self { resource_service }
    }
    
    #[handler]
    pub async fn upload_resource(&self, req: &mut Request, res: &mut Response) {
        let user_id = req.ext::<i64>("user_id")
            .ok_or_else(|| {
                res.status_code(StatusCode::UNAUTHORIZED);
                res.render(Json(serde_json::json!({
                    "error": "Unauthorized"
                })));
                return;
            });
            
        // 处理文件上传
        let mut files = match req.files().await {
            Ok(files) => files,
            Err(e) => {
                res.status_code(StatusCode::BAD_REQUEST);
                res.render(Json(serde_json::json!({
                    "error": format!("Failed to process files: {}", e)
                })));
                return;
            }
        };
        
        if files.is_empty() {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(serde_json::json!({
                "error": "No files uploaded"
            })));
            return;
        }
        
        let file = files.remove(0);
        let file_name = file.name().to_string();
        let file_size = file.size();
        let content_type = file.content_type().map(|ct| ct.to_string());
        
        // 获取其他表单字段
        let title = req.form::<String>("title").await.unwrap_or_default();
        let description = req.form::<String>("description").await.unwrap_or_default();
        let detail_description = req.form::<String>("detail_description").await.ok();
        
        // 处理规格参数，可以是JSON字符串
        let specifications_str = req.form::<String>("specifications").await.ok();
        let specifications = specifications_str.and_then(|s| serde_json::from_str(&s).ok());
        
        let usage_guide = req.form::<String>("usage_guide").await.ok();
        let precautions = req.form::<String>("precautions").await.ok();
        
        // 处理展示图片，可以是逗号分隔的URL列表
        let display_images_str = req.form::<String>("display_images").await.ok();
        let display_images = display_images_str.map(|s| {
            s.split(',').map(|url| url.trim().to_string()).collect::<Vec<String>>()
        });
        
        let price = req.form::<f64>("price").await.unwrap_or(0.0);
        let category_id = req.form::<i64>("category_id").await.ok();
        
        let upload_req = UploadResourceRequest {
            title,
            description,
            detail_description,
            specifications,
            usage_guide,
            precautions,
            display_images,
            file_name,
            file_size,
            content_type,
            price,
            category_id,
            file,
        };
        
        match self.resource_service.upload_resource(*user_id, upload_req).await {
            Ok(resource) => {
                res.render(Json(resource));
            },
            Err(e) => {
                res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                res.render(Json(serde_json::json!({
                    "error": format!("Failed to upload resource: {}", e)
                })));
            }
        }
    }
}