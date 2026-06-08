-- ========================================
-- RSWS_V1 缁熶竴鏁版嵁搴?Schema
-- 鍒涘缓鏃堕棿: 2026-06-08
-- 璇存槑: 瀹屾暣鍒濆杩佺Щ
-- 鍙樻洿: 绉婚櫎 user_api_keys / admin_api_keys 琛?--       绉婚櫎 _encrypted 鍚庣紑
--       paypal_configs 鎵╁睍涓哄畬鏁村瓧娈?-- ========================================

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

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

CREATE TABLE IF NOT EXISTS resource_payment_configs (
    resource_id BIGINT NOT NULL REFERENCES resources(id) ON DELETE CASCADE,
    payment_config_id BIGINT NOT NULL REFERENCES user_payment_configs(id) ON DELETE CASCADE,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    PRIMARY KEY (resource_id, payment_config_id)
);

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

CREATE TABLE IF NOT EXISTS paypal_configs (
    id BIGINT PRIMARY KEY,
    client_id VARCHAR(255) NOT NULL,
    client_secret VARCHAR(255) NOT NULL,
    sandbox BOOLEAN DEFAULT true,
    webhook_id VARCHAR(255),
    webhook_secret VARCHAR(255),
    base_url VARCHAR(500) NOT NULL DEFAULT 'https://api-m.sandbox.paypal.com',
    return_url VARCHAR(500) NOT NULL DEFAULT 'http://localhost:4001/payment/success',
    cancel_url VARCHAR(500) NOT NULL DEFAULT 'http://localhost:4001/payment/cancel',
    brand_name VARCHAR(255) NOT NULL DEFAULT 'RSWS',
    min_amount NUMERIC(10,2) DEFAULT 1.00,
    max_amount NUMERIC(10,2) DEFAULT 10000.00,
    fee_rate NUMERIC(5,4) DEFAULT 0.0000,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

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

CREATE TABLE IF NOT EXISTS blockchain_configs (
    id BIGINT PRIMARY KEY,
    network VARCHAR(50) NOT NULL UNIQUE,
    network_name VARCHAR(100) NOT NULL,
    api_url VARCHAR(500) NOT NULL,
    api_key VARCHAR(500),
    usdt_contract VARCHAR(255) NOT NULL,
    min_confirmations INTEGER DEFAULT 3,
    min_amount NUMERIC(20,6) DEFAULT 0,
    max_amount NUMERIC(20,6) DEFAULT 1000000,
    fee_rate NUMERIC(5,4) DEFAULT 0.0000,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS usdt_listen_configs (
    id BIGINT PRIMARY KEY,
    network VARCHAR(50) NOT NULL UNIQUE,
    api_url VARCHAR(500) NOT NULL,
    api_key VARCHAR(500),
    usdt_contract VARCHAR(255) NOT NULL,
    poll_interval_seconds INTEGER DEFAULT 30,
    min_confirmations INTEGER DEFAULT 3,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS email_configs (
    id BIGINT PRIMARY KEY,
    provider VARCHAR(50) NOT NULL DEFAULT 'smtp',
    host VARCHAR(255),
    port INTEGER,
    username VARCHAR(255),
    password VARCHAR(500),
    use_tls BOOLEAN DEFAULT true,
    from_email VARCHAR(255) NOT NULL,
    from_name VARCHAR(255),
    reply_to VARCHAR(255),
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS log_configs (
    id BIGINT PRIMARY KEY,
    config_key VARCHAR(100) NOT NULL UNIQUE,
    config_value TEXT,
    description TEXT,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS payment_methods (
    id BIGINT PRIMARY KEY,
    method_type VARCHAR(50) NOT NULL UNIQUE,
    method_name VARCHAR(100) NOT NULL,
    is_enabled BOOLEAN DEFAULT true,
    config JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);
