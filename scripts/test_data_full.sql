-- RSWS 测试数据（修正版）

-- ========== 管理员 ==========
INSERT INTO admins (id, email, password_hash, username, is_active, role, permissions, created_at, updated_at)
VALUES (1, 'admin@rsws.com', '$argon2id$v=19$m=65536,t=3,p=4$pN42zqJvzXCJEOe819H13Q$3ISKaZ6JIllQGagrSenn9akFnNy0Yy+k75Hju1HHk4c', '超级管理员', true, 'super_admin', '["*"]', NOW(), NOW())
ON CONFLICT (id) DO NOTHING;
SELECT setval('admins_id_seq', COALESCE((SELECT MAX(id) FROM admins), 0) + 1, false);

-- ========== 普通用户 ==========
INSERT INTO users (id, email, password_hash, username, is_active, email_verified, created_at, updated_at)
VALUES
  (1, 'user1@test.com', '$argon2id$v=19$m=65536,t=3,p=4$pN42zqJvzXCJEOe819H13Q$3ISKaZ6JIllQGagrSenn9akFnNy0Yy+k75Hju1HHk4c', '测试用户1', true, true, NOW(), NOW()),
  (2, 'user2@test.com', '$argon2id$v=19$m=65536,t=3,p=4$pN42zqJvzXCJEOe819H13Q$3ISKaZ6JIllQGagrSenn9akFnNy0Yy+k75Hju1HHk4c', '测试用户2', true, true, NOW(), NOW()),
  (3, 'seller@test.com', '$argon2id$v=19$m=65536,t=3,p=4$pN42zqJvzXCJEOe819H13Q$3ISKaZ6JIllQGagrSenn9akFnNy0Yy+k75Hju1HHk4c', '测试卖家', true, true, NOW(), NOW())
ON CONFLICT (id) DO NOTHING;
SELECT setval('users_id_seq', COALESCE((SELECT MAX(id) FROM users), 0) + 1, false);

-- ========== 分类 ==========
INSERT INTO categories (id, name, slug, description, parent_id, path, sort_order, is_active, created_at, updated_at)
VALUES
  (1, '开发工具', 'dev-tools', '开发相关工具软件', NULL, '1', 10, true, NOW(), NOW()),
  (2, '办公软件', 'office', '办公效率类软件', NULL, '2', 20, true, NOW(), NOW()),
  (3, '设计素材', 'design', '设计图片、模板、素材', NULL, '3', 30, true, NOW(), NOW()),
  (4, 'IDE', 'ide', '集成开发环境', 1, '1.4', 11, true, NOW(), NOW()),
  (5, '代码编辑器', 'code-editor', '轻量级代码编辑器', 1, '1.5', 12, true, NOW(), NOW()),
  (6, 'AI 辅助', 'ai-tools', 'AI 辅助开发工具', 1, '1.6', 13, true, NOW(), NOW()),
  (7, '文档模板', 'doc-templates', 'Word/Excel/PPT 模板', 2, '2.7', 21, true, NOW(), NOW()),
  (8, 'PPT 模板', 'ppt-templates', '演示文稿模板', 2, '2.8', 22, true, NOW(), NOW()),
  (9, 'UI 素材', 'ui-assets', 'UI 界面素材包', 3, '3.9', 31, true, NOW(), NOW()),
  (10, '图标库', 'icon-library', '各类图标资源', 3, '3.10', 32, true, NOW(), NOW())
ON CONFLICT (id) DO NOTHING;
SELECT setval('categories_id_seq', COALESCE((SELECT MAX(id) FROM categories), 0) + 1, false);

-- ========== 资源 ==========
INSERT INTO resources (id, title, description, price, category_id, is_active,
  detail_description, specifications, download_count, owner_type, supported_os, commission_rate,
  created_at, updated_at)
VALUES
  (1, 'VS Code 便携版', 'Microsoft VS Code 便携版，免安装即用', 0.00, 5, true,
   'VS Code 便携版详细说明', '{"version": "1.90.0", "size": "95MB"}'::jsonb, 1024, 'admin', '["Windows","macOS","Linux"]'::jsonb, 0.00,
   NOW(), NOW()),
  (2, 'IntelliJ IDEA 最新版', 'JetBrains IntelliJ IDEA Ultimate 最新版本', 199.00, 4, true,
   'IntelliJ IDEA 详细说明', '{"version": "2024.1", "size": "850MB"}'::jsonb, 512, 'admin', '["Windows","macOS","Linux"]'::jsonb, 0.00,
   NOW(), NOW()),
  (3, 'Office 365 激活工具', 'Office 365 全版本激活工具', 29.99, 2, true,
   '激活工具使用说明', '{"version": "v3.2", "size": "15MB"}'::jsonb, 2048, 'admin', '["Windows"]'::jsonb, 0.00,
   NOW(), NOW()),
  (4, 'AI 代码助手 Pro', '基于 AI 的代码补全和生成工具', 49.99, 6, true,
   'AI 代码助手详细说明', '{"version": "2.5", "size": "120MB"}'::jsonb, 768, 'admin', '["Windows","macOS"]'::jsonb, 0.00,
   NOW(), NOW()),
  (5, '个人开发工具箱', '我自己整理的开发常用工具合集', 9.99, 1, true,
   '工具箱详细说明', '{"version": "1.0", "size": "200MB"}'::jsonb, 256, 'user', '["Windows"]'::jsonb, 0.05,
   NOW(), NOW()),
  (6, '高级 PPT 模板包', '100+ 高级商务 PPT 模板', 19.99, 8, true,
   '模板包详细说明', '{"count": "100+", "size": "500MB"}'::jsonb, 1024, 'user', '["Windows","macOS"]'::jsonb, 0.10,
   NOW(), NOW()),
  (7, 'UI 设计系统素材', '完整 UI 设计系统 Figma 素材', 39.99, 9, true,
   '设计系统详细说明', '{"format": "Figma", "size": "300MB"}'::jsonb, 128, 'user', '["macOS","Windows"]'::jsonb, 0.08,
   NOW(), NOW())
ON CONFLICT (id) DO NOTHING;
SELECT setval('resources_id_seq', COALESCE((SELECT MAX(id) FROM resources), 0) + 1, false);

-- ========== 系统配置 ==========
INSERT INTO system_configs (config_key, config_value, config_type, description, is_public, created_at, updated_at)
VALUES
  ('system.site_name', 'RSWS 虚拟资源交易平台', 'string', '站点名称', true, NOW(), NOW()),
  ('system.site_url', 'http://localhost:4000', 'string', '站点URL', true, NOW(), NOW()),
  ('system.default_currency', 'USD', 'string', '默认货币', true, NOW(), NOW()),
  ('system.contact_email', 'admin@rsws.com', 'string', '联系邮箱', true, NOW(), NOW()),
  ('payment.min_usdt_amount', '1', 'number', '最小 USDT 金额', true, NOW(), NOW()),
  ('payment.max_usdt_amount', '10000', 'number', '最大 USDT 金额', true, NOW(), NOW()),
  ('payment.paypal.fee_rate', '0.0349', 'number', 'PayPal 手续费率', false, NOW(), NOW()),
  ('api_key.session_expire_days', '7', 'number', 'API Key 有效期(天)', false, NOW(), NOW()),
  ('log.level', 'info', 'string', '日志级别', false, NOW(), NOW()),
  ('commission.default_rate', '0.05', 'number', '默认佣金比例', false, NOW(), NOW())
ON CONFLICT (config_key) DO NOTHING;

-- ========== 菜单项 ==========
INSERT INTO menu_items (id, parent_id, title, icon, route_path, route_name, permission, sort_order, is_visible, created_at, updated_at)
VALUES
  (1, NULL, '数据概览', 'DataBoard', '/dashboard', 'Dashboard', NULL, 10, true, NOW(), NOW()),
  (2, NULL, '用户管理', 'User', NULL, NULL, NULL, 20, true, NOW(), NOW()),
  (3, 2, '用户账号', 'UserAccount', '/users', 'UserList', 'user:read', 21, true, NOW(), NOW()),
  (4, 2, '用户 API Key', 'Key', '/user-api-keys', 'UserApiKey', 'user:read', 22, true, NOW(), NOW()),
  (5, 2, '用户资源', 'Resource', '/user-resources', 'UserResource', 'resource:read', 23, true, NOW(), NOW()),
  (6, 2, '用户订单', 'Order', '/user-orders', 'UserOrder', 'order:read', 24, true, NOW(), NOW()),
  (7, NULL, '管理员管理', 'Admin', NULL, NULL, NULL, 30, true, NOW(), NOW()),
  (8, 7, '管理员账号', 'AdminAccount', '/admins', 'AdminList', 'admin:read', 31, true, NOW(), NOW()),
  (9, 7, '管理员 API Key', 'Key', '/admin-api-keys', 'AdminApiKey', 'admin:read', 32, true, NOW(), NOW()),
  (10, 7, '平台资源', 'Resource', '/admin-resources', 'AdminResource', 'resource:read', 33, true, NOW(), NOW()),
  (11, 7, '平台订单', 'Order', '/admin', 'AdminOrder', 'order:read', 34, true, NOW(), NOW()),
  (12, NULL, '系统设置', 'Setting', NULL, NULL, NULL, 40, true, NOW(), NOW()),
  (13, 12, '邮件配置', 'Mail', '/email-config', 'EmailConfig', 'system:manage', 41, true, NOW(), NOW()),
  (14, 12, '日志管理', 'Log', '/system-logs', 'SystemLog', 'system:manage', 42, true, NOW(), NOW()),
  (15, 12, '支付配置', 'Payment', '/payment-config', 'PaymentConfig', 'system:manage', 43, true, NOW(), NOW()),
  (16, 12, '系统设置', 'System', '/system-settings', 'SystemSettings', 'system:manage', 44, true, NOW(), NOW())
ON CONFLICT (id) DO NOTHING;
SELECT setval('menu_items_id_seq', COALESCE((SELECT MAX(id) FROM menu_items), 0) + 1, false);
