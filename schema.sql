-- ========================================
-- RSWS_V1 统一数据库 Schema
-- 创建时间: 2026-05-31
-- 说明: 此为完整数据库结构，不含测试数据和迁移历史
-- 不包含已废弃字段（如 categories.slug）
-- ========================================

-- 启用 UUID 扩展
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- ========================================
-- 用户表
-- ========================================
CREATE TABLE IF NOT EXISTS users (
    id BIGINT PRIMARY KEY,
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    username VARCHAR(100) NOT NULL,
    nickname VARCHAR(100) NOT NULL,
    avatar_url VARCHAR(500),
    bio TEXT,
    website VARCHAR(255),
    location VARCHAR(100),
    is_active BOOLEAN DEFAULT true,
    is_verified BOOLEAN DEFAULT false,
    email_verified BOOLEAN DEFAULT false,
    verify_token VARCHAR(255),
    verify_token_expires_at TIMESTAMP WITH TIME ZONE,
    reset_token VARCHAR(255),
    reset_token_expires_at TIMESTAMP WITH TIME ZONE,
    last_login_at TIMESTAMP WITH TIME ZONE,
    last_login_ip VARCHAR(45),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);

-- ========================================
-- 管理员表
-- ========================================
CREATE TABLE IF NOT EXISTS admins (
    id BIGINT PRIMARY KEY,
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    username VARCHAR(100) NOT NULL,
    nickname VARCHAR(100) NOT NULL,
    avatar_url VARCHAR(500),
    is_active BOOLEAN DEFAULT true,
    role VARCHAR(50) DEFAULT 'admin',
    permissions JSONB DEFAULT '[]',
    last_login_at TIMESTAMP WITH TIME ZONE,
    last_login_ip VARCHAR(45),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_admins_email ON admins(email);

-- ========================================
-- 分类表（不含已废弃的 slug 字段）
-- ========================================
CREATE TABLE IF NOT EXISTS categories (
    id BIGINT PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    description TEXT,
    parent_id BIGINT REFERENCES categories(id) ON DELETE SET NULL,
    path TEXT DEFAULT '',
    sort_order INTEGER DEFAULT 0,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_categories_parent_id ON categories(parent_id);

-- ========================================
-- 资源表
-- ========================================
CREATE TABLE IF NOT EXISTS resources (
    id BIGINT PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    price NUMERIC(10,2) DEFAULT 0.00,
    category_id BIGINT REFERENCES categories(id) ON DELETE SET NULL,
    thumbnail_url VARCHAR(500),
    detail_images TEXT[],
    display_images TEXT[],
    file_url VARCHAR(500),
    file_size BIGINT,
    version VARCHAR(50),
    is_active BOOLEAN DEFAULT true,
    detail_description TEXT,
    specifications JSONB,
    usage_guide TEXT,
    precautions TEXT,
    download_count BIGINT DEFAULT 0,
    owner_type VARCHAR(20) DEFAULT 'admin',
    provider_id BIGINT,
    supported_os JSONB DEFAULT '[]',
    commission_rate NUMERIC(5,4) DEFAULT 0.0000,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_resources_category_id ON resources(category_id);
CREATE INDEX IF NOT EXISTS idx_resources_is_active ON resources(is_active);

-- ========================================
-- 订单表
-- ========================================
CREATE TABLE IF NOT EXISTS orders (
    id BIGINT PRIMARY KEY,
    user_id BIGINT REFERENCES users(id) ON DELETE SET NULL,
    resource_id BIGINT REFERENCES resources(id) ON DELETE SET NULL,
    amount NUMERIC(10,2) NOT NULL,
    status VARCHAR(20) DEFAULT 'pending',
    payment_method VARCHAR(50),
    payment_tx_id VARCHAR(255),
    paid_at TIMESTAMP WITH TIME ZONE,
    expired_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_orders_user_id ON orders(user_id);
CREATE INDEX IF NOT EXISTS idx_orders_status ON orders(status);

-- ========================================
-- 支付交易表
-- ========================================
CREATE TABLE IF NOT EXISTS payment_transactions (
    id BIGINT PRIMARY KEY,
    order_id BIGINT REFERENCES orders(id) ON DELETE CASCADE,
    tx_hash VARCHAR(255),
    amount NUMERIC(10,2),
    status VARCHAR(20) DEFAULT 'pending',
    payment_method VARCHAR(50),
    payer_email VARCHAR(255),
    payer_id VARCHAR(255),
    raw_data JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- ========================================
-- 用户支付配置表
-- ========================================
CREATE TABLE IF NOT EXISTS user_payment_configs (
    id BIGINT PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    config_name VARCHAR(100) NOT NULL,
    payment_method VARCHAR(50) NOT NULL,
    paypal_email VARCHAR(255),
    usdt_address VARCHAR(255),
    usdt_network VARCHAR(50) DEFAULT 'tron',
    is_active BOOLEAN DEFAULT true,
    is_default BOOLEAN DEFAULT false,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- ========================================
-- 资源支付配置关联表
-- ========================================
CREATE TABLE IF NOT EXISTS resource_payment_configs (
    resource_id BIGINT NOT NULL REFERENCES resources(id) ON DELETE CASCADE,
    payment_config_id BIGINT NOT NULL REFERENCES user_payment_configs(id) ON DELETE CASCADE,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    PRIMARY KEY (resource_id, payment_config_id)
);

-- ========================================
-- 系统配置表
-- ========================================
CREATE TABLE IF NOT EXISTS system_configs (
    id BIGINT PRIMARY KEY,
    config_key VARCHAR(100) NOT NULL UNIQUE,
    config_value TEXT,
    config_type VARCHAR(20) DEFAULT 'string',
    description TEXT,
    is_public BOOLEAN DEFAULT false,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- ========================================
-- PayPal 配置表
-- ========================================
CREATE TABLE IF NOT EXISTS paypal_configs (
    id BIGINT PRIMARY KEY,
    client_id VARCHAR(255) NOT NULL,
    client_secret_encrypted VARCHAR(255) NOT NULL,
    mode VARCHAR(20) DEFAULT 'sandbox',
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- ========================================
-- 菜单项表
-- ========================================
CREATE TABLE IF NOT EXISTS menu_items (
    id BIGINT PRIMARY KEY,
    parent_id BIGINT REFERENCES menu_items(id) ON DELETE CASCADE,
    title VARCHAR(100) NOT NULL,
    icon VARCHAR(100),
    route_path VARCHAR(255),
    route_name VARCHAR(100),
    permission VARCHAR(100),
    sort_order INTEGER DEFAULT 0,
    is_visible BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- ========================================
-- 管理员操作日志表
-- ========================================
CREATE TABLE IF NOT EXISTS admin_operation_logs (
    id BIGINT PRIMARY KEY,
    admin_id BIGINT NOT NULL REFERENCES admins(id) ON DELETE CASCADE,
    action VARCHAR(100) NOT NULL,
    resource_type VARCHAR(100),
    resource_id VARCHAR(100),
    details JSONB,
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- ========================================
-- USDT 交易表
-- ========================================
CREATE TABLE IF NOT EXISTS usdt_transactions (
    id BIGINT PRIMARY KEY,
    order_id BIGINT REFERENCES orders(id) ON DELETE SET NULL,
    tx_hash VARCHAR(255) NOT NULL,
    from_address VARCHAR(255),
    to_address VARCHAR(255),
    amount NUMERIC(20,6),
    currency VARCHAR(10) DEFAULT 'USDT',
    network VARCHAR(50),
    confirmations INTEGER DEFAULT 0,
    status VARCHAR(20) DEFAULT 'pending',
    observed_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    confirmed_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- ========================================
-- 佣金规则表
-- ========================================
CREATE TABLE IF NOT EXISTS commission_rules (
    id BIGINT PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    rule_type VARCHAR(50) NOT NULL,
    rate NUMERIC(5,4) DEFAULT 0.0000,
    min_amount NUMERIC(10,2) DEFAULT 0.00,
    max_amount NUMERIC(10,2),
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- ========================================
-- 佣金记录表
-- ========================================
CREATE TABLE IF NOT EXISTS commission_records (
    id BIGINT PRIMARY KEY,
    order_id BIGINT REFERENCES orders(id) ON DELETE SET NULL,
    user_id BIGINT REFERENCES users(id) ON DELETE SET NULL,
    referrer_id BIGINT REFERENCES users(id) ON DELETE SET NULL,
    rule_id BIGINT REFERENCES commission_rules(id) ON DELETE SET NULL,
    order_amount NUMERIC(10,2),
    commission_amount NUMERIC(10,2),
    commission_rate NUMERIC(5,4),
    status VARCHAR(20) DEFAULT 'pending',
    settled_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- ========================================
-- 区块链配置表
-- ========================================
CREATE TABLE IF NOT EXISTS blockchain_configs (
    id BIGINT PRIMARY KEY,
    network VARCHAR(50) NOT NULL UNIQUE,
    network_name VARCHAR(100) NOT NULL,
    api_url VARCHAR(500) NOT NULL,
    api_key_encrypted VARCHAR(500),
    usdt_contract VARCHAR(255) NOT NULL,
    min_confirmations INTEGER DEFAULT 3,
    min_amount NUMERIC(20,6) DEFAULT 0,
    max_amount NUMERIC(20,6) DEFAULT 1000000,
    fee_rate NUMERIC(5,4) DEFAULT 0.0000,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- ========================================
-- USDT 监听配置表
-- ========================================
CREATE TABLE IF NOT EXISTS usdt_listen_configs (
    id BIGINT PRIMARY KEY,
    network VARCHAR(50) NOT NULL UNIQUE,
    api_url VARCHAR(500) NOT NULL,
    api_key_encrypted VARCHAR(500),
    usdt_contract VARCHAR(255) NOT NULL,
    poll_interval_seconds INTEGER DEFAULT 30,
    min_confirmations INTEGER DEFAULT 3,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- ========================================
-- 邮件配置表
-- ========================================
CREATE TABLE IF NOT EXISTS email_configs (
    id BIGINT PRIMARY KEY,
    provider VARCHAR(50) NOT NULL DEFAULT 'smtp',
    host VARCHAR(255),
    port INTEGER,
    username VARCHAR(255),
    password_encrypted VARCHAR(500),
    use_tls BOOLEAN DEFAULT true,
    from_email VARCHAR(255) NOT NULL,
    from_name VARCHAR(255),
    reply_to VARCHAR(255),
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- ========================================
-- 日志配置表（来自 002_log_tables.sql）
-- ========================================
CREATE TABLE IF NOT EXISTS log_configs (
    id BIGINT PRIMARY KEY,
    config_key VARCHAR(100) NOT NULL UNIQUE,
    config_value TEXT,
    description TEXT,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- ========================================
-- USDT 钱包表（如需）
-- ========================================
-- CREATE TABLE IF NOT EXISTS usdt_wallets (
--     id BIGINT PRIMARY KEY,
--     user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
--     address VARCHAR(255) NOT NULL,
--     network VARCHAR(50) NOT NULL DEFAULT 'tron',
--     is_active BOOLEAN DEFAULT true,
--     created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
--     updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
--     UNIQUE(address, network)
-- );

-- ========================================
-- 用户 API Key 表
-- ========================================
CREATE TABLE IF NOT EXISTS user_api_keys (
    id BIGINT PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    api_key VARCHAR(100) NOT NULL UNIQUE,
    name VARCHAR(100) NOT NULL,
    permissions JSONB DEFAULT '["read"]'::jsonb,
    rate_limit INT DEFAULT 100,
    last_used_at TIMESTAMP WITH TIME ZONE,
    expires_at TIMESTAMP WITH TIME ZONE,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_user_api_keys_user_id ON user_api_keys(user_id);
CREATE INDEX IF NOT EXISTS idx_user_api_keys_api_key ON user_api_keys(api_key);

-- ========================================
-- 完成提示
-- ========================================
-- 注意：
-- 1. 所有 ID 使用 BIGINT（非 BIGSERIAL），由应用层生成（snowflake::next_id()）
-- 2. 不含已废弃字段（如 categories.slug）
-- 3. 使用 UTF-8 编码，LF 换行符
-- 4. 执行：psql -d rsws_db -f schema.sql
-- ========================================
