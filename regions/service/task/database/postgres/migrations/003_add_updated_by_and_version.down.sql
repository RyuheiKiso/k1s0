SET search_path TO task_service;
ALTER TABLE tasks DROP COLUMN IF EXISTS updated_by;
ALTER TABLE tasks DROP COLUMN IF EXISTS version;
