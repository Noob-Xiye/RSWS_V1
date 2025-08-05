use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AdminRole {
    #[serde(rename = "operator")]
    Operator, // 普通操作员
    #[serde(rename = "supervisor")]
    Supervisor, // 主管
    #[serde(rename = "admin")]
    Admin, // 管理员
    #[serde(rename = "super_admin")]
    SuperAdmin, // 超级管理员
}

impl AdminRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            AdminRole::Operator => "operator",
            AdminRole::Supervisor => "supervisor",
            AdminRole::Admin => "admin",
            AdminRole::SuperAdmin => "super_admin",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "operator" => Some(AdminRole::Operator),
            "supervisor" => Some(AdminRole::Supervisor),
            "admin" => Some(AdminRole::Admin),
            "super_admin" => Some(AdminRole::SuperAdmin),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AdminPermission {
    #[serde(rename = "user_view")]
    UserView,
    #[serde(rename = "user_manage")]
    UserManage,
    #[serde(rename = "resource_view")]
    ResourceView,
    #[serde(rename = "resource_manage")]
    ResourceManage,
    #[serde(rename = "order_view")]
    OrderView,
    #[serde(rename = "order_manage")]
    OrderManage,
    #[serde(rename = "payment_view")]
    PaymentView,
    #[serde(rename = "payment_manage")]
    PaymentManage,
    #[serde(rename = "system_config")]
    SystemConfig,
    #[serde(rename = "admin_manage")]
    AdminManage,
    #[serde(rename = "report_view")]
    ReportView,
    #[serde(rename = "log_view")]
    LogView,
}

impl AdminPermission {
    pub fn as_str(&self) -> &'static str {
        match self {
            AdminPermission::UserView => "user_view",
            AdminPermission::UserManage => "user_manage",
            AdminPermission::ResourceView => "resource_view",
            AdminPermission::ResourceManage => "resource_manage",
            AdminPermission::OrderView => "order_view",
            AdminPermission::OrderManage => "order_manage",
            AdminPermission::PaymentView => "payment_view",
            AdminPermission::PaymentManage => "payment_manage",
            AdminPermission::SystemConfig => "system_config",
            AdminPermission::AdminManage => "admin_manage",
            AdminPermission::ReportView => "report_view",
            AdminPermission::LogView => "log_view",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "user_view" => Some(AdminPermission::UserView),
            "user_manage" => Some(AdminPermission::UserManage),
            "resource_view" => Some(AdminPermission::ResourceView),
            "resource_manage" => Some(AdminPermission::ResourceManage),
            "order_view" => Some(AdminPermission::OrderView),
            "order_manage" => Some(AdminPermission::OrderManage),
            "payment_view" => Some(AdminPermission::PaymentView),
            "payment_manage" => Some(AdminPermission::PaymentManage),
            "system_config" => Some(AdminPermission::SystemConfig),
            "admin_manage" => Some(AdminPermission::AdminManage),
            "report_view" => Some(AdminPermission::ReportView),
            "log_view" => Some(AdminPermission::LogView),
            _ => None,
        }
    }
}
