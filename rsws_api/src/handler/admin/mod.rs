//! 管理员模块
//!
//! 按功能域拆分为子模块，统一在此 re-export，
//! 使 `handler::admin::*` 路由引用路径保持不变。

// 子模块声明
mod api_key;
mod auth;
mod dashboard;
mod email;
mod log;
mod management;
mod payment_method;
mod resource;
mod user;
mod wallet;

// ---- re-export：保持路由引用兼容 ----
// auth.rs
pub use auth::get_current_admin;
pub use auth::login;

// management.rs
pub use management::activate_admin;
pub use management::create_admin;
pub use management::deactivate_admin;
pub use management::get_admin;
pub use management::list_admins;
pub use management::reset_admin_password;

// api_key.rs
pub use api_key::create_api_key;
pub use api_key::delete_api_key;
pub use api_key::list_api_keys;
pub use api_key::toggle_api_key_status;

// log.rs
pub use log::create_log_config;
pub use log::delete_log_config;
pub use log::get_log_config;
pub use log::list_log_configs;
pub use log::query_system_logs;
pub use log::update_log_config;

// wallet.rs
pub use wallet::list_usdt_wallets;
pub use wallet::update_usdt_wallet;

// dashboard.rs
pub use dashboard::dashboard_stats;
pub use dashboard::revenue_chart;

// user.rs
pub use user::activate_user;
pub use user::deactivate_user;
pub use user::list_users;

// resource.rs
pub use resource::create_platform_resource;
pub use resource::delete_platform_resource;
pub use resource::list_resources;
pub use resource::toggle_platform_resource;
pub use resource::update_platform_resource;

// email.rs
pub use email::get_email_config;
pub use email::update_email_config;

// payment_method.rs
pub use payment_method::create_payment_method;
pub use payment_method::delete_payment_method;
pub use payment_method::list_payment_methods;
