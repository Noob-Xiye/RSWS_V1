import sys

# Argon2 hash for "Admin123"
pw = "$argon2id$v=19$m=65536,t=3,p=4$pN42zqJvzXCJEOe819H13Q$3ISKaZ6JIllQGagrSenn9akFnNy0Yy+k75Hju1HHk4c"
# Same hash for all test users (password: Admin123)
pw_user = pw

lines = []
lines.append("-- RSWS 测试数据（修正版）")
lines.append("")

# admins
lines.append("-- ========== 管理员 ==========")
lines.append("INSERT INTO admins (id, email, password_hash, username, is_active, role, permissions, created_at, updated_at)")
lines.append(f"VALUES (1, 'admin@rsws.com', '{pw}', '超级管理员', true, 'super_admin', '[\"*\"]', NOW(), NOW())")
lines.append("ON CONFLICT (id) DO NOTHING;")
lines.append("SELECT setval('admins_id_seq', COALESCE((SELECT MAX(id) FROM admins), 0) + 1, false);")
lines.append("")

# users
lines.append("-- ========== 普通用户 ==========")
lines.append("INSERT INTO users (id, email, password_hash, username, is_active, email_verified, created_at, updated_at)")
lines.append(f"VALUES")
lines.append(f"  (1, 'user1@test.com', '{pw_user}', '测试用户1', true, true, NOW(), NOW()),")
lines.append(f"  (2, 'user2@test.com', '{pw_user}', '测试用户2', true, true, NOW(), NOW()),")
lines.append(f"  (3, 'seller@test.com', '{pw_user}', '测试卖家', true, true, NOW(), NOW())")
lines.append("ON CONFLICT (id) DO NOTHING;")
lines.append("SELECT setval('users_id_seq', COALESCE((SELECT MAX(id) FROM users), 0) + 1, false);")
lines.append("")

# categories
lines.append("-- ========== 分类 ==========")
lines.append("INSERT INTO categories (id, name, slug, description, parent_id, path, sort_order, is_active, created_at, updated_at)")
lines.append("VALUES")
lines.append("  (1, '开发工具', 'dev-tools', '开发相关工具软件', NULL, '1', 10, true, NOW(), NOW()),")
lines.append("  (2, '办公软件', 'office', '办公效率类软件', NULL, '2', 20, true, NOW(), NOW()),")
lines.append("  (3, '设计素材', 'design', '设计图片、模板、素材', NULL, '3', 30, true, NOW(), NOW()),")
lines.append("  (4, 'IDE', 'ide', '集成开发环境', 1, '1.4', 11, true, NOW(), NOW()),")
lines.append("  (5, '代码编辑器', 'code-editor', '轻量级代码编辑器', 1, '1.5', 12, true, NOW(), NOW()),")
lines.append("  (6, 'AI 辅助', 'ai-tools', 'AI 辅助开发工具', 1, '1.6', 13, true, NOW(), NOW()),")
lines.append("  (7, '文档模板', 'doc-templates', 'Word/Excel/PPT 模板', 2, '2.7', 21, true, NOW(), NOW()),")
lines.append("  (8, 'PPT 模板', 'ppt-templates', '演示文稿模板', 2, '2.8', 22, true, NOW(), NOW()),")
lines.append("  (9, 'UI 素材', 'ui-assets', 'UI 界面素材包', 3, '3.9', 31, true, NOW(), NOW()),")
lines.append("  (10, '图标库', 'icon-library', '各类图标资源', 3, '3.10', 32, true, NOW(), NOW())")
lines.append("ON CONFLICT (id) DO NOTHING;")
lines.append("SELECT setval('categories_id_seq', COALESCE((SELECT MAX(id) FROM categories), 0) + 1, false);")
lines.append("")

# resources
lines.append("-- ========== 资源 ==========")
lines.append("INSERT INTO resources (id, title, description, price, category_id, is_active,")
lines.append("  detail_description, specifications, download_count, owner_type, supported_os, commission_rate,")
lines.append("  created_at, updated_at)")
lines.append("VALUES")
lines.append("  (1, 'VS Code 便携版', 'Microsoft VS Code 便携版，免安装即用', 0.00, 5, true,")
lines.append("   'VS Code 便携版详细说明', '{\"version\": \"1.90.0\", \"size\": \"95MB\"}'::jsonb, 1024, 'admin', '[\"Windows\",\"macOS\",\"Linux\"]'::jsonb, 0.00,")
lines.append("   NOW(), NOW()),")
lines.append("  (2, 'IntelliJ IDEA 最新版', 'JetBrains IntelliJ IDEA Ultimate 最新版本', 199.00, 4, true,")
lines.append("   'IntelliJ IDEA 详细说明', '{\"version\": \"2024.1\", \"size\": \"850MB\"}'::jsonb, 512, 'admin', '[\"Windows\",\"macOS\",\"Linux\"]'::jsonb, 0.00,")
lines.append("   NOW(), NOW()),")
lines.append("  (3, 'Office 365 激活工具', 'Office 365 全版本激活工具', 29.99, 2, true,")
lines.append("   '激活工具使用说明', '{\"version\": \"v3.2\", \"size\": \"15MB\"}'::jsonb, 2048, 'admin', '[\"Windows\"]'::jsonb, 0.00,")
lines.append("   NOW(), NOW()),")
lines.append("  (4, 'AI 代码助手 Pro', '基于 AI 的代码补全和生成工具', 49.99, 6, true,")
lines.append("   'AI 代码助手详细说明', '{\"version\": \"2.5\", \"size\": \"120MB\"}'::jsonb, 768, 'admin', '[\"Windows\",\"macOS\"]'::jsonb, 0.00,")
lines.append("   NOW(), NOW()),")
lines.append("  (5, '个人开发工具箱', '我自己整理的开发常用工具合集', 9.99, 1, true,")
lines.append("   '工具箱详细说明', '{\"version\": \"1.0\", \"size\": \"200MB\"}'::jsonb, 256, 'user', '[\"Windows\"]'::jsonb, 0.05,")
lines.append("   NOW(), NOW()),")
lines.append("  (6, '高级 PPT 模板包', '100+ 高级商务 PPT 模板', 19.99, 8, true,")
lines.append("   '模板包详细说明', '{\"count\": \"100+\", \"size\": \"500MB\"}'::jsonb, 1024, 'user', '[\"Windows\",\"macOS\"]'::jsonb, 0.10,")
lines.append("   NOW(), NOW()),")
lines.append("  (7, 'UI 设计系统素材', '完整 UI 设计系统 Figma 素材', 39.99, 9, true,")
lines.append("   '设计系统详细说明', '{\"format\": \"Figma\", \"size\": \"300MB\"}'::jsonb, 128, 'user', '[\"macOS\",\"Windows\"]'::jsonb, 0.08,")
lines.append("   NOW(), NOW())")
lines.append("ON CONFLICT (id) DO NOTHING;")
lines.append("SELECT setval('resources_id_seq', COALESCE((SELECT MAX(id) FROM resources), 0) + 1, false);")
lines.append("")

# system_configs
lines.append("-- ========== 系统配置 ==========")
lines.append("INSERT INTO system_configs (config_key, config_value, config_type, description, is_public, created_at, updated_at)")
lines.append("VALUES")
lines.append("  ('system.site_name', 'RSWS 虚拟资源交易平台', 'string', '站点名称', true, NOW(), NOW()),")
lines.append("  ('system.site_url', 'http://localhost:4000', 'string', '站点URL', true, NOW(), NOW()),")
lines.append("  ('system.default_currency', 'USD', 'string', '默认货币', true, NOW(), NOW()),")
lines.append("  ('system.contact_email', 'admin@rsws.com', 'string', '联系邮箱', true, NOW(), NOW()),")
lines.append("  ('payment.min_usdt_amount', '1', 'number', '最小 USDT 金额', true, NOW(), NOW()),")
lines.append("  ('payment.max_usdt_amount', '10000', 'number', '最大 USDT 金额', true, NOW(), NOW()),")
lines.append("  ('payment.paypal.fee_rate', '0.0349', 'number', 'PayPal 手续费率', false, NOW(), NOW()),")
lines.append("  ('api_key.session_expire_days', '7', 'number', 'API Key 有效期(天)', false, NOW(), NOW()),")
lines.append("  ('log.level', 'info', 'string', '日志级别', false, NOW(), NOW()),")
lines.append("  ('commission.default_rate', '0.05', 'number', '默认佣金比例', false, NOW(), NOW())")
lines.append("ON CONFLICT (config_key) DO NOTHING;")
lines.append("")

# menu_items
lines.append("-- ========== 菜单项 ==========")
lines.append("INSERT INTO menu_items (id, parent_id, title, icon, route_path, route_name, permission, sort_order, is_visible, created_at, updated_at)")
lines.append("VALUES")
lines.append("  (1, NULL, '数据概览', 'DataBoard', '/dashboard', 'Dashboard', NULL, 10, true, NOW(), NOW()),")
lines.append("  (2, NULL, '用户管理', 'User', NULL, NULL, NULL, 20, true, NOW(), NOW()),")
lines.append("  (3, 2, '用户账号', 'UserAccount', '/users', 'UserList', 'user:read', 21, true, NOW(), NOW()),")
lines.append("  (4, 2, '用户 API Key', 'Key', '/user-api-keys', 'UserApiKey', 'user:read', 22, true, NOW(), NOW()),")
lines.append("  (5, 2, '用户资源', 'Resource', '/user-resources', 'UserResource', 'resource:read', 23, true, NOW(), NOW()),")
lines.append("  (6, 2, '用户订单', 'Order', '/user-orders', 'UserOrder', 'order:read', 24, true, NOW(), NOW()),")
lines.append("  (7, NULL, '管理员管理', 'Admin', NULL, NULL, NULL, 30, true, NOW(), NOW()),")
lines.append("  (8, 7, '管理员账号', 'AdminAccount', '/admins', 'AdminList', 'admin:read', 31, true, NOW(), NOW()),")
lines.append("  (9, 7, '管理员 API Key', 'Key', '/admin-api-keys', 'AdminApiKey', 'admin:read', 32, true, NOW(), NOW()),")
lines.append("  (10, 7, '平台资源', 'Resource', '/admin-resources', 'AdminResource', 'resource:read', 33, true, NOW(), NOW()),")
lines.append("  (11, 7, '平台订单', 'Order', '/admin', 'AdminOrder', 'order:read', 34, true, NOW(), NOW()),")
lines.append("  (12, NULL, '系统设置', 'Setting', NULL, NULL, NULL, 40, true, NOW(), NOW()),")
lines.append("  (13, 12, '邮件配置', 'Mail', '/email-config', 'EmailConfig', 'system:manage', 41, true, NOW(), NOW()),")
lines.append("  (14, 12, '日志管理', 'Log', '/system-logs', 'SystemLog', 'system:manage', 42, true, NOW(), NOW()),")
lines.append("  (15, 12, '支付配置', 'Payment', '/payment-config', 'PaymentConfig', 'system:manage', 43, true, NOW(), NOW()),")
lines.append("  (16, 12, '系统设置', 'System', '/system-settings', 'SystemSettings', 'system:manage', 44, true, NOW(), NOW())")
lines.append("ON CONFLICT (id) DO NOTHING;")
lines.append("SELECT setval('menu_items_id_seq', COALESCE((SELECT MAX(id) FROM menu_items), 0) + 1, false);")

sql = "\n".join(lines) + "\n"

out = r"F:\GitRepo\RSWS_V1\scripts\test_data_full.sql"
with open(out, "w", encoding="utf-8") as f:
    f.write(sql)

print(f"OK: wrote {len(sql)} chars -> test_data_full.sql")
