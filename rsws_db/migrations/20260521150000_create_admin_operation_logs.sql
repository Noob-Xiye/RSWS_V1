-- Migration: 20260521150000_create_admin_operation_logs.sql
-- 描述: 创建管理员操作日志表

CREATE TABLE IF NOT EXISTS admin_operation_logs (
    id BIGSERIAL PRIMARY KEY,
    admin_id BIGINT REFERENCES admins(id) ON DELETE SET NULL,
    module VARCHAR(50) NOT NULL,           -- 模块：users/resources/orders/settings
    action VARCHAR(50) NOT NULL,           -- 操作：create/update/delete/view
    resource_id BIGINT,                    -- 关联的资源ID（可选）
    resource_type VARCHAR(50),             -- 资源类型：user/resource/order/api_key
    ip_address INET,                      -- 操作IP
    user_agent TEXT,                       -- User Agent
    request_method VARCHAR(10),            -- GET/POST/PUT/DELETE
    request_path VARCHAR(500),             -- 请求路径
    request_params JSONB,                  -- 请求参数（脱敏）
    response_code INTEGER,                 -- 响应状态码
    execution_time_ms INTEGER,             -- 执行时间（毫秒）
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- 索引
CREATE INDEX IF NOT EXISTS idx_operation_logs_admin_id ON admin_operation_logs(admin_id);
CREATE INDEX IF NOT EXISTS idx_operation_logs_module ON admin_operation_logs(module);
CREATE INDEX IF NOT EXISTS idx_operation_logs_created_at ON admin_operation_logs(created_at DESC);

-- 分区优化（可选，数据量大时启用 BRIN 索引）
-- CREATE INDEX idx_operation_logs_created_at_month ON admin_operation_logs USING BRIN (created_at);

COMMENT ON TABLE admin_operation_logs IS '管理员操作日志（审计用）';
COMMENT ON COLUMN admin_operation_logs.admin_id IS '操作的管理员ID';
COMMENT ON COLUMN admin_operation_logs.module IS '操作的模块名称';
COMMENT ON COLUMN admin_operation_logs.action IS '操作类型：create/update/delete/view';
COMMENT ON COLUMN admin_operation_logs.request_params IS '请求参数（已脱敏，不含密码/API Key）';
