//! 管理员模块
//!
//! 按功能域拆分为子模块，统一在此 re-export，
//! 使 `handler::admin::*` 路由引用路径保持不变。

// 子模块声明
mod api_key;
mod audit_log;
mod auth;
mod category;
mod dashboard;
mod email;
mod error_log;
mod log;
mod login_log;
mod management;
mod order;
mod oss;
mod payment_method;
mod paypal;
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

// audit_log.rs
pub use audit_log::get_audit_stats;
pub use audit_log::get_resource_history;
pub use audit_log::list_audit_logs;

// error_log.rs
pub use error_log::get_error_log;
pub use error_log::get_error_stats;
pub use error_log::list_error_logs;
pub use error_log::resolve_error;

// login_log.rs
pub use login_log::get_login_stats;
pub use login_log::list_login_logs;

// wallet.rs
pub use wallet::list_usdt_wallets;
pub use wallet::update_usdt_wallet;

// dashboard.rs
pub use dashboard::dashboard_stats;
pub use dashboard::get_log_stats;
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

// oss.rs
pub use oss::get_storage_config;
pub use oss::test_storage_connection;
pub use oss::update_storage_config;

// paypal.rs
pub use paypal::create_paypal_config;
pub use paypal::get_paypal_config;
pub use paypal::list_paypal_configs;
pub use paypal::set_paypal_config_active;
pub use paypal::update_paypal_config;

// category.rs
pub use category::admin_list_categories;
pub use category::batch_update_sort;
pub use category::create_category;
pub use category::delete_category;
pub use category::update_category;

// order.rs
pub use order::admin_list_orders;
