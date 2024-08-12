-- Add migration script here
-- Ensure no NULL values exist
UPDATE objects_v0
SET object_type = 0
WHERE object_type IS NULL;

UPDATE objects_v0
SET color = 0
WHERE color IS NULL;

UPDATE objects_v0
SET position = ''
WHERE position IS NULL;

UPDATE objects_v0
SET size = ''
WHERE size IS NULL;

-- Alter the table to set NOT NULL constraints
ALTER TABLE objects_v0
ALTER COLUMN object_type SET NOT NULL;

ALTER TABLE objects_v0
ALTER COLUMN color SET NOT NULL;

ALTER TABLE objects_v0
ALTER COLUMN position SET NOT NULL;

ALTER TABLE objects_v0
ALTER COLUMN size SET NOT NULL;
