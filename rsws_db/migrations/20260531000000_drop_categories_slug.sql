-- Drop unused slug column from categories table
-- Date: 2026-05-31
-- Reason: Field is not used by frontend or backend. Clean code principle.

ALTER TABLE categories DROP COLUMN IF EXISTS slug;
