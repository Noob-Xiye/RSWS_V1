//! 共享模块
//!
//! 两边都需要使用的功能（webhook、upload、支付回调等），
//! 不属于 admin 也不属于 custom，独立存放避免重复实现。

mod payment;
mod upload;
mod webhook;

// upload.rs
pub use upload::complete_upload;
pub use upload::init_upload;
pub use upload::upload_chunk;
pub use upload::upload_single;

// webhook.rs
pub use webhook::paypal_webhook;
pub use webhook::usdt_webhook;

// payment.rs
pub use payment::get_usdt_address;
pub use payment::paypal_cancel;
pub use payment::paypal_success;
