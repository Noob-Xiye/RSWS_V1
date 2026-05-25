-- ========================================
-- RSWS 完整数据库 Schema (整合版)
-- 版本: 001_initial_schema
-- ========================================

-- 枚举类型
DO  BEGIN
    CREATE TYPE order_status AS ENUM ('pending','paid','completed','cancelled','refunded','failed');
EXCEPTION
    WHEN duplicate_object THEN null;
END ;

DO  BEGIN
    CREATE TYPE transaction_status AS ENUM ('pending','completed','failed','cancelled');
EXCEPTION
    WHEN duplicate_object THEN null;
END ;

-- 用户表
CREATE TABLE IF NOT EXISTS users (
    id BIGSERIAL PRIMARY KEY,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255),
    username VARCHAR(100),
    avatar_url VARCHAR(500),
    is_active BOOLEAN DEFAULT true,
    email_verified BOOLEAN DEFAULT false,
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

-- 资源表 (C2C)
CREATE TABLE IF NOT EXISTS resources (
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
    download_count INTEGER DEFAULT 0,
    provider_type VARCHAR(20) DEFAULT 'admin',
    provider_id BIGINT,
    owner_type VARCHAR(20) DEFAULT 'user',
    supported_os JSONB DEFAULT '[]',
    commission_rate DECIMAL(5,4) DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- 分类表 (层级)
CREATE TABLE IF NOT EXISTS categories (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    slug VARCHAR(100) NOT NULL UNIQUE,
    description TEXT,
    parent_id BIGINT,
    path VARCHAR(500),
    sort_order INTEGER DEFAULT 0,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- 用户收款配置
CREATE TABLE IF NOT EXISTS user_payment_configs (
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

-- 资源收款配置
CREATE TABLE IF NOT EXISTS resource_payment_configs (
    id BIGSERIAL PRIMARY KEY,
    resource_id BIGINT NOT NULL REFERENCES resources(id) ON DELETE CASCADE,
    payment_config_id BIGINT NOT NULL REFERENCES user_payment_configs(id) ON DELETE CASCADE,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(resource_id, payment_config_id)
);


-- 订单
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
    referrer_id BIGINT REFERENCES users(id),
    transaction_id VARCHAR(50),
    UNIQUE(user_id, resource_id)
);

-- 支付交易
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

-- 系统配置
CREATE TABLE IF NOT EXISTS system_configs (
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

-- PayPal配置
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

-- 区块链配置
CREATE TABLE IF NOT EXISTS blockchain_configs (
    id SERIAL PRIMARY KEY,
    network VARCHAR(50) NOT NULL UNIQUE,
    network_name VARCHAR(100) NOT NULL,
    api_url VARCHAR(500) NOT NULL,
    api_key_encrypted TEXT,
    usdt_contract VARCHAR(100) NOT NULL,
    wallet_addresses JSONB DEFAULT '[]',
    min_confirmations INTEGER NOT NULL DEFAULT 1,
    min_amount DECIMAL(10,2) NOT NULL DEFAULT 1.00,
    max_amount DECIMAL(10,2) NOT NULL DEFAULT 50000.00,
    fee_rate DECIMAL(5,4) NOT NULL DEFAULT 0,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- 支付方式
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


-- 邮件配置
CREATE TABLE IF NOT EXISTS email_configs (
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

-- USDT交易
CREATE TABLE IF NOT EXISTS usdt_transactions (
    id BIGINT PRIMARY KEY,
    tx_hash VARCHAR(66) NOT NULL UNIQUE,
    network VARCHAR(20) NOT NULL,
    from_address VARCHAR(100) NOT NULL,
    to_address VARCHAR(100) NOT NULL,
    amount NUMERIC(20,6) NOT NULL,
    block_number BIGINT,
    confirmations INTEGER DEFAULT 0,
    status VARCHAR(20) DEFAULT 'pending',
    order_id BIGINT REFERENCES orders(id),
    processed_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- USDT钱包
CREATE TABLE IF NOT EXISTS usdt_wallets (
    id BIGINT PRIMARY KEY,
    address VARCHAR(100) NOT NULL UNIQUE,
    network VARCHAR(20) NOT NULL,
    name VARCHAR(100),
    is_active BOOLEAN DEFAULT true,
    total_received NUMERIC(20,6) DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- USDT监听配置
CREATE TABLE IF NOT EXISTS usdt_listen_configs (
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

-- 管理员API Keys

-- 域名证书
CREATE TABLE IF NOT EXISTS domains (
    id SERIAL PRIMARY KEY,
    domain VARCHAR(255) NOT NULL UNIQUE,
    cert_pem TEXT NOT NULL,
    key_pem TEXT NOT NULL,
    is_active BOOLEAN DEFAULT true,
    expires_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- 管理员操作日志
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

-- 系统日志
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

-- 错误日志
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

-- 支付日志
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

-- Webhook日志
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

-- 请求日志
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

-- 日志配置
CREATE TABLE IF NOT EXISTS log_configs (
    id SERIAL PRIMARY KEY,
    config_key VARCHAR(100) NOT NULL UNIQUE,
    config_value TEXT NOT NULL,
    config_type VARCHAR(20) NOT NULL DEFAULT 'string',
    description TEXT,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- 佣金规则
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

-- 佣金记录
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

-- 跨平台配置
CREATE TABLE IF NOT EXISTS cross_platform_configs (
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

-- 跨平台订单
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

-- 索引
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_users_active ON users(is_active);
CREATE INDEX IF NOT EXISTS idx_admins_email ON admins(email);
CREATE INDEX IF NOT EXISTS idx_admins_role ON admins(role);
CREATE INDEX IF NOT EXISTS idx_resources_cat ON resources(category_id);
CREATE INDEX IF NOT EXISTS idx_resources_active ON resources(is_active);
CREATE INDEX IF NOT EXISTS idx_resources_provider ON resources(provider_type);
CREATE INDEX IF NOT EXISTS idx_categories_parent ON categories(parent_id);
CREATE INDEX IF NOT EXISTS idx_categories_slug ON categories(slug);
CREATE INDEX IF NOT EXISTS idx_orders_user ON orders(user_id);
CREATE INDEX IF NOT EXISTS idx_orders_status ON orders(status);
CREATE INDEX IF NOT EXISTS idx_payment_trans_order ON payment_transactions(order_id);
CREATE INDEX IF NOT EXISTS idx_system_config_key ON system_configs(config_key);
CREATE INDEX IF NOT EXISTS idx_blockchain_net ON blockchain_configs(network);
CREATE INDEX IF NOT EXISTS idx_usdt_tx_hash ON usdt_transactions(tx_hash);
CREATE INDEX IF NOT EXISTS idx_usdt_wallet_addr ON usdt_wallets(address);
CREATE INDEX IF NOT EXISTS idx_admin_ops_logs ON admin_operation_logs(admin_id);
CREATE INDEX IF NOT EXISTS idx_system_logs_level ON system_logs(log_level);
CREATE INDEX IF NOT EXISTS idx_error_logs_type ON error_logs(error_type);
CREATE INDEX IF NOT EXISTS idx_request_logs_req ON request_logs(request_id);

-- 初始数据
INSERT INTO blockchain_configs (network, network_name, api_url, usdt_contract, min_confirmations) VALUES
('tron', 'TRON', 'https://api.trongrid.io', 'TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t', 3),
('ethereum', 'Ethereum', 'https://api.etherscan.io', '0xdAC17F958D2ee523a2206206994597C13D831ec7', 12)
ON CONFLICT (network) DO NOTHING;

INSERT INTO payment_method_configs (method_id, method_name, icon_url, sort_order) VALUES
('paypal', 'PayPal', '/icons/paypal.svg', 1),
('usdt_tron', 'USDT (TRC20)', '/icons/usdt-tron.svg', 2),
('usdt_eth', 'USDT (ERC20)', '/icons/usdt-eth.svg', 3)
ON CONFLICT (method_id) DO NOTHING;

INSERT INTO usdt_listen_configs (network, api_url, usdt_contract, poll_interval_seconds, min_confirmations)
VALUES 
('tron', 'https://api.trongrid.io', 'TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t', 10, 3),
('ethereum', 'https://api.etherscan.io', '0xdAC17F958D2ee523a2206206994597C13D831ec7', 15, 12)
ON CONFLICT (network) DO NOTHING;

INSERT INTO system_configs (config_key, config_value, config_type, description, is_public) VALUES
('system.site_name', 'RSWS', 'string', '站点名称', true),
('system.site_url', 'https://example.com', 'string', '站点URL', true),
('system.default_currency', 'USD', 'string', '默认货币', true),
('payment.min_usdt_amount', '1', 'number', '最小USDT', true),
('payment.max_usdt_amount', '10000', 'number', '最大USDT', true),
('api_key.session_expire_days', '7', 'number', 'API Key有效期(天)', false),
('log.level', 'info', 'string', '日志级别', false)
ON CONFLICT (config_key) DO NOTHING;

