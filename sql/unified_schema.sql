-- ========================================
-- RSWS 统一数据库架构文件
-- 包含所有表结构、索引、枚举类型和初始数据
-- ========================================

-- 订单状态枚举
CREATE TYPE order_status AS ENUM (
    'pending',
    'paid', 
    'completed',
    'cancelled',
    'refunded',
    'failed'
);

-- 交易状态枚举
CREATE TYPE transaction_status AS ENUM (
    'pending',
    'completed',
    'failed',
    'cancelled'
);

-- ========================================
-- 核心业务表
-- ========================================

-- 用户表
CREATE TABLE IF NOT EXISTS users (
    id BIGSERIAL PRIMARY KEY,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255),
    username VARCHAR(100),
    avatar_url VARCHAR(500),
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- 管理员表
CREATE TABLE IF NOT EXISTS admins (
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

-- 资源表（修改后）
CREATE TABLE IF NOT EXISTS resources (
    id BIGSERIAL PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    price DECIMAL(10,2) NOT NULL,
    category_id BIGINT,
    file_url VARCHAR(500),
    thumbnail_url VARCHAR(500),
    is_active BOOLEAN DEFAULT true,
    -- 商品详情字段
    detail_description TEXT,
    specifications JSONB,
    usage_guide TEXT,
    precautions TEXT,
    display_images TEXT[],
    -- 新增字段：资源提供者标记
    provider_type VARCHAR(20) NOT NULL DEFAULT 'admin', -- 'admin' 或 'user'
    provider_id BIGINT, -- 当provider_type='user'时，关联users表的id；当provider_type='admin'时，关联admins表的id
    commission_rate DECIMAL(5,4) DEFAULT 0.0000, -- 佣金比例（仅用户提供的资源）
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- 用户收款配置表（新增）
CREATE TABLE IF NOT EXISTS user_payment_configs (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    config_name VARCHAR(100) NOT NULL, -- 配置名称
    payment_method VARCHAR(50) NOT NULL, -- 'paypal' 或 'usdt_tron' 或 'usdt_eth'
    -- PayPal配置
    paypal_email VARCHAR(255), -- PayPal收款邮箱
    paypal_merchant_id VARCHAR(100), -- PayPal商户ID
    -- USDT配置
    usdt_address VARCHAR(100), -- USDT收款地址
    usdt_network VARCHAR(20), -- 'tron' 或 'ethereum'
    -- 通用配置
    is_active BOOLEAN DEFAULT true,
    is_default BOOLEAN DEFAULT false, -- 是否为默认收款方式
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(user_id, config_name)
);

-- 资源收款配置关联表（新增）
CREATE TABLE IF NOT EXISTS resource_payment_configs (
    id BIGSERIAL PRIMARY KEY,
    resource_id BIGINT NOT NULL REFERENCES resources(id) ON DELETE CASCADE,
    payment_config_id BIGINT NOT NULL REFERENCES user_payment_configs(id) ON DELETE CASCADE,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(resource_id, payment_config_id)
);

-- 订单表
CREATE TABLE IF NOT EXISTS orders (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id),
    resource_id BIGINT NOT NULL REFERENCES resources(id),
    amount DECIMAL(10,2) NOT NULL,
    status order_status NOT NULL DEFAULT 'pending',
    payment_method VARCHAR(50),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    expired_at TIMESTAMP WITH TIME ZONE,
    UNIQUE(user_id, resource_id)
);

-- 支付交易表
CREATE TABLE IF NOT EXISTS payment_transactions (
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
-- 配置管理表
-- ========================================

-- PayPal配置表
CREATE TABLE IF NOT EXISTS paypal_configs (
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

-- 区块链配置表
CREATE TABLE IF NOT EXISTS blockchain_configs (
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

-- 支付方式配置表
CREATE TABLE IF NOT EXISTS payment_method_configs (
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

-- 加密配置表
CREATE TABLE IF NOT EXISTS encryption_configs (
    id SERIAL PRIMARY KEY,
    config_name VARCHAR(100) NOT NULL UNIQUE,
    encryption_key_encrypted TEXT NOT NULL,
    algorithm VARCHAR(50) NOT NULL DEFAULT 'AES-256-GCM',
    key_version INTEGER NOT NULL DEFAULT 1,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- ========================================
-- API管理表
-- ========================================

-- 管理员API Key表
CREATE TABLE IF NOT EXISTS admin_api_keys (
    id SERIAL PRIMARY KEY,
    admin_id INTEGER NOT NULL REFERENCES admins(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    api_key VARCHAR(64) NOT NULL UNIQUE,
    api_secret_encrypted TEXT NOT NULL,
    permissions JSONB NOT NULL DEFAULT '[]',
    rate_limit INTEGER,
    last_used_at TIMESTAMP WITH TIME ZONE,
    expires_at TIMESTAMP WITH TIME ZONE,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- ========================================
-- 日志系统表
-- ========================================

-- 管理员操作日志表
CREATE TABLE IF NOT EXISTS admin_operation_logs (
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

-- 系统日志表
CREATE TABLE IF NOT EXISTS system_logs (
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

-- 错误日志表
CREATE TABLE IF NOT EXISTS error_logs (
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

-- 支付日志表
CREATE TABLE IF NOT EXISTS payment_logs (
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

-- Webhook日志表
CREATE TABLE IF NOT EXISTS webhook_logs (
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

-- 请求日志表
CREATE TABLE IF NOT EXISTS request_logs (
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
-- 佣金系统表
-- ========================================

-- 佣金规则表
CREATE TABLE IF NOT EXISTS commission_rules (
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

-- 佣金记录表
CREATE TABLE IF NOT EXISTS commission_records (
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
-- 跨平台集成表
-- ========================================

-- 跨平台配置表
CREATE TABLE IF NOT EXISTS cross_platform_configs (
    id BIGINT PRIMARY KEY,
    platform_name VARCHAR(50) NOT NULL UNIQUE,
    api_endpoint VARCHAR(255) NOT NULL,
    api_key VARCHAR(255) NOT NULL,
    api_secret VARCHAR(255),
    webhook_secret VARCHAR(255),
    is_active BOOLEAN DEFAULT true,
    sync_interval INTEGER DEFAULT 300,
    last_sync_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- 跨平台订单表
CREATE TABLE IF NOT EXISTS cross_platform_orders (
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

-- ========================================
-- 索引创建
-- ========================================

-- 用户表索引
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_is_active ON users(is_active);

-- 管理员表索引
CREATE INDEX idx_admins_email ON admins(email);
CREATE INDEX idx_admins_role ON admins(role);
CREATE INDEX idx_admins_is_active ON admins(is_active);

-- 资源表新增索引
CREATE INDEX idx_resources_provider_type ON resources(provider_type);
CREATE INDEX idx_resources_provider_id ON resources(provider_id);

-- 用户收款配置表索引
CREATE INDEX idx_user_payment_configs_user_id ON user_payment_configs(user_id);
CREATE INDEX idx_user_payment_configs_payment_method ON user_payment_configs(payment_method);
CREATE INDEX idx_user_payment_configs_is_active ON user_payment_configs(is_active);
CREATE INDEX idx_user_payment_configs_is_default ON user_payment_configs(is_default);

-- 资源收款配置关联表索引
CREATE INDEX idx_resource_payment_configs_resource_id ON resource_payment_configs(resource_id);
CREATE INDEX idx_resource_payment_configs_payment_config_id ON resource_payment_configs(payment_config_id);
CREATE INDEX idx_resources_category_id ON resources(category_id);
CREATE INDEX idx_resources_is_active ON resources(is_active);
CREATE INDEX idx_resources_specifications ON resources USING GIN (specifications);

-- 订单表索引
CREATE INDEX idx_orders_user_id ON orders(user_id);
CREATE INDEX idx_orders_status ON orders(status);
CREATE INDEX idx_orders_created_at ON orders(created_at);

-- 支付交易表索引
CREATE INDEX idx_payment_transactions_order_id ON payment_transactions(order_id);
CREATE INDEX idx_payment_transactions_user_id ON payment_transactions(user_id);
CREATE INDEX idx_payment_transactions_status ON payment_transactions(status);

-- 配置表索引
CREATE INDEX idx_blockchain_configs_network ON blockchain_configs(network);
CREATE INDEX idx_blockchain_configs_is_active ON blockchain_configs(is_active);
CREATE INDEX idx_payment_method_configs_method_id ON payment_method_configs(method_id);
CREATE INDEX idx_payment_method_configs_is_active ON payment_method_configs(is_active);
CREATE INDEX idx_payment_method_configs_sort_order ON payment_method_configs(sort_order);

-- API Key表索引
CREATE INDEX idx_admin_api_keys_admin_id ON admin_api_keys(admin_id);
CREATE INDEX idx_admin_api_keys_api_key ON admin_api_keys(api_key);
CREATE INDEX idx_admin_api_keys_active ON admin_api_keys(is_active);

-- 日志表索引
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

-- 佣金表索引
CREATE INDEX idx_commission_rules_active ON commission_rules(is_active);
CREATE INDEX idx_commission_records_order_id ON commission_records(order_id);
CREATE INDEX idx_commission_records_user_id ON commission_records(user_id);
CREATE INDEX idx_commission_records_referrer_id ON commission_records(referrer_id);
CREATE INDEX idx_commission_records_status ON commission_records(status);

-- 跨平台表索引
CREATE INDEX idx_cross_platform_configs_platform ON cross_platform_configs(platform_name);
CREATE INDEX idx_cross_platform_orders_platform ON cross_platform_orders(platform_name);
CREATE INDEX idx_cross_platform_orders_email ON cross_platform_orders(customer_email);
CREATE INDEX idx_cross_platform_orders_status ON cross_platform_orders(status);

-- ========================================
-- 初始数据插入
-- ========================================

-- 插入默认区块链配置
INSERT INTO blockchain_configs (network, network_name, api_url, usdt_contract, wallet_addresses, min_confirmations) VALUES
('tron', 'TRON', 'https://api.trongrid.io', 'TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t', '[]', 1),
('ethereum', 'Ethereum', 'https://api.etherscan.io/api', '0xdAC17F958D2ee523a2206206994597C13D831ec7', '[]', 12)
ON CONFLICT (network) DO NOTHING;

-- 插入默认支付方式配置
INSERT INTO payment_method_configs (method_id, method_name, icon_url, sort_order) VALUES
('paypal', 'PayPal', '/icons/paypal.svg', 1),
('usdt_tron', 'USDT (TRC20)', '/icons/usdt-tron.svg', 2),
('usdt_eth', 'USDT (ERC20)', '/icons/usdt-eth.svg', 3)
ON CONFLICT (method_id) DO NOTHING;

-- 插入默认加密配置
INSERT INTO encryption_configs (config_name, encryption_key_encrypted, algorithm) 
VALUES ('default', 'your_base64_encoded_encrypted_key_here', 'AES-256-GCM')
ON CONFLICT (config_name) DO NOTHING;