-- RSWS v0.1.1 USDT 监听系统补全
-- 依赖: usdt_transactions, resources, orders 表已存在

-- 1. 创建平台钱包表（USDT 收款地址）
CREATE TABLE IF NOT EXISTS usdt_wallets (
    id          BIGINT      PRIMARY KEY,  -- ID 由 Rust snowflake::next_id() 生成
    network     VARCHAR(20) NOT NULL,  -- 'tron' | 'ethereum'
    address     VARCHAR(64) NOT NULL,
    name        VARCHAR(100),
    is_active   BOOLEAN     NOT NULL DEFAULT false,
    total_received NUMERIC(30,8) NOT NULL DEFAULT 0,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT usdt_wallets_network_address_key UNIQUE (network, address)
);

-- 2. resources 表加钱包关联
ALTER TABLE resources ADD COLUMN IF NOT EXISTS wallet_id BIGINT
    REFERENCES usdt_wallets(id) ON DELETE SET NULL;

-- 3. orders 表加交易 hash 记录
ALTER TABLE orders ADD COLUMN IF NOT EXISTS transaction_id VARCHAR(128);

-- 4. usdt_transactions 表加区块号
ALTER TABLE usdt_transactions ADD COLUMN IF NOT EXISTS block_number BIGINT;

-- 5. 插入默认 TRON 钱包 + TronGrid 配置（Mock 模式）
INSERT INTO usdt_wallets (id, network, address, name, is_active, total_received)
VALUES (7300000000001, 'tron', 'T9ydSnpLxUeJFLvJsMgqMRkGZRBf7yNhH', 'Platform TRC20 Wallet', true, 0)
ON CONFLICT DO NOTHING;

-- 6. 插入 TronGrid 监听配置（is_active=false 暂时禁用，可通过 Admin API 激活）
INSERT INTO usdt_listen_configs (id, network, api_url, api_key, usdt_contract, poll_interval_seconds, min_confirmations, is_active)
VALUES (
    7300000000001,
    'tron',
    'https://api.trongrid.io',
    NULL,
    'TR7NHmqjeNQHG7uHypHpP6QqQqQqQqQqQq',  -- USDT TRC20 合约
    10,
    3,
    false  -- 默认禁用，等用户配置 API Key 后激活
)
ON CONFLICT DO NOTHING;