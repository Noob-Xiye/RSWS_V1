-- 迁移：为 usdt_wallets 添加 (address, network) 唯一约束
-- 日期：2026-05-30
-- 目的：支持 ON CONFLICT (address, network) 的 UPSERT 操作

-- 1. 添加唯一约束
ALTER TABLE IF EXISTS usdt_wallets 
ADD CONSTRAINT uk_usdt_wallets_address_network UNIQUE (address, network);

-- 2. 验证约束已添加
-- SELECT indexname, indexdef 
-- FROM pg_indexes 
-- WHERE tablename = 'usdt_wallets' 
--   AND indexname = 'uk_usdt_wallets_address_network';
