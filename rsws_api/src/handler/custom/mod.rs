//! 用户端模块
//!
//! 用户端 handler 按功能域拆分为子模块，统一在此 re-export，
//! 使 `handler::custom::*` 路由引用路径保持不变。

mod category;
mod order;
mod resource;
mod user;

// category.rs
pub use category::list_categories;

// order.rs
pub use order::cancel_order;
pub use order::check_order_status;
pub use order::check_purchase;
pub use order::complete_order;
pub use order::create_order;
pub use order::get_order;
pub use order::get_resource_download;
pub use order::initiate_payment;
pub use order::list_orders;
pub use order::refund_order;

// resource.rs
pub use resource::create_resource;
pub use resource::delete_resource;
pub use resource::get_resource;
pub use resource::list_resources;
pub use resource::update_resource;

// user.rs
pub use user::change_password;
pub use user::get_current_user;
pub use user::get_user;
pub use user::login;
pub use user::register;
pub use user::send_code;
pub use user::update_profile;
pub use user::upload_avatar;
