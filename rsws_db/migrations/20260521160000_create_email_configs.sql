-- Migration: 20260521160000_create_email_configs.sql
-- 描述: 创建邮件配置表

CREATE TABLE IF NOT EXISTS email_configs (
    id BIGSERIAL PRIMARY KEY,
    smtp_host VARCHAR(255) NOT NULL,
    smtp_port INTEGER NOT NULL DEFAULT 587,
    smtp_username VARCHAR(255) NOT NULL,
    smtp_password_encrypted TEXT NOT NULL,  -- AES-256 加密存储
    sender_email VARCHAR(255) NOT NULL,
    sender_name VARCHAR(100),
    use_tls BOOLEAN NOT NULL DEFAULT TRUE,
    use_ssl BOOLEAN NOT NULL DEFAULT FALSE,
    is_active BOOLEAN NOT NULL DEFAULT FALSE,
    verification_template TEXT,             -- 验证码邮件模板（变量：{code}, {expire_minutes}）
    order_notification_template TEXT,        -- 订单通知模板
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- 索引
CREATE INDEX IF NOT EXISTS idx_email_configs_is_active ON email_configs(is_active);

-- 注释
COMMENT ON TABLE email_configs IS '邮件服务配置（SMTP）';
COMMENT ON COLUMN email_configs.smtp_password_encrypted IS 'AES-256 加密后的密码';
COMMENT ON COLUMN email_configs.verification_template IS '验证码邮件模板，支持变量：{code}, {expire_minutes}';
COMMENT ON COLUMN email_configs.order_notification_template IS '订单通知邮件模板';
