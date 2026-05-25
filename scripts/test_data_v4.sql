-- 测试数据（与实际 schema 完全对齐）
-- 重置序列
SELECT setval('admins_id_seq', 1, false);
SELECT setval('users_id_seq', 1, false);
SELECT setval('resources_id_seq', 1, false);
SELECT setval('categories_id_seq', 1, false);
SELECT setval('orders_id_seq', 1, false);
SELECT setval('menu_items_id_seq', 1, false);

-- 管理员账号（密码 Admin123，有效 Argon2 hash）
INSERT INTO admins (id, email, password_hash, username, is_active, role, created_at, updated_at)
VALUES (1, 'admin@rsws.com', '$v=19$m=65536,t=3,p=4$jrFo5uujiEMDIuhgktMU8g$SpadAPnq1ZUsXDLsZxt728yXvMvPzzBsIEq1B3xJQHk', '超级管理员', true, 'super_admin', NOW(), NOW())
ON CONFLICT (id) DO NOTHING;

-- 普通用户（密码 User1234）
INSERT INTO users (id, email, password_hash, username, is_active, created_at, updated_at)
VALUES
  (1, 'user1@rsws.com', '$v=19$m=65536,t=3,p=4$gpDEpXr3LBIg1IIz4ITJVg$0YT6Qwu45/OvDufVeo96bcWniep4rRlaGsZHft++Lr0', '用户1', true, NOW(), NOW()),
  (2, 'user2@rsws.com', '$v=19$m=65536,t=3,p=4$gpDEpXr3LBIg1IIz4ITJVg$0YT6Qwu45/OvDufVeo96bcWniep4rRlaGsZHft++Lr0', '用户2', true, NOW(), NOW()),
  (3, 'user3@rsws.com', '$v=19$m=65536,t=3,p=4$gpDEpXr3LBIg1IIz4ITJVg$0YT6Qwu45/OvDufVeo96bcWniep4rRlaGsZHft++Lr0', '用户3', true, NOW(), NOW())
ON CONFLICT (id) DO NOTHING;

-- 分类
INSERT INTO categories (id, name, slug, parent_id, "path", sort_order, is_active, created_at)
VALUES
  (1, '软件开发', 'software-dev', NULL, '1', 1, true, NOW()),
  (2, 'API 接口', 'api', NULL, '2', 2, true, NOW()),
  (3, '游戏模组', 'game-mods', NULL, '3', 3, true, NOW()),
  (4, 'Python 脚本', 'python', 1, '1/4', 4, true, NOW())
ON CONFLICT (id) DO NOTHING;

-- 资源（owner_type / provider_id）
INSERT INTO resources (id, title, description, price, category_id, owner_type, provider_id, file_url, is_active, created_at, updated_at)
VALUES
  (1, 'Python 爬虫合集', '包含 10 个实战爬虫项目', 29.99, 4, 'admin', 1, 'https://example.com/dl/py.zip', true, NOW(), NOW()),
  (2, 'React Admin 模板', '开箱即用的后台管理模板', 49.99, 2, 'admin', 1, 'https://example.com/dl/react.zip', true, NOW(), NOW()),
  (3, 'Minecraft 光影包', 'SEUS PTGI HRR 3.0', 0.00, 3, 'user', 1, 'https://example.com/dl/mc.zip', true, NOW(), NOW())
ON CONFLICT (id) DO NOTHING;

-- 菜单（列名：route_path, route_name, permission）
INSERT INTO menu_items (id, parent_id, title, icon, route_path, route_name, permission, sort_order, is_visible, created_at, updated_at)
VALUES
  (1, NULL, '数据概览', 'DataAnalysis', '/dashboard', 'dashboard', NULL, 1, true, NOW(), NOW()),
  (2, NULL, '用户管理', 'User', '/user', NULL, NULL, 2, true, NOW(), NOW()),
  (3, 2, '用户账号', NULL, '/users', 'users', NULL, 1, true, NOW(), NOW()),
  (4, 2, '用户资源', NULL, '/user-resources', 'user-resources', NULL, 2, true, NOW(), NOW()),
  (5, 2, '用户订单', NULL, '/user-orders', 'user-orders', NULL, 3, true, NOW(), NOW()),
  (6, NULL, '管理员管理', 'Setting', '/admin', NULL, NULL, 3, true, NOW(), NOW()),
  (7, 6, '管理员账号', NULL, '/admins', 'admins', NULL, 1, true, NOW(), NOW()),
  (8, 6, '平台资源', NULL, '/platform-resources', 'platform-resources', NULL, 2, true, NOW(), NOW()),
  (9, 6, '平台订单', NULL, '/platform-orders', 'platform-orders', NULL, 3, true, NOW(), NOW()),
  (10, NULL, '系统设置', 'Setting', '/system', NULL, NULL, 4, true, NOW(), NOW()),
  (11, 10, '邮件配置', NULL, '/email-config', 'email-config', NULL, 1, true, NOW(), NOW()),
  (12, 10, '日志管理', NULL, '/logs', 'logs', NULL, 2, true, NOW(), NOW()),
  (13, 10, '支付配置', NULL, '/payment-config', 'payment-config', NULL, 3, true, NOW(), NOW()),
  (14, 10, '系统设置', NULL, '/system-config', 'system-config', NULL, 4, true, NOW(), NOW())
ON CONFLICT (id) DO NOTHING;