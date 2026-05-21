-- 删除 user_sessions 和 admin_api_keys 表 (API Key 改为纯 Redis 存储)
-- 日期: 2026-05-19

-- 删除 user_sessions 表
DROP TABLE IF EXISTS user_sessions CASCADE;

-- 删除 admin_api_keys 表
DROP TABLE IF EXISTS admin_api_keys CASCADE;
