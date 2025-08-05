use salvo::prelude::*;
use serde::{Deserialize, Serialize};
use rsws_service::config_service::ConfigService;
use rsws_model::config::ConfigValue;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct SetConfigRequest {
    pub key: String,
    pub value: ConfigValue,
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ConfigResponse {
    pub key: String,
    pub value: ConfigValue,
    pub description: Option<String>,
}

pub struct ConfigHandler {
    config_service: Arc<ConfigService>,
}

impl ConfigHandler {
    pub fn new(config_service: Arc<ConfigService>) -> Self {
        Self { config_service }
    }

    // 获取配置
    #[handler]
    pub async fn get_config(&self, req: &mut Request, res: &mut Response) {
        let key = req.param::<String>("key").unwrap_or_default();
        
        match self.config_service.get_system_config(&key).await {
            Ok(Some(value)) => {
                let response = ConfigResponse {
                    key,
                    value,
                    description: None, // 可以从数据库获取
                };
                res.render(Json(response));
            },
            Ok(None) => {
                res.status_code(StatusCode::NOT_FOUND);
                res.render(Json(serde_json::json!({
                    "error": "Configuration not found"
                })));
            },
            Err(e) => {
                res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                res.render(Json(serde_json::json!({
                    "error": format!("Failed to get configuration: {}", e)
                })));
            }
        }
    }

    // 设置配置
    #[handler]
    pub async fn set_config(&self, req: &mut Request, res: &mut Response) {
        let request: SetConfigRequest = match req.parse_json().await {
            Ok(req) => req,
            Err(_) => {
                res.status_code(StatusCode::BAD_REQUEST);
                res.render(Json(serde_json::json!({
                    "error": "Invalid request format"
                })));
                return;
            }
        };

        // TODO: 从JWT中获取用户ID
        let user_id = None;

        match self.config_service.set_system_config(
            &request.key,
            request.value,
            request.description,
            user_id,
        ).await {
            Ok(_) => {
                res.render(Json(serde_json::json!({
                    "message": "Configuration updated successfully"
                })));
            },
            Err(e) => {
                res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                res.render(Json(serde_json::json!({
                    "error": format!("Failed to set configuration: {}", e)
                })));
            }
        }
    }

    // 清除配置缓存
    #[handler]
    pub async fn clear_cache(&self, _req: &mut Request, res: &mut Response) {
        self.config_service.clear_cache().await;
        res.render(Json(serde_json::json!({
            "message": "Configuration cache cleared successfully"
        })));
    }
}