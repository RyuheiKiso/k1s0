DROP TRIGGER IF EXISTS update_workflow_definitions_updated_at ON saga.workflow_definitions;
DROP INDEX IF EXISTS saga.idx_workflow_definitions_enabled;

ALTER TABLE saga.workflow_definitions
    DROP COLUMN IF EXISTS updated_at;

ALTER TABLE saga.workflow_definitions
    DROP COLUMN IF EXISTS enabled;

ALTER TABLE saga.workflow_definitions
    DROP COLUMN IF EXISTS version;

DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_schema = 'saga'
          AND table_name = 'workflow_definitions'
          AND column_name = 'definition'
    ) AND NOT EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_schema = 'saga'
          AND table_name = 'workflow_definitions'
          AND column_name = 'steps'
    ) THEN
        ALTER TABLE saga.workflow_definitions RENAME COLUMN definition TO steps;
    END IF;
END $$;
