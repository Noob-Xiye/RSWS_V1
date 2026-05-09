-- ========================================
-- 分类表
-- 创建时间: 2026-05-09 19:00:00
-- ========================================

CREATE TABLE categories (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    description TEXT,
    sort_order INTEGER DEFAULT 0,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- 插入默认分类
INSERT INTO categories (name, description, sort_order) VALUES 
    ('模板', '各类模板资源', 1),
    ('插件', '插件资源', 2),
    ('教程', '教程资料', 3),
    ('工具', '工具软件', 4);

CREATE INDEX idx_categories_sort_order ON categories(sort_order);
CREATE INDEX idx_categories_is_active ON categories(is_active);
