ALTER TABLE resources ADD COLUMN IF NOT EXISTS supported_os JSONB DEFAULT '[]'::jsonb;
CREATE INDEX IF NOT EXISTS idx_resources_supported_os ON resources USING GIN (supported_os);
