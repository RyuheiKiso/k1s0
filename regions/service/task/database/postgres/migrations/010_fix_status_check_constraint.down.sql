-- 010_fix_status_check_constraint のロールバック
-- review を含む制約を削除し、元の4値制約に戻す
SET LOCAL search_path TO task_service, public;

DO $$
BEGIN
    ALTER TABLE tasks DROP CONSTRAINT IF EXISTS chk_tasks_status;
    ALTER TABLE tasks
        ADD CONSTRAINT chk_tasks_status
        CHECK (status IN ('open', 'in_progress', 'done', 'cancelled'));
EXCEPTION WHEN duplicate_object THEN
    NULL;
END $$;
