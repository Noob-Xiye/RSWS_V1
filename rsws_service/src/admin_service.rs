//! 管理员服务
//!
//! 管理员 CRUD
//! 认证统一走 API Key,不使用 JWT
//! Admin API Key 管理已迁移至 api_key_manager

use rsws_common::error::RswsError;
use rsws_db::admin::AdminRepository;
use rsws_model::user_models::admin::*;
use std::sync::Arc;

/// 管理员服务
pub struct AdminService {
    admin_repo: Arc<AdminRepository>,
}

impl AdminService {
    /// 创建管理员服务
    pub fn new(admin_repo: AdminRepository) -> Self {
        Self {
            admin_repo: Arc::new(admin_repo),
        }
    }

    /// 管理员登录(验证密码,返回管理员信息)
    /// 注意:登录后客户端需自行使用 Admin API Key 调用受保护接口
    pub async fn login(
        &self,
        email: &str,
        password: &str,
        ip_address: Option<&str>,
    ) -> Result<AdminInfo, RswsError> {
        let admin = self
            .admin_repo
            .verify_admin_credentials(email, password)
            .await?
            .ok_or_else(|| RswsError::unauthorized("Invalid email or password"))?;

        if !admin.is_active {
            return Err(RswsError::forbidden("Account is disabled"));
        }

        // 更新登录信息
        self.admin_repo
            .update_admin_login(admin.id, ip_address)
            .await?;

        // 记录操作日志
        let _ = self
            .admin_repo
            .log_admin_operation(admin.id, "login", None, None, None, ip_address, None)
            .await;

        let permissions: Vec<String> =
            serde_json::from_value(admin.permissions.clone()).unwrap_or_default();

        Ok(AdminInfo {
            id: admin.id,
            email: admin.email,
            username: admin.username,
            nickname: admin.nickname,
            avatar_url: admin.avatar_url,
            is_active: admin.is_active,
            role: admin.role,
            permissions,
        })
    }

    /// 通过 ID 获取管理员信息
    pub async fn get_admin_info(&self, id: i64) -> Result<AdminInfo, RswsError> {
        let admin = self
            .admin_repo
            .get_admin_by_id(id)
            .await?
            .ok_or_else(|| RswsError::not_found("Admin not found"))?;

        let permissions: Vec<String> =
            serde_json::from_value(admin.permissions.clone()).unwrap_or_default();

        Ok(AdminInfo {
            id: admin.id,
            email: admin.email,
            username: admin.username,
            nickname: admin.nickname,
            avatar_url: admin.avatar_url,
            is_active: admin.is_active,
            role: admin.role,
            permissions,
        })
    }

    /// 创建管理员
    pub async fn create_admin(
        &self,
        email: &str,
        password: &str,
        username: &str,
        role: &str,
        creator_id: Option<i64>,
        ip_address: Option<&str>,
    ) -> Result<Admin, RswsError> {
        if self.admin_repo.email_exists(email).await? {
            return Err(RswsError::conflict("Email already exists"));
        }

        let admin = self
            .admin_repo
            .create_admin(email, password, username, role)
            .await?;

        if let Some(creator_id) = creator_id {
            let _ = self
                .admin_repo
                .log_admin_operation(
                    creator_id,
                    "create",
                    Some("admin"),
                    Some(&admin.id.to_string()),
                    Some(&format!("Created admin: {}", admin.username)),
                    ip_address,
                    None,
                )
                .await;
        }

        Ok(admin)
    }

    /// 更新管理员信息
    /// 注意：调用方需在密码修改或禁用后调用 api_key_manager.invalidate(admin_id) 使旧 session 失效
    pub async fn update_admin(
        &self,
        request: UpdateAdminRequest,
        updater_id: i64,
        ip_address: Option<&str>,
    ) -> Result<Admin, RswsError> {
        let updated = self.admin_repo.update_admin(&request).await?;

        // 注意：原逻辑在密码修改和禁用时会 invalidate_admin_keys，
        // 现已迁移至 handler 层由 api_key_manager.invalidate(admin_id) 处理

        let _ = self
            .admin_repo
            .log_admin_operation(
                updater_id,
                "update",
                Some("admin"),
                Some(&updated.id.to_string()),
                Some(&format!("Updated admin: {}", updated.username)),
                ip_address,
                None,
            )
            .await;

        Ok(updated)
    }

    /// 获取管理员列表
    pub async fn list_admins(
        &self,
        page: i64,
        page_size: i64,
        role: Option<&str>,
    ) -> Result<(Vec<AdminInfo>, i64), RswsError> {
        let admins = self.admin_repo.get_admins(page, page_size, role).await?;
        let total = self.admin_repo.get_admins_count(role).await?;

        let admin_infos = admins
            .into_iter()
            .map(|admin| {
                let permissions: Vec<String> =
                    serde_json::from_value(admin.permissions.clone()).unwrap_or_default();
                AdminInfo {
                    id: admin.id,
                    email: admin.email,
                    username: admin.username,
                    nickname: admin.nickname,
                    avatar_url: admin.avatar_url,
                    is_active: admin.is_active,
                    role: admin.role,
                    permissions,
                }
            })
            .collect();

        Ok((admin_infos, total))
    }

    /// 停用管理员
    /// 注意：调用方需在停用后调用 api_key_manager.invalidate(admin_id) 使旧 session 失效
    pub async fn deactivate_admin(
        &self,
        admin_id: i64,
        operator_id: i64,
        ip_address: Option<&str>,
    ) -> Result<(), RswsError> {
        let request = UpdateAdminRequest {
            id: admin_id,
            email: None,
            password: None,
            username: None,
            avatar_url: None,
            is_active: Some(false),
            role: None,
            permissions: None,
        };
        self.admin_repo.update_admin(&request).await?;

        // 注意：原逻辑会调用 invalidate_admin_keys，
        // 现已迁移至 handler 层由 api_key_manager.invalidate(admin_id) 处理

        let _ = self
            .admin_repo
            .log_admin_operation(
                operator_id,
                "deactivate",
                Some("admin"),
                Some(&admin_id.to_string()),
                None,
                ip_address,
                None,
            )
            .await;

        Ok(())
    }

    /// 激活管理员
    pub async fn activate_admin(
        &self,
        admin_id: i64,
        operator_id: i64,
        ip_address: Option<&str>,
    ) -> Result<(), RswsError> {
        let request = UpdateAdminRequest {
            id: admin_id,
            email: None,
            password: None,
            username: None,
            avatar_url: None,
            is_active: Some(true),
            role: None,
            permissions: None,
        };
        self.admin_repo.update_admin(&request).await?;

        // 记录操作日志
        let _ = self
            .admin_repo
            .log_admin_operation(
                operator_id,
                "activate",
                Some("admin"),
                Some(&admin_id.to_string()),
                None,
                ip_address,
                None,
            )
            .await;

        Ok(())
    }

    /// 重置管理员密码
    /// 注意：调用方需在密码重置后调用 api_key_manager.invalidate(admin_id) 使旧 session 失效
    pub async fn reset_password(
        &self,
        admin_id: i64,
        new_password: &str,
        operator_id: i64,
        ip_address: Option<&str>,
    ) -> Result<(), RswsError> {
        let request = UpdateAdminRequest {
            id: admin_id,
            email: None,
            password: Some(new_password.to_string()),
            username: None,
            avatar_url: None,
            is_active: None,
            role: None,
            permissions: None,
        };

        self.update_admin(request, operator_id, ip_address).await?;

        Ok(())
    }
}
