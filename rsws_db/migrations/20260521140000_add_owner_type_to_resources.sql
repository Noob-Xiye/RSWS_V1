-- Migration: 20260521140000_add_owner_type_to_resources.sql
-- 描述: 为 resources 表添加 owner_type 和 provider_id 字段

-- 添加 owner_type 字段（资源归属：user=用户资源, platform=平台资源）
ALTER TABLE resources 
ADD COLUMN IF NOT EXISTS owner_type VARCHAR(20) NOT NULL DEFAULT 'user' 
  CHECK (owner_type IN ('user', 'platform'));

-- 添加 provider_id 字段（平台资源时记录上传的管理员ID，审计用）
ALTER TABLE resources 
ADD COLUMN IF NOT EXISTS provider_id BIGINT REFERENCES admins(id) ON DELETE SET NULL;

-- 索引
CREATE INDEX IF NOT EXISTS idx_resources_owner_type ON resources(owner_type);
CREATE INDEX IF NOT EXISTS idx_resources_provider_id ON resources(provider_id);

-- 注释
COMMENT ON COLUMN resources.owner_type IS '资源归属：user=用户资源, platform=平台资源';
COMMENT ON COLUMN resources.provider_id IS '平台资源时记录上传的管理员ID（审计用）';

-- 为现有数据设置默认值（owner_type 已经是 DEFAULT 'user'）
-- 无需额外更新，新字段已设置默认值
