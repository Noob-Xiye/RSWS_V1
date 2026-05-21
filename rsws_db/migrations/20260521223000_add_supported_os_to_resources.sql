-- 为 resources 表添加 supported_os 字段（JSONB 数组）
-- 支持的操作系统：windows, macos, linux, ios, android

ALTER TABLE resources 
ADD COLUMN IF NOT EXISTS supported_os JSONB DEFAULT '[]'::jsonb;

-- 创建 GIN 索引加速查询
CREATE INDEX IF NOT EXISTS idx_resources_supported_os 
ON resources USING GIN (supported_os);

COMMENT ON COLUMN resources.supported_os IS '支持的操作系统：["windows", "macos", "linux", "ios", "android"]';
