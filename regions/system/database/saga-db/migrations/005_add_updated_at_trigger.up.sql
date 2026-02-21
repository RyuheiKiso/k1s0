-- saga-db: saga_step_logs の updated_at 関連拡張
-- 注意: saga_states のトリガーは 002_create_saga_states.up.sql で作成済み
--       saga.update_updated_at() 関数は 001_create_schema.up.sql で作成済み

-- saga_step_logs に updated_at カラムを追加
ALTER TABLE saga.saga_step_logs
    ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW();

-- saga_step_logs の updated_at トリガー
CREATE TRIGGER trigger_saga_step_logs_update_updated_at
    BEFORE UPDATE ON saga.saga_step_logs
    FOR EACH ROW
    EXECUTE FUNCTION saga.update_updated_at();
