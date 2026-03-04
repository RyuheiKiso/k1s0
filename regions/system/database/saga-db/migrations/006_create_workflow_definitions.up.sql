-- saga-db: workflow_definitions table

CREATE TABLE IF NOT EXISTS saga.workflow_definitions (
    name        VARCHAR(255) PRIMARY KEY,
    version     INT NOT NULL DEFAULT 1,
    definition  JSONB NOT NULL DEFAULT '[]',
    enabled     BOOLEAN NOT NULL DEFAULT TRUE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_workflow_definitions_enabled
    ON saga.workflow_definitions (enabled);

DROP TRIGGER IF EXISTS update_workflow_definitions_updated_at ON saga.workflow_definitions;
CREATE TRIGGER update_workflow_definitions_updated_at
    BEFORE UPDATE ON saga.workflow_definitions
    FOR EACH ROW
    EXECUTE FUNCTION saga.update_updated_at();
