-- ========================================
-- 多级分类支持
-- 创建时间: 2026-05-21 12:25:00
-- ========================================

-- 添加 parent_id 字段（自引用外键）
ALTER TABLE categories ADD COLUMN parent_id BIGINT DEFAULT NULL REFERENCES categories(id) ON DELETE SET NULL;

-- 添加路径索引（用于查询某分类的所有子分类）
CREATE INDEX idx_categories_parent_id ON categories(parent_id);

-- 添加复合索引（parent_id + sort_order，用于排序子分类列表）
CREATE INDEX idx_categories_parent_sort ON categories(parent_id, sort_order);

-- 添加层级路径字段（可选，用于快速查询所有祖先）
-- 存储格式："/1/2/3/"（从根到当前节点的完整路径）
ALTER TABLE categories ADD COLUMN path VARCHAR(500) DEFAULT '';

-- 添加路径索引（支持 LIKE '/1/%' 查询所有后代）
CREATE INDEX idx_categories_path ON categories(path);

-- 添加注释
COMMENT ON COLUMN categories.parent_id IS '父分类ID，NULL表示顶级分类';
COMMENT ON COLUMN categories.path IS '层级路径，格式：/1/2/3/，用于快速查询祖先和后代';

-- 为现有分类设置 path（顶级分类）
UPDATE categories SET path = '/' || id || '/' WHERE parent_id IS NULL;
