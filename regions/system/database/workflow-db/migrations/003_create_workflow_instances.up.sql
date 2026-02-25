CREATE TABLE IF NOT EXISTS workflow.workflow_instances (
    id               UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    definition_id    UUID         NOT NULL REFERENCES workflow.workflow_definitions(id),
    workflow_name    VARCHAR(255) NOT NULL DEFAULT '',
    title            VARCHAR(255) NOT NULL DEFAULT '',
    initiator_id     VARCHAR(255) NOT NULL DEFAULT '',
    current_step_id  VARCHAR(255) NOT NULL DEFAULT '',
    status           VARCHAR(50)  NOT NULL DEFAULT 'running',
    context          JSONB        NOT NULL DEFAULT '{}',
    started_at       TIMESTAMPTZ,
    completed_at     TIMESTAMPTZ,
    created_at       TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT chk_instances_status CHECK (status IN ('running', 'completed', 'cancelled', 'failed'))
);

CREATE INDEX IF NOT EXISTS idx_workflow_instances_definition_id ON workflow.workflow_instances (definition_id);
CREATE INDEX IF NOT EXISTS idx_workflow_instances_status ON workflow.workflow_instances (status);
CREATE INDEX IF NOT EXISTS idx_workflow_instances_initiator_id ON workflow.workflow_instances (initiator_id);

CREATE TRIGGER trigger_workflow_instances_update_updated_at
    BEFORE UPDATE ON workflow.workflow_instances
    FOR EACH ROW
    EXECUTE FUNCTION workflow.update_updated_at();
