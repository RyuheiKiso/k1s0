DROP TRIGGER IF EXISTS trigger_saga_step_logs_update_updated_at ON saga.saga_step_logs;
ALTER TABLE saga.saga_step_logs DROP COLUMN IF EXISTS updated_at;
