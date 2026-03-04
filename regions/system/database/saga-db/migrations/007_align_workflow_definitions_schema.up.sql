-- Align legacy workflow_definitions schema (name, steps, created_at)
-- to canonical schema (name, version, definition, enabled, created_at, updated_at).

DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_schema = 'saga'
          AND table_name = 'workflow_definitions'
          AND column_name = 'steps'
    ) AND NOT EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_schema = 'saga'
          AND table_name = 'workflow_definitions'
          AND column_name = 'definition'
    ) THEN
        ALTER TABLE saga.workflow_definitions RENAME COLUMN steps TO definition;
    END IF;
END $$;

ALTER TABLE saga.workflow_definitions
    ADD COLUMN IF NOT EXISTS version INT NOT NULL DEFAULT 1;

ALTER TABLE saga.workflow_definitions
    ADD COLUMN IF NOT EXISTS enabled BOOLEAN NOT NULL DEFAULT TRUE;

ALTER TABLE saga.workflow_definitions
    ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW();

UPDATE saga.workflow_definitions
SET definition = '[]'::jsonb
WHERE definition IS NULL;

ALTER TABLE saga.workflow_definitions
    ALTER COLUMN definition SET DEFAULT '[]'::jsonb;

ALTER TABLE saga.workflow_definitions
    ALTER COLUMN definition SET NOT NULL;

CREATE INDEX IF NOT EXISTS idx_workflow_definitions_enabled
    ON saga.workflow_definitions (enabled);

DROP TRIGGER IF EXISTS update_workflow_definitions_updated_at ON saga.workflow_definitions;
CREATE TRIGGER update_workflow_definitions_updated_at
    BEFORE UPDATE ON saga.workflow_definitions
    FOR EACH ROW
    EXECUTE FUNCTION saga.update_updated_at();
