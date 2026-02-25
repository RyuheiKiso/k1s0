CREATE TABLE IF NOT EXISTS workflow.workflow_definitions (
    id          UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    name        VARCHAR(255) NOT NULL UNIQUE,
    description TEXT         NOT NULL DEFAULT '',
    steps       JSONB        NOT NULL DEFAULT '[]',
    enabled     BOOLEAN      NOT NULL DEFAULT true,
    version     INT          NOT NULL DEFAULT 1,
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_workflow_definitions_name ON workflow.workflow_definitions (name);
CREATE INDEX IF NOT EXISTS idx_workflow_definitions_enabled ON workflow.workflow_definitions (enabled);

CREATE TRIGGER trigger_workflow_definitions_update_updated_at
    BEFORE UPDATE ON workflow.workflow_definitions
    FOR EACH ROW
    EXECUTE FUNCTION workflow.update_updated_at();
