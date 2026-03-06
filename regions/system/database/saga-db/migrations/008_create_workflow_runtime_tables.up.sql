CREATE SCHEMA IF NOT EXISTS workflow;

CREATE TABLE IF NOT EXISTS workflow.workflow_definitions (
    id          TEXT PRIMARY KEY,
    name        VARCHAR(255) NOT NULL UNIQUE,
    description TEXT NOT NULL DEFAULT '',
    steps       JSONB NOT NULL DEFAULT '[]'::jsonb,
    enabled     BOOLEAN NOT NULL DEFAULT TRUE,
    version     INT NOT NULL DEFAULT 1,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS workflow.workflow_instances (
    id              TEXT PRIMARY KEY,
    definition_id   TEXT NOT NULL REFERENCES workflow.workflow_definitions(id) ON DELETE CASCADE,
    workflow_name   VARCHAR(255) NOT NULL,
    title           TEXT NOT NULL,
    initiator_id    TEXT NOT NULL,
    current_step_id TEXT NOT NULL DEFAULT '',
    status          VARCHAR(32) NOT NULL,
    context         JSONB NOT NULL DEFAULT '{}'::jsonb,
    started_at      TIMESTAMPTZ,
    completed_at    TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS workflow.workflow_tasks (
    id          TEXT PRIMARY KEY,
    instance_id TEXT NOT NULL REFERENCES workflow.workflow_instances(id) ON DELETE CASCADE,
    step_id     TEXT NOT NULL,
    step_name   TEXT NOT NULL,
    assignee_id TEXT NOT NULL DEFAULT '',
    status      VARCHAR(32) NOT NULL,
    comment     TEXT,
    actor_id    TEXT,
    due_at      TIMESTAMPTZ,
    decided_at  TIMESTAMPTZ,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_workflow_definitions_enabled
    ON workflow.workflow_definitions (enabled);

CREATE INDEX IF NOT EXISTS idx_workflow_instances_definition_id
    ON workflow.workflow_instances (definition_id);

CREATE INDEX IF NOT EXISTS idx_workflow_instances_status
    ON workflow.workflow_instances (status);

CREATE INDEX IF NOT EXISTS idx_workflow_instances_initiator_id
    ON workflow.workflow_instances (initiator_id);

CREATE INDEX IF NOT EXISTS idx_workflow_tasks_instance_id
    ON workflow.workflow_tasks (instance_id);

CREATE INDEX IF NOT EXISTS idx_workflow_tasks_assignee_id
    ON workflow.workflow_tasks (assignee_id);

CREATE INDEX IF NOT EXISTS idx_workflow_tasks_status
    ON workflow.workflow_tasks (status);

DROP TRIGGER IF EXISTS update_workflow_definitions_updated_at ON workflow.workflow_definitions;
CREATE TRIGGER update_workflow_definitions_updated_at
    BEFORE UPDATE ON workflow.workflow_definitions
    FOR EACH ROW
    EXECUTE FUNCTION saga.update_updated_at();

DROP TRIGGER IF EXISTS update_workflow_tasks_updated_at ON workflow.workflow_tasks;
CREATE TRIGGER update_workflow_tasks_updated_at
    BEFORE UPDATE ON workflow.workflow_tasks
    FOR EACH ROW
    EXECUTE FUNCTION saga.update_updated_at();
