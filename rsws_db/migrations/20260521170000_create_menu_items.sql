-- Migration: 20260521170000_create_menu_items.sql
-- 描述: 创建动态菜单表并插入初始数据

CREATE TABLE IF NOT EXISTS menu_items (
    id BIGSERIAL PRIMARY KEY,
    parent_id BIGINT REFERENCES menu_items(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,           -- 显示名称
    icon VARCHAR(100),                     -- 图标（element-plus icon）
    path VARCHAR(500),                     -- 路由路径
    component VARCHAR(255),                -- 前端组件路径
    permission VARCHAR(100),              -- 权限标识：admin:user:view
    sort_order INTEGER NOT NULL DEFAULT 0,
    is_hidden BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- 索引
CREATE INDEX IF NOT EXISTS idx_menu_items_parent_id ON menu_items(parent_id);
CREATE INDEX IF NOT EXISTS idx_menu_items_permission ON menu_items(permission);

COMMENT ON TABLE menu_items IS '动态菜单配置（支持多级菜单）';
COMMENT ON COLUMN menu_items.parent_id IS '父级菜单ID，NULL 表示顶级菜单';
COMMENT ON COLUMN menu_items.permission IS '权限标识，用于前端按钮级权限控制';

-- ============================================
-- 插入初始菜单数据
-- ============================================

-- 顶级菜单（parent_id = NULL）
INSERT INTO menu_items (parent_id, name, icon, path, component, permission, sort_order) VALUES
(NULL, '数据概览', 'Odometer', '/admin/dashboard', 'views/dashboard/index.vue', 'dashboard:view', 0),
(NULL, '用户管理', 'User', NULL, NULL, 'user:view', 1),
(NULL, '管理员管理', 'Setting', NULL, NULL, 'admin:view', 2),
(NULL, '系统设置', 'Tools', NULL, NULL, 'system:view', 3);

-- 用户管理子菜单（parent_id = 2）
INSERT INTO menu_items (parent_id, name, icon, path, component, permission, sort_order) VALUES
(2, '用户账号管理', 'UserFilled', '/admin/users', 'views/user/index.vue', 'user:view', 0),
(2, '用户 API Key', 'Key', '/admin/user-api-keys', 'views/user-api-key/index.vue', 'user:apikey:view', 1),
(2, '用户资源', 'Goods', '/admin/user-resources', 'views/user-resource/index.vue', 'user:resource:view', 2),
(2, '用户订单', 'List', '/admin/user-orders', 'views/user-order/index.vue', 'user:order:view', 3);

-- 管理员管理子菜单（parent_id = 3）
INSERT INTO menu_items (parent_id, name, icon, path, component, permission, sort_order) VALUES
(3, '管理员账号', 'Avatar', '/admin/admins', 'views/admin/index.vue', 'admin:view', 0),
(3, '管理员 API Key', 'Key', '/admin/admin-api-keys', 'views/admin-api-key/index.vue', 'admin:apikey:view', 1),
(3, '平台资源', 'Box', '/admin/platform-resources', 'views/platform-resource/index.vue', 'platform:resource:view', 2),
(3, '平台订单', 'Tickets', '/admin/platform-orders', 'views/platform-order/index.vue', 'platform:order:view', 3);

-- 系统设置子菜单（parent_id = 4）
INSERT INTO menu_items (parent_id, name, icon, path, component, permission, sort_order) VALUES
(4, '邮件配置', 'Message', '/admin/email-config', 'views/email-config/index.vue', 'system:email:view', 0),
(4, '日志管理', 'Notebook', '/admin/logs', 'views/log/index.vue', 'system:log:view', 1),
(4, '支付配置', 'Money', '/admin/payment-config', 'views/payment-config/index.vue', 'system:payment:view', 2),
(4, '系统设置', 'Setting', '/admin/settings', 'views/settings/index.vue', 'system:settings:view', 3);
