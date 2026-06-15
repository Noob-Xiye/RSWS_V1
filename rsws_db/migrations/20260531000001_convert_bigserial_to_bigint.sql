-- 将所有表的 BIGSERIAL 改为 BIGINT（移除自增属性）
-- 原因：应用层使用 snowflake::next_id() 生成ID，数据库不应自增
-- 日期：2026-05-31

-- ========== 处理函数：删除序列并修改列类型 ==========
-- PostgreSQL 的 BIGSERIAL 会创建序列，需要显式删除

-- ========== users ==========
ALTER TABLE users ALTER COLUMN id DROP DEFAULT;
DROP SEQUENCE IF EXISTS users_id_seq CASCADE;

-- ========== admins ==========
ALTER TABLE admins ALTER COLUMN id DROP DEFAULT;
DROP SEQUENCE IF EXISTS admins_id_seq CASCADE;

-- ========== categories ==========
ALTER TABLE categories ALTER COLUMN id DROP DEFAULT;
DROP SEQUENCE IF EXISTS categories_id_seq CASCADE;

-- ========== resources ==========
ALTER TABLE resources ALTER COLUMN id DROP DEFAULT;
DROP SEQUENCE IF EXISTS resources_id_seq CASCADE;

-- ========== orders ==========
ALTER TABLE orders ALTER COLUMN id DROP DEFAULT;
DROP SEQUENCE IF EXISTS orders_id_seq CASCADE;

-- ========== payment_transactions ==========
ALTER TABLE payment_transactions ALTER COLUMN id DROP DEFAULT;
DROP SEQUENCE IF EXISTS payment_transactions_id_seq CASCADE;

-- ========== user_payment_configs ==========
ALTER TABLE user_payment_configs ALTER COLUMN id DROP DEFAULT;
DROP SEQUENCE IF EXISTS user_payment_configs_id_seq CASCADE;

-- ========== system_configs ==========
ALTER TABLE system_configs ALTER COLUMN id DROP DEFAULT;
DROP SEQUENCE IF EXISTS system_configs_id_seq CASCADE;

-- ========== paypal_configs ==========
ALTER TABLE paypal_configs ALTER COLUMN id DROP DEFAULT;
DROP SEQUENCE IF EXISTS paypal_configs_id_seq CASCADE;

-- ========== menu_items ==========
ALTER TABLE menu_items ALTER COLUMN id DROP DEFAULT;
DROP SEQUENCE IF EXISTS menu_items_id_seq CASCADE;

-- ========== admin_operation_logs ==========
ALTER TABLE admin_operation_logs ALTER COLUMN id DROP DEFAULT;
DROP SEQUENCE IF EXISTS admin_operation_logs_id_seq CASCADE;

-- ========== usdt_transactions ==========
ALTER TABLE usdt_transactions ALTER COLUMN id DROP DEFAULT;
DROP SEQUENCE IF EXISTS usdt_transactions_id_seq CASCADE;

-- ========== commission_rules ==========
ALTER TABLE commission_rules ALTER COLUMN id DROP DEFAULT;
DROP SEQUENCE IF EXISTS commission_rules_id_seq CASCADE;

-- ========== commission_records ==========
ALTER TABLE commission_records ALTER COLUMN id DROP DEFAULT;
DROP SEQUENCE IF EXISTS commission_records_id_seq CASCADE;

-- ========== usdt_wallets (如果使用了 BIGSERIAL) ==========
-- 检查 rsws_db/migrations 目录，确认是否有此表
-- ALTER TABLE usdt_wallets ALTER COLUMN id DROP DEFAULT;
-- DROP SEQUENCE IF EXISTS usdt_wallets_id_seq CASCADE;
