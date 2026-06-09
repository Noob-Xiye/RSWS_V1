-- ========================================
-- RSWS_V1 增量迁移
-- 功能: log_configs 表增加 level 字段和 config_type 字段
-- 时间: 2026-06-08
-- ========================================

-- log_configs 表增加 level 和 config_type 字段
ALTER TABLE log_configs
    ADD COLUMN IF NOT EXISTS config_type VARCHAR(50) DEFAULT 'string',
    ADD COLUMN IF NOT EXISTS level VARCHAR(20) DEFAULT 'info';

-- 插入默认日志配置（如果不存在）
INSERT INTO log_configs (config_key, config_value, config_type, description, level, is_active)
VALUES
    ('log.level', 'info', 'string', '全局日志级别: trace, debug, info, warn, error', 'info', true),
    ('log.request', 'true', 'bool', '是否记录请求日志', 'info', true),
    ('log.admin', 'true', 'bool', '是否记录管理操作日志', 'info', true),
    ('log.payment', 'true', 'bool', '是否记录支付日志', 'info', true),
    ('log.error', 'true', 'bool', '是否记录错误日志', 'error', true),
    ('log.module.oss', 'info', 'string', 'OSS 模块日志级别', 'info', true),
    ('log.module.payment', 'info', 'string', '支付模块日志级别', 'info', true),
    ('log.module.webhook', 'info', 'string', 'Webhook 模块日志级别', 'info', true),
    ('log.module.auth', 'warn', 'string', '认证模块日志级别（默认警告以上）', 'warn', true)
ON CONFLICT (config_key) DO NOTHING;

-- 为 system_logs 的 log_level 和 module 列添加索引（加快查询）
CREATE INDEX IF NOT EXISTS idx_system_logs_level ON system_logs(log_level);
CREATE INDEX IF NOT EXISTS idx_system_logs_module ON system_logs(module);
CREATE INDEX IF NOT EXISTS idx_system_logs_created_at ON system_logs(created_at DESC);
