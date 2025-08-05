use salvo::prelude::*;
use std::sync::Arc;
use rsws_service::admin_service::AdminService;
use rsws_model::user::admin::*;
use rsws_model::user::role::*;
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct AdminClaims {
    sub: String,        // 主题 (admin ID)
    exp: usize,         // 过期时间
    iat: usize,         // 签发时间
    role: String,       // 角色
    permissions: Vec<String>, // 权限
}

pub struct AdminHandler {
    admin_service: Arc<AdminService>,
    jwt_secret: String,
}

impl AdminHandler {
    pub fn new(admin_service: Arc<AdminService>, jwt_secret: String) -> Self {
        Self {
            admin_service,
            jwt_secret,
        }
    }
    
    // 管理员登录
    #[handler]
    pub async fn login(&self, req: &mut Request, res: &mut Response) {
        let login_req = match req.parse_json::<AdminLoginRequest>().await {
            Ok(req) => req,
            Err(e) => {
                res.status_code(StatusCode::BAD_REQUEST);
                res.render(Json(serde_json::json!({
                    "error": format!("Invalid request: {}", e)
                })));
                return;
            }
        };
        
        let ip = req.remote_addr().map(|addr| addr.ip());
        let user_agent = req.headers().get("User-Agent")
            .and_then(|v| v.to_str().ok());
        
        match self.admin_service.login(login_req, ip, user_agent).await {
            Ok(login_res) => {
                res.render(Json(login_res));
            },
            Err(e) => {
                let (status, message) = match e {
                    rsws_common::error::ServiceError::NotFound(_) => {
                        (StatusCode::UNAUTHORIZED, "Invalid credentials")
                    },
                    rsws_common::error::ServiceError::Unauthorized(msg) => {
                        (StatusCode::UNAUTHORIZED, msg.as_str())
                    },
                    _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
                };
                
                res.status_code(status);
                res.render(Json(serde_json::json!({
                    "error": message
                })));
            }
        }
    }
    
    // 创建管理员
    #[handler]
    pub async fn create_admin(&self, req: &mut Request, depot: &mut Depot, res: &mut Response) {
        // 验证权限
        let admin_id = match self.verify_admin_permission(depot, "admin_manage").await {
            Ok(id) => id,
            Err(status) => {
                res.status_code(status);
                res.render(Json(serde_json::json!({
                    "error": "Insufficient permissions"
                })));
                return;
            }
        };
        
        let create_req = match req.parse_json::<CreateAdminRequest>().await {
            Ok(req) => req,
            Err(e) => {
                res.status_code(StatusCode::BAD_REQUEST);
                res.render(Json(serde_json::json!({
                    "error": format!("Invalid request: {}", e)
                })));
                return;
            }
        };
        
        let ip = req.remote_addr().map(|addr| addr.ip());
        let user_agent = req.headers().get("User-Agent")
            .and_then(|v| v.to_str().ok());
        
        match self.admin_service.create_admin(create_req, Some(admin_id), ip, user_agent).await {
            Ok(admin) => {
                res.render(Json(admin));
            },
            Err(e) => {
                let (status, message) = match e {
                    rsws_common::error::ServiceError::AlreadyExists(msg) => {
                        (StatusCode::CONFLICT, msg)
                    },
                    _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
                };
                
                res.status_code(status);
                res.render(Json(serde_json::json!({
                    "error": message
                })));
            }
        }
    }
    
    // 获取管理员信息
    #[handler]
    pub async fn get_admin_info(&self, req: &mut Request, depot: &mut Depot, res: &mut Response) {
        // 验证权限
        let admin_id = match self.verify_admin_permission(depot, "admin_manage").await {
            Ok(id) => id,
            Err(status) => {
                res.status_code(status);
                res.render(Json(serde_json::json!({
                    "error": "Insufficient permissions"
                })));
                return;
            }
        };
        
        let id = req.param::<i64>("id").unwrap_or(admin_id);
        
        match self.admin_service.get_admin_info(id).await {
            Ok(admin_info) => {
                res.render(Json(admin_info));
            },
            Err(e) => {
                let (status, message) = match e {
                    rsws_common::error::ServiceError::NotFound(msg) => {
                        (StatusCode::NOT_FOUND, msg)
                    },
                    _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
                };
                
                res.status_code(status);
                res.render(Json(serde_json::json!({
                    "error": message
                })));
            }
        }
    }
    
    // 更新管理员信息
    #[handler]
    pub async fn update_admin(&self, req: &mut Request, depot: &mut Depot, res: &mut Response) {
        // 验证权限
        let admin_id = match self.verify_admin_permission(depot, "admin_manage").await {
            Ok(id) => id,
            Err(status) => {
                res.status_code(status);
                res.render(Json(serde_json::json!({
                    "error": "Insufficient permissions"
                })));
                return;
            }
        };
        
        let update_req = match req.parse_json::<UpdateAdminRequest>().await {
            Ok(req) => req,
            Err(e) => {
                res.status_code(StatusCode::BAD_REQUEST);
                res.render(Json(serde_json::json!({
                    "error": format!("Invalid request: {}", e)
                })));
                return;
            }
        };
        
        let ip = req.remote_addr().map(|addr| addr.ip());
        let user_agent = req.headers().get("User-Agent")
            .and_then(|v| v.to_str().ok());
        
        match self.admin_service.update_admin(update_req, admin_id, ip, user_agent).await {
            Ok(admin) => {
                res.render(Json(admin));
            },
            Err(e) => {
                let (status, message) = match e {
                    rsws_common::error::ServiceError::NotFound(msg) => {
                        (StatusCode::NOT_FOUND, msg)
                    },
                    rsws_common::error::ServiceError::AlreadyExists(msg) => {
                        (StatusCode::CONFLICT, msg)
                    },
                    _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
                };
                
                res.status_code(status);
                res.render(Json(serde_json::json!({
                    "error": message
                })));
            }
        }
    }
    
    // 获取管理员列表
    #[handler]
    pub async fn get_admins(&self, req: &mut Request, depot: &mut Depot, res: &mut Response) {
        // 验证权限
        match self.verify_admin_permission(depot, "admin_manage").await {
            Ok(_) => {},
            Err(status) => {
                res.status_code(status);
                res.render(Json(serde_json::json!({
                    "error": "Insufficient permissions"
                })));
                return;
            }
        };
        
        let page = req.query::<i64>("page").unwrap_or(1);
        let page_size = req.query::<i64>("page_size").unwrap_or(10);
        let role = req.query::<String>("role");
        
        match self.admin_service.get_admins(page, page_size, role.as_deref()).await {
            Ok((admins, total)) => {
                res.render(Json(serde_json::json!({
                    "data": admins,
                    "total": total,
                    "page": page,
                    "page_size": page_size
                })));
            },
            Err(_) => {
                res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                res.render(Json(serde_json::json!({
                    "error": "Internal server error"
                })));
            }
        }
    }
    
    // 验证管理员权限
    async fn verify_admin_permission(&self, depot: &mut Depot, required_permission: &str) -> Result<i64, StatusCode> {
        let auth_header = depot.get::<String>("Authorization")
            .ok_or(StatusCode::UNAUTHORIZED)?;
        
        if !auth_header.starts_with("Bearer ") {
            return Err(StatusCode::UNAUTHORIZED);
        }
        
        let token = &auth_header[7..]; // 去掉 "Bearer " 前缀
        
        let claims = decode::<AdminClaims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_bytes()),
            &Validation::new(Algorithm::HS256),
        ).map_err(|_| StatusCode::UNAUTHORIZED)?.claims;
        
        let admin_id = claims.sub.parse::<i64>().map_err(|_| StatusCode::UNAUTHORIZED)?;
        
        // 检查权限
        if claims.permissions.contains(&"admin_manage".to_string()) || 
           claims.permissions.contains(&required_permission.to_string()) || 
           claims.role == "super_admin" {
            Ok(admin_id)
        } else {
            Err(StatusCode::FORBIDDEN)
        }
    }
}