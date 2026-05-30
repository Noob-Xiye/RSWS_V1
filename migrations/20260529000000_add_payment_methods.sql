-- 支付方式管理表
CREATE TABLE IF NOT EXISTS payment_methods (
    id BIGSERIAL PRIMARY KEY,
    method_type VARCHAR(50) NOT NULL UNIQUE,
    method_name VARCHAR(100) NOT NULL,
    is_enabled BOOLEAN DEFAULT true,
    config JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- 插入默认支付方式
INSERT INTO payment_methods (method_type, method_name, is_enabled, config)
VALUES 
    ('paypal', 'PayPal', true, '{}'),
    ('usdt', 'USDT (Tron)', true, '{}')
ON CONFLICT (method_type) DO NOTHING;
