CREATE TABLE IF NOT EXISTS workflow.workflow_tasks (
    id           UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    instance_id  UUID         NOT NULL REFERENCES workflow.workflow_instances(id) ON DELETE CASCADE,
    step_id      VARCHAR(255) NOT NULL DEFAULT '',
    step_name    VARCHAR(255) NOT NULL DEFAULT '',
    assignee_id  VARCHAR(255) NOT NULL DEFAULT '',
    status       VARCHAR(50)  NOT NULL DEFAULT 'pending',
    comment      TEXT,
    actor_id     VARCHAR(255),
    due_at       TIMESTAMPTZ,
    decided_at   TIMESTAMPTZ,
    created_at   TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT chk_tasks_status CHECK (status IN ('pending', 'assigned', 'approved', 'rejected'))
);

CREATE INDEX IF NOT EXISTS idx_workflow_tasks_instance_id ON workflow.workflow_tasks (instance_id);
CREATE INDEX IF NOT EXISTS idx_workflow_tasks_assignee_id ON workflow.workflow_tasks (assignee_id);
CREATE INDEX IF NOT EXISTS idx_workflow_tasks_status ON workflow.workflow_tasks (status);
CREATE INDEX IF NOT EXISTS idx_workflow_tasks_due_at ON workflow.workflow_tasks (due_at);

CREATE TRIGGER trigger_workflow_tasks_update_updated_at
    BEFORE UPDATE ON workflow.workflow_tasks
    FOR EACH ROW
    EXECUTE FUNCTION workflow.update_updated_at();
