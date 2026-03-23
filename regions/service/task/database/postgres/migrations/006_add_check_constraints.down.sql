-- CHECK 制約を削除してマイグレーションをロールバックする
SET search_path TO task_service;

ALTER TABLE tasks DROP CONSTRAINT IF EXISTS chk_tasks_status;
ALTER TABLE tasks DROP CONSTRAINT IF EXISTS chk_tasks_priority;
