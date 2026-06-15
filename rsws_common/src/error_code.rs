//! RSWS 核心错误码定义
//!
//! 统一的错误码体系，所有模块共用
//! 格式: XYYZZ
//! - X: 模块 (1=系统, 2=用户, 3=资源, 4=订单, 5=支付, 6=配置, 7=认证)
//! - YY: 子模块
//! - ZZ: 具体错误

use serde::{Deserialize, Serialize};

/// 错误码定义
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorCode(pub i32);

impl ErrorCode {
    // ==================== 系统级错误 (1xxxx) ====================
    pub const SUCCESS: Self = Self(0);
    pub const UNKNOWN: Self = Self(10000);
    pub const BAD_REQUEST: Self = Self(10001);
    pub const UNAUTHORIZED: Self = Self(10002);
    pub const FORBIDDEN: Self = Self(10003);
    pub const NOT_FOUND: Self = Self(10004);
    pub const METHOD_NOT_ALLOWED: Self = Self(10005);
    pub const CONFLICT: Self = Self(10006);
    pub const INTERNAL_ERROR: Self = Self(10007);
    pub const SERVICE_UNAVAILABLE: Self = Self(10008);
    pub const TIMEOUT: Self = Self(10009);
    pub const RATE_LIMIT_EXCEEDED: Self = Self(10010);
    pub const INVALID_PARAMETER: Self = Self(10011);
    pub const INVALID_REQUEST_FORMAT: Self = Self(10012);

    // ==================== 认证错误 (2xxxx) ====================
    pub const AUTH_INVALID_CREDENTIALS: Self = Self(20001);
    pub const AUTH_TOKEN_EXPIRED: Self = Self(20002);
    pub const AUTH_TOKEN_INVALID: Self = Self(20003);
    pub const AUTH_SIGNATURE_INVALID: Self = Self(20004);
    pub const AUTH_SIGNATURE_EXPIRED: Self = Self(20005);
    pub const AUTH_API_KEY_NOT_FOUND: Self = Self(20006);
    pub const AUTH_API_KEY_DISABLED: Self = Self(20007);
    pub const AUTH_SESSION_EXPIRED: Self = Self(20008);
    pub const AUTH_PASSWORD_MISMATCH: Self = Self(20009);
    pub const AUTH_PASSWORD_TOO_WEAK: Self = Self(20010);
    pub const AUTH_ACCOUNT_DISABLED: Self = Self(20011);
    pub const AUTH_ACCOUNT_LOCKED: Self = Self(20012);
    pub const AUTH_2FA_REQUIRED: Self = Self(20013);
    pub const AUTH_2FA_INVALID: Self = Self(20014);
    pub const AUTH_INVALID_API_KEY: Self = Self(20015);
    pub const AUTH_API_KEY_EXPIRED: Self = Self(20016);
    pub const AUTH_INVALID_SIGNATURE: Self = Self(20017);
    pub const AUTH_TIMESTAMP_EXPIRED: Self = Self(20018);
    pub const AUTH_MISSING_CREDENTIALS: Self = Self(20019);
    pub const AUTH_PERMISSION_DENIED: Self = Self(20020);

    // ==================== 用户错误 (3xxxx) ====================
    pub const USER_NOT_FOUND: Self = Self(30001);
    pub const USER_ALREADY_EXISTS: Self = Self(30002);
    pub const USER_EMAIL_EXISTS: Self = Self(30003);
    pub const USER_USERNAME_EXISTS: Self = Self(30004);
    pub const USER_EMAIL_INVALID: Self = Self(30005);
    pub const USER_PASSWORD_INVALID: Self = Self(30006);
    pub const USER_AVATAR_TOO_LARGE: Self = Self(30007);
    pub const USER_AVATAR_INVALID_TYPE: Self = Self(30008);
    pub const USER_PROFILE_INCOMPLETE: Self = Self(30009);
    pub const USER_DISABLED: Self = Self(30010);

    // ==================== 资源错误 (4xxxx) ====================
    pub const RESOURCE_NOT_FOUND: Self = Self(40001);
    pub const RESOURCE_ALREADY_EXISTS: Self = Self(40002);
    pub const RESOURCE_NOT_ACTIVE: Self = Self(40003);
    pub const RESOURCE_FILE_TOO_LARGE: Self = Self(40004);
    pub const RESOURCE_FILE_INVALID_TYPE: Self = Self(40005);
    pub const RESOURCE_UPLOAD_FAILED: Self = Self(40006);
    pub const RESOURCE_DELETE_FAILED: Self = Self(40007);
    pub const RESOURCE_NOT_OWNER: Self = Self(40008);
    pub const RESOURCE_PENDING_REVIEW: Self = Self(40009);
    pub const RESOURCE_REJECTED: Self = Self(40010);

    // ==================== 订单错误 (5xxxx) ====================
    pub const ORDER_NOT_FOUND: Self = Self(50001);
    pub const ORDER_ALREADY_EXISTS: Self = Self(50002);
    pub const ORDER_ALREADY_PAID: Self = Self(50003);
    pub const ORDER_ALREADY_CANCELLED: Self = Self(50004);
    pub const ORDER_EXPIRED: Self = Self(50005);
    pub const ORDER_STATUS_INVALID: Self = Self(50006);
    pub const ORDER_AMOUNT_MISMATCH: Self = Self(50007);
    pub const ORDER_REFUND_FAILED: Self = Self(50008);

    // ==================== 支付错误 (6xxxx) ====================
    pub const PAYMENT_METHOD_NOT_SUPPORTED: Self = Self(60001);
    pub const PAYMENT_AMOUNT_INVALID: Self = Self(60002);
    pub const PAYMENT_AMOUNT_TOO_SMALL: Self = Self(60003);
    pub const PAYMENT_AMOUNT_TOO_LARGE: Self = Self(60004);
    pub const PAYMENT_TRANSACTION_NOT_FOUND: Self = Self(60005);
    pub const PAYMENT_TRANSACTION_FAILED: Self = Self(60006);
    pub const PAYMENT_GATEWAY_ERROR: Self = Self(60007);

    // PayPal 特定错误 (601xx)
    pub const PAYPAL_ORDER_NOT_FOUND: Self = Self(60101);
    pub const PAYPAL_ORDER_NOT_APPROVED: Self = Self(60102);
    pub const PAYPAL_CAPTURE_FAILED: Self = Self(60103);
    pub const PAYPAL_REFUND_FAILED: Self = Self(60104);
    pub const PAYPAL_WEBHOOK_INVALID: Self = Self(60105);

    // USDT 特定错误 (602xx)
    pub const USDT_ADDRESS_INVALID: Self = Self(60201);
    pub const USDT_TRANSACTION_NOT_FOUND: Self = Self(60202);
    pub const USDT_TRANSACTION_NOT_CONFIRMED: Self = Self(60203);
    pub const USDT_AMOUNT_MISMATCH: Self = Self(60204);
    pub const USDT_NETWORK_ERROR: Self = Self(60205);
    pub const USDT_WALLET_NOT_FOUND: Self = Self(60206);

    // ==================== 配置错误 (7xxxx) ====================
    pub const CONFIG_NOT_FOUND: Self = Self(70001);
    pub const CONFIG_INVALID_VALUE: Self = Self(70002);
    pub const CONFIG_ENCRYPTION_FAILED: Self = Self(70003);
    pub const CONFIG_DECRYPTION_FAILED: Self = Self(70004);

    // ==================== 数据库错误 (8xxxx) ====================
    pub const DB_CONNECTION_FAILED: Self = Self(80001);
    pub const DB_QUERY_FAILED: Self = Self(80002);
    pub const DB_TRANSACTION_FAILED: Self = Self(80003);
    pub const DB_UNIQUE_VIOLATION: Self = Self(80004);
    pub const DB_FOREIGN_KEY_VIOLATION: Self = Self(80005);

    // ==================== 缓存错误 (9xxxx) ====================
    pub const CACHE_CONNECTION_FAILED: Self = Self(90001);
    pub const CACHE_GET_FAILED: Self = Self(90002);
    pub const CACHE_SET_FAILED: Self = Self(90003);

    /// 获取错误消息
    pub fn message(&self) -> &'static str {
        match self.0 {
            // 系统
            0 => "Success",
            10000 => "Unknown error",
            10001 => "Bad request",
            10002 => "Unauthorized",
            10003 => "Forbidden",
            10004 => "Not found",
            10005 => "Method not allowed",
            10006 => "Conflict",
            10007 => "Internal server error",
            10008 => "Service unavailable",
            10009 => "Request timeout",
            10010 => "Rate limit exceeded",
            10011 => "Invalid parameter",
            10012 => "Invalid request format",

            // 认证
            20001 => "Invalid credentials",
            20002 => "Token expired",
            20003 => "Invalid token",
            20004 => "Invalid signature",
            20005 => "Signature expired",
            20006 => "API key not found",
            20007 => "API key disabled",
            20008 => "Session expired",
            20009 => "Password mismatch",
            20010 => "Password too weak",
            20011 => "Account disabled",
            20012 => "Account locked",
            20013 => "2FA required",
            20014 => "Invalid 2FA code",
            20015 => "Invalid API key",
            20016 => "API key expired",
            20017 => "Invalid signature",
            20018 => "Timestamp expired",
            20019 => "Missing credentials",
            20020 => "Permission denied",

            // 用户
            30001 => "User not found",
            30002 => "User already exists",
            30003 => "Email already exists",
            30004 => "Username already exists",
            30005 => "Invalid email format",
            30006 => "Invalid password format",
            30007 => "Avatar file too large",
            30008 => "Invalid avatar file type",
            30009 => "Profile incomplete",
            30010 => "User disabled",

            // 资源
            40001 => "Resource not found",
            40002 => "Resource already exists",
            40003 => "Resource not active",
            40004 => "File too large",
            40005 => "Invalid file type",
            40006 => "Upload failed",
            40007 => "Delete failed",
            40008 => "Not resource owner",
            40009 => "Resource pending review",
            40010 => "Resource rejected",

            // 订单
            50001 => "Order not found",
            50002 => "Order already exists",
            50003 => "Order already paid",
            50004 => "Order already cancelled",
            50005 => "Order expired",
            50006 => "Invalid order status",
            50007 => "Amount mismatch",
            50008 => "Refund failed",

            // 支付
            60001 => "Payment method not supported",
            60002 => "Invalid payment amount",
            60003 => "Amount too small",
            60004 => "Amount too large",
            60005 => "Transaction not found",
            60006 => "Transaction failed",
            60007 => "Payment gateway error",

            // PayPal
            60101 => "PayPal order not found",
            60102 => "PayPal order not approved",
            60103 => "PayPal capture failed",
            60104 => "PayPal refund failed",
            60105 => "Invalid PayPal webhook",

            // USDT
            60201 => "Invalid USDT address",
            60202 => "USDT transaction not found",
            60203 => "USDT transaction not confirmed",
            60204 => "USDT amount mismatch",
            60205 => "USDT network error",
            60206 => "USDT wallet not found",

            // 配置
            70001 => "Config not found",
            70002 => "Invalid config value",
            70003 => "Encryption failed",
            70004 => "Decryption failed",

            // 数据库
            80001 => "Database connection failed",
            80002 => "Database query failed",
            80003 => "Database transaction failed",
            80004 => "Unique constraint violation",
            80005 => "Foreign key violation",

            // 缓存
            90001 => "Cache connection failed",
            90002 => "Cache get failed",
            90003 => "Cache set failed",

            _ => "Unknown error",
        }
    }

    /// 转换为 HTTP 状态码
    pub fn http_status(&self) -> u16 {
        match self.0 / 10000 {
            0 => 200, // 成功
            1 => match self.0 {
                // 系统错误
                10001 => 400,
                10002 => 401,
                10003 => 403,
                10004 => 404,
                10005 => 405,
                10006 => 409,
                10010 => 429,
                _ => 500,
            },
            2 => 401, // 认证错误
            3 => 400, // 用户错误
            4 => 404, // 资源错误
            5 => 400, // 订单错误
            6 => 400, // 支付错误
            7 => 500, // 配置错误
            8 => 500, // 数据库错误
            9 => 500, // 缓存错误
            _ => 500,
        }
    }
}

impl From<i32> for ErrorCode {
    fn from(code: i32) -> Self {
        Self(code)
    }
}

impl From<salvo::http::StatusCode> for ErrorCode {
    fn from(status: salvo::http::StatusCode) -> Self {
        match status.as_u16() {
            200 => Self::SUCCESS,
            400 => Self::BAD_REQUEST,
            401 => Self::UNAUTHORIZED,
            403 => Self::FORBIDDEN,
            404 => Self::NOT_FOUND,
            405 => Self::METHOD_NOT_ALLOWED,
            409 => Self::CONFLICT,
            429 => Self::RATE_LIMIT_EXCEEDED,
            500 => Self::INTERNAL_ERROR,
            503 => Self::SERVICE_UNAVAILABLE,
            _ => Self::UNKNOWN,
        }
    }
}

impl ErrorCode {
    /// 从 HTTP 状态码创建 ErrorCode
    pub fn from_status(status: salvo::http::StatusCode) -> Self {
        Self::from(status)
    }
}

impl From<ErrorCode> for i32 {
    fn from(code: ErrorCode) -> Self {
        code.0
    }
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
