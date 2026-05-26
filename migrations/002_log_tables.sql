-- RSWS 日志表初始化
-- 文件名: migrations/002_log_tables.sql

-- ========== log_configs 日志配置 ==========
CREATE TABLE IF NOT EXISTS log_configs (
    id BIGSERIAL PRIMARY KEY,
    config_key VARCHAR(100) NOT NULL UNIQUE,
    config_value TEXT,
    config_type VARCHAR(20) DEFAULT 'string',
    description TEXT,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- ========== system_logs 系统日志 ==========
CREATE TABLE IF NOT EXISTS system_logs (
    id BIGINT NOT NULL,
    log_level VARCHAR(20) NOT NULL,
    module VARCHAR(100) NOT NULL,
    message TEXT NOT NULL,
    context JSONB,
    user_id BIGINT,
    admin_id BIGINT,
    ip_address VARCHAR(45),
    user_agent TEXT,
    request_id VARCHAR(100),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_system_logs_level ON system_logs(log_level);
CREATE INDEX IF NOT EXISTS idx_system_logs_module ON system_logs(module);
CREATE INDEX IF NOT EXISTS idx_system_logs_created_at ON system_logs(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_system_logs_user_id ON system_logs(user_id);
CREATE INDEX IF NOT EXISTS idx_system_logs_admin_id ON system_logs(admin_id);

-- ========== error_logs 错误日志 ==========
CREATE TABLE IF NOT EXISTS error_logs (
    id BIGINT NOT NULL,
    error_type VARCHAR(100) NOT NULL,
    error_message TEXT NOT NULL,
    stack_trace TEXT,
    module VARCHAR(100),
    function_name VARCHAR(100),
    user_id BIGINT,
    admin_id BIGINT,
    request_id VARCHAR(100),
    context JSONB,
    ip_address VARCHAR(45),
    user_agent TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_error_logs_type ON error_logs(error_type);
CREATE INDEX IF NOT EXISTS idx_error_logs_created_at ON error_logs(created_at DESC);

-- ========== payment_logs 支付日志 ==========
CREATE TABLE IF NOT EXISTS payment_logs (
    id BIGINT NOT NULL,
    transaction_id VARCHAR(255),
    order_id BIGINT,
    user_id BIGINT NOT NULL,
    payment_method VARCHAR(50) NOT NULL,
    amount BIGINT NOT NULL,
    currency VARCHAR(10) DEFAULT 'USD',
    status VARCHAR(20) NOT NULL,
    provider_response JSONB,
    gateway_transaction_id VARCHAR(255),
    ip_address VARCHAR(45),
    user_agent TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_payment_logs_order_id ON payment_logs(order_id);
CREATE INDEX IF NOT EXISTS idx_payment_logs_user_id ON payment_logs(user_id);
CREATE INDEX IF NOT EXISTS idx_payment_logs_status ON payment_logs(status);
CREATE INDEX IF NOT EXISTS idx_payment_logs_created_at ON payment_logs(created_at DESC);

-- ========== request_logs 请求日志 ==========
CREATE TABLE IF NOT EXISTS request_logs (
    id BIGINT NOT NULL,
    request_id VARCHAR(100) NOT NULL,
    method VARCHAR(10) NOT NULL,
    path VARCHAR(255) NOT NULL,
    query_params JSONB,
    user_id BIGINT,
    admin_id BIGINT,
    ip_address VARCHAR(45),
    user_agent TEXT,
    response_status INTEGER,
    duration_ms INTEGER,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_request_logs_request_id ON request_logs(request_id);
CREATE INDEX IF NOT EXISTS idx_request_logs_created_at ON request_logs(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_request_logs_path ON request_logs(path);
