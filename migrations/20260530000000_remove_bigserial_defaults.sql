-- 迁移：移除所有表的 BIGSERIAL 默认值，统一使用应用层雪花ID生成
-- 日期：2026-05-30
-- 影响：24张表，移除 id 列的 nextval(...) 默认值

-- 说明：
-- 1. 已有数据的 id=1,2,3... 保留不变
-- 2. 新插入的数据必须由应用层通过 snowflake::next_id() 生成
-- 3. 序列本身不删除（无害，只是不再使用）

-- ============ 系统相关表 ============
ALTER TABLE IF EXISTS admin_operation_logs ALTER COLUMN id DROP DEFAULT;
ALTER TABLE IF EXISTS error_logs ALTER COLUMN id DROP DEFAULT;
ALTER TABLE IF EXISTS request_logs ALTER COLUMN id DROP DEFAULT;
ALTER TABLE IF EXISTS system_logs ALTER COLUMN id DROP DEFAULT;
ALTER TABLE IF EXISTS system_configs ALTER COLUMN id DROP DEFAULT;

-- ============ 管理员相关表 ============
ALTER TABLE IF EXISTS admins ALTER COLUMN id DROP DEFAULT;
ALTER TABLE IF EXISTS admin_api_keys ALTER COLUMN id DROP DEFAULT;

-- ============ 用户相关表 ============
ALTER TABLE IF EXISTS users ALTER COLUMN id DROP DEFAULT;
ALTER TABLE IF EXISTS user_api_keys ALTER COLUMN id DROP DEFAULT;
ALTER TABLE IF EXISTS user_payment_configs ALTER COLUMN id DROP DEFAULT;

-- ============ 分类与资源相关表 ============
ALTER TABLE IF EXISTS categories ALTER COLUMN id DROP DEFAULT;
ALTER TABLE IF EXISTS resources ALTER COLUMN id DROP DEFAULT;
ALTER TABLE IF EXISTS resource_views ALTER COLUMN id DROP DEFAULT;
ALTER TABLE IF EXISTS resource_downloads ALTER COLUMN id DROP DEFAULT;

-- ============ 订单与支付相关表 ============
ALTER TABLE IF EXISTS orders ALTER COLUMN id DROP DEFAULT;
ALTER TABLE IF EXISTS order_items ALTER COLUMN id DROP DEFAULT;
ALTER TABLE IF EXISTS payment_methods ALTER COLUMN id DROP DEFAULT;
ALTER TABLE IF EXISTS payment_transactions ALTER COLUMN id DROP DEFAULT;
ALTER TABLE IF EXISTS payment_logs ALTER COLUMN id DROP DEFAULT;
ALTER TABLE IF EXISTS commissions ALTER COLUMN id DROP DEFAULT;
ALTER TABLE IF EXISTS commission_records ALTER COLUMN id DROP DEFAULT;
ALTER TABLE IF EXISTS commission_rules ALTER COLUMN id DROP DEFAULT;

-- ============ 配置相关表 ============
ALTER TABLE IF EXISTS paypal_configs ALTER COLUMN id DROP DEFAULT;
ALTER TABLE IF EXISTS blockchain_configs ALTER COLUMN id DROP DEFAULT;
ALTER TABLE IF EXISTS usdt_wallets ALTER COLUMN id DROP DEFAULT;
ALTER TABLE IF EXISTS usdt_transactions ALTER COLUMN id DROP DEFAULT;
ALTER TABLE IF EXISTS usdt_listen_configs ALTER COLUMN id DROP DEFAULT;
ALTER TABLE IF EXISTS email_configs ALTER COLUMN id DROP DEFAULT;
ALTER TABLE IF EXISTS log_configs ALTER COLUMN id DROP DEFAULT;
ALTER TABLE IF EXISTS menu_items ALTER COLUMN id DROP DEFAULT;

-- ============ 验证：确认所有 id 列已移除默认值 ============
-- 查询应该返回 0 行（表示所有表都已正确处理）
-- SELECT tablename, column_default 
-- FROM pg_tables t
-- JOIN information_schema.columns c ON t.tablename = c.table_name
-- WHERE c.column_name = 'id' 
--   AND t.schemaname = 'public'
--   AND c.column_default LIKE '%nextval%';
