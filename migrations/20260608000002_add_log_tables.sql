-- Migration: Add login_logs, error_logs, audit_logs tables and fix operation_type
-- Created: 2026-06-08

-- ============================================
-- 1. Fix operation_type field in system_logs
-- ============================================
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'system_logs' AND column_name = 'operation_type'
    ) THEN
        ALTER TABLE system_logs ADD COLUMN operation_type VARCHAR(50);
    END IF;
END $$;

-- Add index for operation_type if not exists
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_indexes 
        WHERE indexname = 'idx_system_logs_operation_type'
    ) THEN
        CREATE INDEX idx_system_logs_operation_type ON system_logs(operation_type);
    END IF;
END $$;

-- ============================================
-- 2. Create login_logs table
-- ============================================
CREATE TABLE IF NOT EXISTS login_logs (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT REFERENCES users(id) ON DELETE SET NULL,
    login_type VARCHAR(20) NOT NULL CHECK (login_type IN ('password', 'api_key', 'oauth', 'email_link')),
    status VARCHAR(20) NOT NULL CHECK (status IN ('success', 'failed', 'locked', 'expired')),
    ip_address INET,
    user_agent TEXT,
    device_info JSONB DEFAULT '{}',
    fail_reason VARCHAR(100),
    request_id VARCHAR(64),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_login_logs_user_id ON login_logs(user_id);
CREATE INDEX IF NOT EXISTS idx_login_logs_created_at ON login_logs(created_at);
CREATE INDEX IF NOT EXISTS idx_login_logs_status ON login_logs(status);
CREATE INDEX IF NOT EXISTS idx_login_logs_ip_address ON login_logs(ip_address);

-- ============================================
-- 3. Create error_logs table
-- ============================================
CREATE TABLE IF NOT EXISTS error_logs (
    id BIGSERIAL PRIMARY KEY,
    error_type VARCHAR(50) NOT NULL CHECK (error_type IN ('panic', 'exception', 'timeout', 'validation', 'database', 'external_api')),
    error_message TEXT NOT NULL,
    stack_trace TEXT,
    request_id VARCHAR(64),
    user_id BIGINT REFERENCES users(id) ON DELETE SET NULL,
    context JSONB DEFAULT '{}',
    source_file VARCHAR(255),
    line_number INTEGER,
    resolved BOOLEAN DEFAULT FALSE,
    resolved_at TIMESTAMPTZ,
    resolved_by BIGINT REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_error_logs_error_type ON error_logs(error_type);
CREATE INDEX IF NOT EXISTS idx_error_logs_created_at ON error_logs(created_at);
CREATE INDEX IF NOT EXISTS idx_error_logs_request_id ON error_logs(request_id);
CREATE INDEX IF NOT EXISTS idx_error_logs_resolved ON error_logs(resolved);

-- ============================================
-- 4. Create audit_logs table
-- ============================================
CREATE TABLE IF NOT EXISTS audit_logs (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT REFERENCES users(id) ON DELETE SET NULL,
    admin_id BIGINT REFERENCES admins(id) ON DELETE SET NULL,
    action VARCHAR(50) NOT NULL CHECK (action IN (
        'user_login', 'user_logout', 'user_register', 'user_update',
        'wallet_create', 'wallet_update', 'withdraw', 'deposit',
        'order_create', 'order_cancel', 'order_complete', 'order_refund',
        'resource_create', 'resource_update', 'resource_delete',
        'permission_change', 'role_change', 'config_update',
        'api_key_create', 'api_key_revoke', 'password_change'
    )),
    resource_type VARCHAR(50) NOT NULL CHECK (resource_type IN ('user', 'admin', 'order', 'wallet', 'resource', 'config', 'api_key', 'system')),
    resource_id BIGINT,
    old_value JSONB,
    new_value JSONB,
    change_summary TEXT,
    ip_address INET,
    user_agent TEXT,
    verified_by VARCHAR(20) CHECK (verified_by IN ('2fa', 'email', 'sms', 'password', 'api_key')),
    risk_level VARCHAR(20) DEFAULT 'low' CHECK (risk_level IN ('low', 'medium', 'high', 'critical')),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_audit_logs_user_id ON audit_logs(user_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_admin_id ON audit_logs(admin_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_action ON audit_logs(action);
CREATE INDEX IF NOT EXISTS idx_audit_logs_resource ON audit_logs(resource_type, resource_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_created_at ON audit_logs(created_at);
CREATE INDEX IF NOT EXISTS idx_audit_logs_risk_level ON audit_logs(risk_level);

-- ============================================
-- 5. Add log_configs for new modules
-- ============================================
INSERT INTO log_configs (config_key, config_value, config_type, level, description) VALUES
    ('log_login_enabled', 'true', 'bool', NULL, '启用登录日志记录'),
    ('log_login_level', 'info', 'string', 'info', '登录日志级别'),
    ('log_error_enabled', 'true', 'bool', NULL, '启用错误日志记录'),
    ('log_error_level', 'error', 'string', 'error', '错误日志级别'),
    ('log_audit_enabled', 'true', 'bool', NULL, '启用审计日志记录'),
    ('log_audit_level', 'warn', 'string', 'warn', '审计日志级别'),
    ('log_slow_login_threshold_ms', '3000', 'number', NULL, '慢登录告警阈值（毫秒）')
ON CONFLICT (config_key) DO NOTHING;
