-- ========================================
-- RSWS 初始数据库架构
-- 迁移版本: 20260506185039
-- 
-- 包含所有表结构、索引、枚举类型和初始数据
-- 适用于 PostgreSQL 12+
-- ========================================

-- ========================================
-- 第一部分: 枚举类型定义
-- ========================================

CREATE TYPE order_status AS ENUM (
    'pending',
    'paid', 
    'completed',
    'cancelled',
    'refunded',
    'failed'
);

CREATE TYPE transaction_status AS ENUM (
    'pending',
    'completed',
    'failed',
    'cancelled'
);

-- ========================================
-- 第二部分: 核心业务表
-- ========================================

CREATE TABLE users (
    id BIGSERIAL PRIMARY KEY,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255),
    username VARCHAR(100),
    avatar_url VARCHAR(500),
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE TABLE admins (
    id BIGSERIAL PRIMARY KEY,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    username VARCHAR(100) NOT NULL,
    avatar_url VARCHAR(500),
    is_active BOOLEAN DEFAULT true,
    role VARCHAR(50) NOT NULL DEFAULT 'operator',
    permissions JSONB DEFAULT '[]',
    last_login_at TIMESTAMP WITH TIME ZONE,
    last_login_ip VARCHAR(50),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE TABLE resources (
    id BIGSERIAL PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    price DECIMAL(10,2) NOT NULL,
    category_id BIGINT,
    file_url VARCHAR(500),
    thumbnail_url VARCHAR(500),
    is_active BOOLEAN DEFAULT true,
    detail_description TEXT,
    specifications JSONB,
    usage_guide TEXT,
    precautions TEXT,
    display_images TEXT[],
    provider_type VARCHAR(20) NOT NULL DEFAULT 'admin',
    provider_id BIGINT,
    commission_rate DECIMAL(5,4) DEFAULT 0.0000,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE TABLE user_payment_configs (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    config_name VARCHAR(100) NOT NULL,
    payment_method VARCHAR(50) NOT NULL,
    paypal_email VARCHAR(255),
    paypal_merchant_id VARCHAR(100),
    usdt_address VARCHAR(100),
    usdt_network VARCHAR(20),
    is_active BOOLEAN DEFAULT true,
    is_default BOOLEAN DEFAULT false,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(user_id, config_name)
);

CREATE TABLE resource_payment_configs (
    id BIGSERIAL PRIMARY KEY,
    resource_id BIGINT NOT NULL REFERENCES resources(id) ON DELETE CASCADE,
    payment_config_id BIGINT NOT NULL REFERENCES user_payment_configs(id) ON DELETE CASCADE,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(resource_id, payment_config_id)
);

CREATE TABLE orders (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id),
    resource_id BIGINT NOT NULL REFERENCES resources(id),
    amount DECIMAL(10,2) NOT NULL,
    status order_status NOT NULL DEFAULT 'pending',
    payment_method VARCHAR(50),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    expired_at TIMESTAMP WITH TIME ZONE,
    referrer_id BIGINT REFERENCES users(id),
    transaction_id VARCHAR(50),
    UNIQUE(user_id, resource_id)
);

CREATE TABLE payment_transactions (
    id VARCHAR(50) PRIMARY KEY,
    order_id BIGINT NOT NULL REFERENCES orders(id),
    user_id BIGINT NOT NULL REFERENCES users(id),
    amount DECIMAL(10,2) NOT NULL,
    currency VARCHAR(10) DEFAULT 'USD',
    payment_method VARCHAR(50) NOT NULL,
    provider_transaction_id VARCHAR(255),
    status transaction_status NOT NULL DEFAULT 'pending',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    completed_at TIMESTAMP WITH TIME ZONE
);

-- ========================================
-- 第三部分: 用户会话表 (API Key 认证)
-- ========================================

CREATE TABLE user_sessions (
    id BIGINT PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    session_token VARCHAR(255) NOT NULL UNIQUE,
    api_key VARCHAR(64) NOT NULL UNIQUE,
    device_info JSONB,
    ip_address INET,
    user_agent TEXT,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    last_activity TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- ========================================
-- 第四部分: 配置管理表
-- ========================================

CREATE TABLE system_configs (
    id SERIAL PRIMARY KEY,
    config_key VARCHAR(100) NOT NULL UNIQUE,
    config_value TEXT NOT NULL,
    config_type VARCHAR(20) NOT NULL DEFAULT 'string',
    description TEXT,
    is_encrypted BOOLEAN DEFAULT false,
    is_public BOOLEAN DEFAULT false,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE TABLE paypal_configs (
    id SERIAL PRIMARY KEY,
    client_id VARCHAR(255) NOT NULL,
    client_secret_encrypted TEXT NOT NULL,
    sandbox BOOLEAN DEFAULT true,
    webhook_id VARCHAR(255),
    webhook_secret_encrypted TEXT,
    base_url VARCHAR(500) NOT NULL,
    return_url VARCHAR(500) NOT NULL,
    cancel_url VARCHAR(500) NOT NULL,
    brand_name VARCHAR(100) NOT NULL,
    min_amount DECIMAL(10,2) NOT NULL DEFAULT 0.01,
    max_amount DECIMAL(10,2) NOT NULL DEFAULT 10000.00,
    fee_rate DECIMAL(5,4) NOT NULL DEFAULT 0.0349,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE TABLE blockchain_configs (
    id SERIAL PRIMARY KEY,
    network VARCHAR(50) NOT NULL UNIQUE,
    network_name VARCHAR(100) NOT NULL,
    api_url VARCHAR(500) NOT NULL,
    api_key_encrypted TEXT,
    usdt_contract VARCHAR(100) NOT NULL,
    wallet_addresses JSONB NOT NULL DEFAULT '[]',
    min_confirmations INTEGER NOT NULL DEFAULT 1,
    min_amount DECIMAL(10,2) NOT NULL DEFAULT 1.00,
    max_amount DECIMAL(10,2) NOT NULL DEFAULT 50000.00,
    fee_rate DECIMAL(5,4) NOT NULL DEFAULT 0.0000,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE TABLE payment_method_configs (
    id SERIAL PRIMARY KEY,
    method_id VARCHAR(50) NOT NULL UNIQUE,
    method_name VARCHAR(100) NOT NULL,
    icon_url VARCHAR(500),
    description TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0,
    is_active BOOLEAN DEFAULT true,
    config_json JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE TABLE encryption_configs (
    id SERIAL PRIMARY KEY,
    config_name VARCHAR(100) NOT NULL UNIQUE,
    encryption_key_encrypted TEXT NOT NULL,
    algorithm VARCHAR(50) NOT NULL DEFAULT 'AES-256-GCM',
    key_version INTEGER NOT NULL DEFAULT 1,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE TABLE email_configs (
    id SERIAL PRIMARY KEY,
    provider VARCHAR(50) NOT NULL DEFAULT 'smtp',
    host VARCHAR(255),
    port INTEGER,
    username VARCHAR(255),
    password_encrypted TEXT,
    use_tls BOOLEAN DEFAULT true,
    from_email VARCHAR(255) NOT NULL,
    from_name VARCHAR(100),
    reply_to VARCHAR(255),
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- ========================================
-- 第五部分: USDT 监听服务表
-- ========================================

CREATE TABLE usdt_transactions (
    id BIGINT PRIMARY KEY,
    tx_hash VARCHAR(66) NOT NULL UNIQUE,
    network VARCHAR(20) NOT NULL,
    from_address VARCHAR(100) NOT NULL,
    to_address VARCHAR(100) NOT NULL,
    amount DECIMAL(20, 6) NOT NULL,
    block_number BIGINT,
    confirmations INTEGER DEFAULT 0,
    status VARCHAR(20) DEFAULT 'pending',
    order_id BIGINT REFERENCES orders(id),
    processed_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE TABLE usdt_wallets (
    id BIGINT PRIMARY KEY,
    address VARCHAR(100) NOT NULL UNIQUE,
    network VARCHAR(20) NOT NULL,
    name VARCHAR(100),
    is_active BOOLEAN DEFAULT true,
    total_received DECIMAL(20, 6) DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE TABLE usdt_listen_configs (
    id SERIAL PRIMARY KEY,
    network VARCHAR(20) NOT NULL UNIQUE,
    api_url VARCHAR(255) NOT NULL,
    api_key_encrypted TEXT,
    usdt_contract VARCHAR(100) NOT NULL,
    poll_interval_seconds INTEGER DEFAULT 10,
    min_confirmations INTEGER DEFAULT 3,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- ========================================
-- 第六部分: API 管理表
-- ========================================

CREATE TABLE admin_api_keys (
    id SERIAL PRIMARY KEY,
    admin_id INTEGER NOT NULL REFERENCES admins(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    api_key VARCHAR(64) NOT NULL UNIQUE,
    permissions JSONB NOT NULL DEFAULT '[]',
    rate_limit INTEGER,
    last_used_at TIMESTAMP WITH TIME ZONE,
    expires_at TIMESTAMP WITH TIME ZONE,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE TABLE domains (
    id SERIAL PRIMARY KEY,
    domain VARCHAR(255) NOT NULL UNIQUE,
    cert_pem TEXT NOT NULL,
    key_pem TEXT NOT NULL,
    is_active BOOLEAN DEFAULT true,
    expires_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- ========================================
-- 第七部分: 日志系统表
-- ========================================

CREATE TABLE admin_operation_logs (
    id BIGSERIAL PRIMARY KEY,
    admin_id BIGINT NOT NULL REFERENCES admins(id),
    operation_type VARCHAR(50) NOT NULL,
    operation_target VARCHAR(50),
    target_id VARCHAR(100),
    operation_content TEXT,
    ip_address VARCHAR(50),
    user_agent TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE TABLE system_logs (
    id BIGINT PRIMARY KEY,
    log_level VARCHAR(20) NOT NULL,
    module VARCHAR(100) NOT NULL,
    message TEXT NOT NULL,
    context JSONB,
    user_id BIGINT,
    admin_id BIGINT,
    ip_address VARCHAR(50),
    user_agent TEXT,
    request_id VARCHAR(100),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE TABLE error_logs (
    id BIGINT PRIMARY KEY,
    error_type VARCHAR(100) NOT NULL,
    error_message TEXT NOT NULL,
    stack_trace TEXT,
    module VARCHAR(100) NOT NULL,
    function_name VARCHAR(200),
    user_id BIGINT,
    admin_id BIGINT,
    request_id VARCHAR(100),
    context JSONB,
    ip_address VARCHAR(50),
    user_agent TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE TABLE payment_logs (
    id BIGINT PRIMARY KEY,
    transaction_id VARCHAR(50),
    order_id BIGINT,
    user_id BIGINT NOT NULL,
    payment_method VARCHAR(50) NOT NULL,
    amount DECIMAL(10,2) NOT NULL,
    currency VARCHAR(10) DEFAULT 'USD',
    status VARCHAR(50) NOT NULL,
    provider_response JSONB,
    gateway_transaction_id VARCHAR(255),
    ip_address VARCHAR(50),
    user_agent TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE TABLE webhook_logs (
    id BIGINT PRIMARY KEY,
    webhook_type VARCHAR(100) NOT NULL,
    source VARCHAR(100) NOT NULL,
    event_type VARCHAR(100) NOT NULL,
    payload JSONB NOT NULL,
    headers JSONB,
    signature VARCHAR(500),
    status VARCHAR(50) NOT NULL,
    response_code INTEGER,
    response_message TEXT,
    processed_at TIMESTAMP WITH TIME ZONE,
    retry_count INTEGER DEFAULT 0,
    ip_address VARCHAR(50),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE TABLE request_logs (
    id BIGINT PRIMARY KEY,
    request_id VARCHAR(100) NOT NULL,
    method VARCHAR(10) NOT NULL,
    path VARCHAR(500) NOT NULL,
    query_params JSONB,
    headers JSONB,
    body_size INTEGER,
    user_id BIGINT,
    admin_id BIGINT,
    ip_address VARCHAR(50),
    user_agent TEXT,
    response_status INTEGER,
    response_size INTEGER,
    duration_ms INTEGER,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- ========================================
-- 第八部分: 佣金系统表
-- ========================================

CREATE TABLE commission_rules (
    id BIGINT PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    rule_type VARCHAR(20) NOT NULL,
    rate DECIMAL(10,4) NOT NULL,
    min_amount DECIMAL(10,2),
    max_amount DECIMAL(10,2),
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE TABLE commission_records (
    id BIGINT PRIMARY KEY,
    order_id BIGINT NOT NULL REFERENCES orders(id),
    user_id BIGINT NOT NULL REFERENCES users(id),
    referrer_id BIGINT REFERENCES users(id),
    rule_id BIGINT NOT NULL REFERENCES commission_rules(id),
    order_amount DECIMAL(10,2) NOT NULL,
    commission_amount DECIMAL(10,2) NOT NULL,
    commission_rate DECIMAL(10,4) NOT NULL,
    status VARCHAR(20) DEFAULT 'pending',
    paid_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- ========================================
-- 第九部分: 跨平台集成表
-- ========================================

CREATE TABLE cross_platform_configs (
    id BIGINT PRIMARY KEY,
    platform_name VARCHAR(50) NOT NULL UNIQUE,
    api_endpoint VARCHAR(255) NOT NULL,
    api_key VARCHAR(255) NOT NULL,
    webhook_secret VARCHAR(255),
    is_active BOOLEAN DEFAULT true,
    sync_interval INTEGER DEFAULT 300,
    last_sync_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE TABLE cross_platform_orders (
    id BIGINT PRIMARY KEY,
    platform_name VARCHAR(50) NOT NULL,
    platform_order_id VARCHAR(100) NOT NULL,
    customer_email VARCHAR(255),
    customer_name VARCHAR(100),
    total_amount DECIMAL(10,2) NOT NULL,
    currency VARCHAR(10) DEFAULT 'USD',
    status VARCHAR(50) NOT NULL,
    order_data JSONB NOT NULL,
    synced_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    platform_created_at TIMESTAMP WITH TIME ZONE,
    UNIQUE(platform_name, platform_order_id)
);

-- 日志配置表
CREATE TABLE log_configs (
    id SERIAL PRIMARY KEY,
    config_key VARCHAR(100) NOT NULL UNIQUE,
    config_value TEXT NOT NULL,
    config_type VARCHAR(20) NOT NULL DEFAULT 'string',
    description TEXT,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- ========================================
-- 第十部分: 索引创建
-- ========================================

CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_is_active ON users(is_active);

CREATE INDEX idx_admins_email ON admins(email);
CREATE INDEX idx_admins_role ON admins(role);
CREATE INDEX idx_admins_is_active ON admins(is_active);

CREATE INDEX idx_resources_provider_type ON resources(provider_type);
CREATE INDEX idx_resources_provider_id ON resources(provider_id);
CREATE INDEX idx_resources_category_id ON resources(category_id);
CREATE INDEX idx_resources_is_active ON resources(is_active);
CREATE INDEX idx_resources_specifications ON resources USING GIN (specifications);

CREATE INDEX idx_user_payment_configs_user_id ON user_payment_configs(user_id);
CREATE INDEX idx_user_payment_configs_payment_method ON user_payment_configs(payment_method);
CREATE INDEX idx_user_payment_configs_is_active ON user_payment_configs(is_active);
CREATE INDEX idx_user_payment_configs_is_default ON user_payment_configs(is_default);

CREATE INDEX idx_resource_payment_configs_resource_id ON resource_payment_configs(resource_id);
CREATE INDEX idx_resource_payment_configs_payment_config_id ON resource_payment_configs(payment_config_id);

CREATE INDEX idx_orders_user_id ON orders(user_id);
CREATE INDEX idx_orders_status ON orders(status);
CREATE INDEX idx_orders_created_at ON orders(created_at);

CREATE INDEX idx_payment_transactions_order_id ON payment_transactions(order_id);
CREATE INDEX idx_payment_transactions_user_id ON payment_transactions(user_id);
CREATE INDEX idx_payment_transactions_status ON payment_transactions(status);

CREATE INDEX idx_user_sessions_user_id ON user_sessions(user_id);
CREATE INDEX idx_user_sessions_session_token ON user_sessions(session_token);
CREATE INDEX idx_user_sessions_api_key ON user_sessions(api_key);
CREATE INDEX idx_user_sessions_expires_at ON user_sessions(expires_at);
CREATE INDEX idx_user_sessions_active ON user_sessions(is_active);

CREATE INDEX idx_system_configs_key ON system_configs(config_key);
CREATE INDEX idx_system_configs_public ON system_configs(is_public);

CREATE INDEX idx_blockchain_configs_network ON blockchain_configs(network);
CREATE INDEX idx_blockchain_configs_is_active ON blockchain_configs(is_active);

CREATE INDEX idx_payment_method_configs_method_id ON payment_method_configs(method_id);
CREATE INDEX idx_payment_method_configs_is_active ON payment_method_configs(is_active);
CREATE INDEX idx_payment_method_configs_sort_order ON payment_method_configs(sort_order);

CREATE INDEX idx_usdt_tx_hash ON usdt_transactions(tx_hash);
CREATE INDEX idx_usdt_tx_network ON usdt_transactions(network);
CREATE INDEX idx_usdt_tx_to_address ON usdt_transactions(to_address);
CREATE INDEX idx_usdt_tx_status ON usdt_transactions(status);
CREATE INDEX idx_usdt_tx_order_id ON usdt_transactions(order_id);
CREATE INDEX idx_usdt_tx_created_at ON usdt_transactions(created_at);

CREATE INDEX idx_usdt_wallet_address ON usdt_wallets(address);
CREATE INDEX idx_usdt_wallet_network ON usdt_wallets(network);
CREATE INDEX idx_usdt_wallet_active ON usdt_wallets(is_active);

CREATE INDEX idx_admin_api_keys_admin_id ON admin_api_keys(admin_id);
CREATE INDEX idx_admin_api_keys_api_key ON admin_api_keys(api_key);
CREATE INDEX idx_admin_api_keys_active ON admin_api_keys(is_active);

CREATE INDEX idx_domains_domain ON domains(domain);
CREATE INDEX idx_domains_active ON domains(is_active);

CREATE INDEX idx_admin_operation_logs_admin_id ON admin_operation_logs(admin_id);
CREATE INDEX idx_admin_operation_logs_operation_type ON admin_operation_logs(operation_type);
CREATE INDEX idx_admin_operation_logs_created_at ON admin_operation_logs(created_at);

CREATE INDEX idx_system_logs_level ON system_logs(log_level);
CREATE INDEX idx_system_logs_module ON system_logs(module);
CREATE INDEX idx_system_logs_created_at ON system_logs(created_at);
CREATE INDEX idx_system_logs_user_id ON system_logs(user_id);
CREATE INDEX idx_system_logs_request_id ON system_logs(request_id);

CREATE INDEX idx_error_logs_error_type ON error_logs(error_type);
CREATE INDEX idx_error_logs_module ON error_logs(module);
CREATE INDEX idx_error_logs_created_at ON error_logs(created_at);
CREATE INDEX idx_error_logs_request_id ON error_logs(request_id);

CREATE INDEX idx_payment_logs_transaction_id ON payment_logs(transaction_id);
CREATE INDEX idx_payment_logs_order_id ON payment_logs(order_id);
CREATE INDEX idx_payment_logs_user_id ON payment_logs(user_id);
CREATE INDEX idx_payment_logs_status ON payment_logs(status);
CREATE INDEX idx_payment_logs_created_at ON payment_logs(created_at);

CREATE INDEX idx_webhook_logs_webhook_type ON webhook_logs(webhook_type);
CREATE INDEX idx_webhook_logs_source ON webhook_logs(source);
CREATE INDEX idx_webhook_logs_status ON webhook_logs(status);
CREATE INDEX idx_webhook_logs_created_at ON webhook_logs(created_at);

CREATE INDEX idx_request_logs_request_id ON request_logs(request_id);
CREATE INDEX idx_request_logs_path ON request_logs(path);
CREATE INDEX idx_request_logs_user_id ON request_logs(user_id);
CREATE INDEX idx_request_logs_created_at ON request_logs(created_at);

CREATE INDEX idx_commission_rules_active ON commission_rules(is_active);
CREATE INDEX idx_commission_records_order_id ON commission_records(order_id);
CREATE INDEX idx_commission_records_user_id ON commission_records(user_id);
CREATE INDEX idx_commission_records_referrer_id ON commission_records(referrer_id);
CREATE INDEX idx_commission_records_status ON commission_records(status);

CREATE INDEX idx_cross_platform_configs_platform ON cross_platform_configs(platform_name);
CREATE INDEX idx_cross_platform_orders_platform ON cross_platform_orders(platform_name);
CREATE INDEX idx_cross_platform_orders_email ON cross_platform_orders(customer_email);
CREATE INDEX idx_cross_platform_orders_status ON cross_platform_orders(status);

CREATE INDEX idx_log_configs_key ON log_configs(config_key);
CREATE INDEX idx_log_configs_active ON log_configs(is_active);

-- ========================================
-- 第十一部分: 初始数据插入
-- ========================================

-- 区块链配置
INSERT INTO blockchain_configs (network, network_name, api_url, usdt_contract, wallet_addresses, min_confirmations) VALUES
('tron', 'TRON', 'https://api.trongrid.io', 'TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t', '[]', 3),
('ethereum', 'Ethereum', 'https://api.etherscan.io', '0xdAC17F958D2ee523a2206206994597C13D831ec7', '[]', 12)
ON CONFLICT (network) DO NOTHING;

-- PayPal 配置（sandbox 默认）
INSERT INTO paypal_configs (client_id, client_secret_encrypted, sandbox, webhook_id, base_url, return_url, cancel_url, brand_name, min_amount, max_amount, fee_rate)
VALUES (
    'sandbox_client_id_placeholder',
    'sandbox_client_secret_placeholder',
    true,
    NULL,
    'https://api-m.sandbox.paypal.com',
    'http://localhost:3000/payment/success',
    'http://localhost:3000/payment/cancel',
    'RSWS Store',
    0.01,
    10000.00,
    0.0349
)
ON CONFLICT DO NOTHING;

-- 支付方式配置
INSERT INTO payment_method_configs (method_id, method_name, icon_url, sort_order) VALUES
('paypal', 'PayPal', '/icons/paypal.svg', 1),
('usdt_tron', 'USDT (TRC20)', '/icons/usdt-tron.svg', 2),
('usdt_eth', 'USDT (ERC20)', '/icons/usdt-eth.svg', 3)
ON CONFLICT (method_id) DO NOTHING;

-- 加密配置
INSERT INTO encryption_configs (config_name, encryption_key_encrypted, algorithm) 
VALUES ('default', 'your_base64_encoded_encrypted_key_here', 'AES-256-GCM')
ON CONFLICT (config_name) DO NOTHING;

-- USDT 监听配置
INSERT INTO usdt_listen_configs (network, api_url, usdt_contract, poll_interval_seconds, min_confirmations)
VALUES 
('tron', 'https://api.trongrid.io', 'TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t', 10, 3),
('ethereum', 'https://api.etherscan.io', '0xdAC17F958D2ee523a2206206994597C13D831ec7', 15, 12)
ON CONFLICT (network) DO NOTHING;

-- API Key 认证配置
INSERT INTO system_configs (config_key, config_value, config_type, description, is_encrypted, is_public) VALUES
('api_key.session_expire_days', '7', 'number', 'API Key 会话有效期 (天)', false, false),
('api_key.signature_expire_seconds', '300', 'number', '签名有效期 (秒，防重放攻击)', false, false),
('api_key.max_sessions_per_user', '5', 'number', '每个用户最大会话数', false, false),
('api_key.enable_rate_limit', 'true', 'boolean', '是否启用速率限制', false, false),
('api_key.default_rate_limit', '100', 'number', '默认速率限制 (次/分钟)', false, false)
ON CONFLICT (config_key) DO NOTHING;

-- 日志配置 (system_configs)
INSERT INTO system_configs (config_key, config_value, config_type, description, is_encrypted, is_public) VALUES
('log.level', 'info', 'string', '日志级别: trace, debug, info, warn, error', false, false),
('log.enable_database_logging', 'true', 'boolean', '是否启用数据库日志', false, false),
('log.enable_file_logging', 'false', 'boolean', '是否启用文件日志', false, false),
('log.file_path', '/var/log/rsws/app.log', 'string', '日志文件路径', false, false),
('log.max_file_size', '10485760', 'number', '日志文件最大大小 (字节)', false, false),
('log.retention_days', '30', 'number', '日志保留天数', false, false),
('log.enable_error_logging', 'true', 'boolean', '是否启用错误日志', false, false),
('log.enable_operation_logging', 'true', 'boolean', '是否启用操作日志', false, false),
('log.enable_payment_logging', 'true', 'boolean', '是否启用支付日志', false, false)
ON CONFLICT (config_key) DO NOTHING;

-- 系统参数配置
INSERT INTO system_configs (config_key, config_value, config_type, description, is_encrypted, is_public) VALUES
('system.site_name', 'RSWS', 'string', '站点名称', false, true),
('system.site_url', 'https://example.com', 'string', '站点 URL', false, true),
('system.admin_email', 'admin@example.com', 'string', '管理员邮箱', false, false),
('system.default_currency', 'USD', 'string', '默认货币', false, true),
('system.order_expire_minutes', '30', 'number', '订单过期时间 (分钟)', false, false),
('system.max_upload_size', '104857600', 'number', '最大上传文件大小 (字节)', false, false),
('system.allowed_file_types', '["zip", "rar", "7z", "pdf", "doc", "docx"]', 'json', '允许的文件类型', false, false)
ON CONFLICT (config_key) DO NOTHING;

-- 安全配置
INSERT INTO system_configs (config_key, config_value, config_type, description, is_encrypted, is_public) VALUES
('security.password_min_length', '8', 'number', '密码最小长度', false, false),
('security.password_require_uppercase', 'true', 'boolean', '密码是否需要大写字母', false, false),
('security.password_require_lowercase', 'true', 'boolean', '密码是否需要小写字母', false, false),
('security.password_require_number', 'true', 'boolean', '密码是否需要数字', false, false),
('security.password_require_special', 'false', 'boolean', '密码是否需要特殊字符', false, false),
('security.login_max_attempts', '5', 'number', '最大登录尝试次数', false, false),
('security.login_lockout_minutes', '15', 'number', '登录锁定时间 (分钟)', false, false),
('security.enable_2fa', 'false', 'boolean', '是否启用双因素认证', false, false),
('security.session_timeout_minutes', '30', 'number', '会话超时时间 (分钟)', false, false)
ON CONFLICT (config_key) DO NOTHING;

-- 资源配置
INSERT INTO system_configs (config_key, config_value, config_type, description, is_encrypted, is_public) VALUES
('resource.default_commission_rate', '0.10', 'number', '默认佣金比例 (0.10 = 10%)', false, false),
('resource.max_free_preview_percent', '20', 'number', '免费预览最大百分比', false, false),
('resource.require_review', 'true', 'boolean', '用户上传资源是否需要审核', false, false),
('resource.auto_approve_trusted', 'false', 'boolean', '是否自动批准信任用户上传', false, false)
ON CONFLICT (config_key) DO NOTHING;

-- 支付配置
INSERT INTO system_configs (config_key, config_value, config_type, description, is_encrypted, is_public) VALUES
('payment.order_timeout_minutes', '30', 'number', '订单超时时间 (分钟)', false, false),
('payment.auto_confirm_usdt', 'true', 'boolean', '是否自动确认 USDT 支付', false, false),
('payment.usdt_unique_decimal', 'true', 'boolean', '是否使用唯一小数位匹配', false, false),
('payment.min_usdt_amount', '1', 'number', '最小 USDT 支付金额', false, true),
('payment.max_usdt_amount', '10000', 'number', '最大 USDT 支付金额', false, true)
ON CONFLICT (config_key) DO NOTHING;

-- 通知配置
INSERT INTO system_configs (config_key, config_value, config_type, description, is_encrypted, is_public) VALUES
('notification.email_on_register', 'true', 'boolean', '注册时发送邮件', false, false),
('notification.email_on_purchase', 'true', 'boolean', '购买时发送邮件', false, false),
('notification.email_on_payment_success', 'true', 'boolean', '支付成功时发送邮件', false, false),
('notification.email_on_resource_approved', 'true', 'boolean', '资源审核通过时发送邮件', false, false)
ON CONFLICT (config_key) DO NOTHING;

-- 日志配置 (log_configs 表)
INSERT INTO log_configs (config_key, config_value, config_type, description) VALUES
('log.level', 'info', 'string', '日志级别: trace, debug, info, warn, error'),
('log.enable_database_logging', 'true', 'boolean', '是否启用数据库日志'),
('log.enable_file_logging', 'false', 'boolean', '是否启用文件日志'),
('log.file_path', '/var/log/rsws/app.log', 'string', '日志文件路径'),
('log.max_file_size', '10485760', 'number', '日志文件最大大小 (字节)'),
('log.retention_days', '30', 'number', '日志保留天数'),
('log.enable_error_logging', 'true', 'boolean', '是否启用错误日志'),
('log.enable_operation_logging', 'true', 'boolean', '是否启用操作日志'),
('log.enable_payment_logging', 'true', 'boolean', '是否启用支付日志'),
('log.enable_request_logging', 'true', 'boolean', '是否启用请求日志')
ON CONFLICT (config_key) DO NOTHING;

-- ========================================
-- 第十二部分: 表注释
-- ========================================

COMMENT ON TABLE users IS '用户表';
COMMENT ON TABLE admins IS '管理员表';
COMMENT ON TABLE resources IS '资源表，支持 C2C 模式';
COMMENT ON TABLE user_payment_configs IS '用户收款配置表 (C2C 模式)';
COMMENT ON TABLE resource_payment_configs IS '资源收款配置关联表';
COMMENT ON TABLE orders IS '订单表';
COMMENT ON TABLE payment_transactions IS '支付交易表';
COMMENT ON TABLE user_sessions IS '用户会话表 (API Key 认证)';
COMMENT ON TABLE system_configs IS '系统配置表，存储所有可动态修改的配置项';
COMMENT ON TABLE paypal_configs IS 'PayPal 支付配置表';
COMMENT ON TABLE blockchain_configs IS '区块链配置表';
COMMENT ON TABLE payment_method_configs IS '支付方式配置表';
COMMENT ON TABLE encryption_configs IS '加密配置表';
COMMENT ON TABLE email_configs IS '邮件服务配置表';
COMMENT ON TABLE usdt_transactions IS 'USDT 交易记录表';
COMMENT ON TABLE usdt_wallets IS '收款地址池';
COMMENT ON TABLE usdt_listen_configs IS 'USDT 监听配置';
COMMENT ON TABLE admin_api_keys IS '管理员 API Key 表';
COMMENT ON TABLE domains IS '域名证书管理表';
COMMENT ON TABLE admin_operation_logs IS '管理员操作日志表';
COMMENT ON TABLE system_logs IS '系统日志表';
COMMENT ON TABLE error_logs IS '错误日志表';
COMMENT ON TABLE payment_logs IS '支付日志表';
COMMENT ON TABLE webhook_logs IS 'Webhook 日志表';
COMMENT ON TABLE request_logs IS '请求日志表';
COMMENT ON TABLE commission_rules IS '佣金规则表';
COMMENT ON TABLE commission_records IS '佣金记录表';
COMMENT ON TABLE cross_platform_configs IS '跨平台配置表';
COMMENT ON TABLE cross_platform_orders IS '跨平台订单表';
COMMENT ON TABLE log_configs IS '日志配置表，后台可动态管理';

COMMENT ON COLUMN system_configs.config_key IS '配置键，使用点分隔命名空间 (如 api_key.session_expire_days)';
COMMENT ON COLUMN system_configs.config_value IS '配置值，文本形式存储';
COMMENT ON COLUMN system_configs.config_type IS '值类型: string, number, boolean, json';
COMMENT ON COLUMN system_configs.is_encrypted IS '是否加密存储';
COMMENT ON COLUMN system_configs.is_public IS '是否可公开读取 (前端 API 可访问)';

COMMENT ON COLUMN usdt_transactions.tx_hash IS '交易 hash，唯一标识';
COMMENT ON COLUMN usdt_transactions.network IS '网络类型: tron 或 ethereum';
COMMENT ON COLUMN usdt_transactions.status IS '交易状态: pending-待处理, confirmed-已确认, processed-已处理, unmatched-未匹配';
COMMENT ON COLUMN usdt_transactions.order_id IS '关联的订单 ID，匹配成功时填充';
COMMENT ON COLUMN usdt_wallets.address IS '收款地址';
COMMENT ON COLUMN usdt_wallets.network IS '网络类型: tron 或 ethereum';
COMMENT ON COLUMN usdt_wallets.total_received IS '累计收款金额';
COMMENT ON COLUMN usdt_listen_configs.api_url IS 'API 地址 (TronGrid 或 Etherscan)';
COMMENT ON COLUMN usdt_listen_configs.usdt_contract IS 'USDT 合约地址';
COMMENT ON COLUMN usdt_listen_configs.poll_interval_seconds IS '轮询间隔 (秒)';
COMMENT ON COLUMN usdt_listen_configs.min_confirmations IS '最小确认数';
