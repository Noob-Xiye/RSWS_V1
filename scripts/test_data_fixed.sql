-- =======================================
-- RSWS 测试数据注入脚本（修正版）
-- =======================================

-- ========== 管理员账号 ==========
INSERT INTO admins (id, email, password_hash, username, is_active, role, permissions, created_at, updated_at)
VALUES
  (1, 'admin@rsws.com',
   '$argon2id$v=19$m=65536,t=3,p=4$pN42zqJvzXCJEOe819H13Q$3ISKaZ6JIllQGagrSenn9akFnNy0Yy+k75Hju1HHk4c',
   '超级管理员', true, 'super_admin', '["*"]', NOW(), NOW())
ON CONFLICT (id) DO NOTHING;

SELECT setval('admins_id_seq', COALESCE((SELECT MAX(id) FROM admins), 0) + 1, false);

