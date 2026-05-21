-- 添加 nickname 列到 users 表
-- 迁移版本: 20260521103000

ALTER TABLE users ADD COLUMN IF NOT EXISTS nickname VARCHAR(100);

-- 用 username 的值初始化现有行的 nickname
UPDATE users SET nickname = username WHERE nickname IS NULL;

-- 为现有管理员账号设置中文昵称（可选，手动执行）
-- UPDATE users SET nickname = '管理员' WHERE email = 'admin@rsws.com';
