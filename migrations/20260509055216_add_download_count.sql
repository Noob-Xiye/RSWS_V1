-- Add download_count column to resources table
ALTER TABLE resources ADD COLUMN IF NOT EXISTS download_count INTEGER NOT NULL DEFAULT 0;
