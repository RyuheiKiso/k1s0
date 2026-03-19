-- flow_definitions: イベント監視フロー定義
CREATE TABLE IF NOT EXISTS event_monitor.flow_definitions (
    id          UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    name        VARCHAR(255) NOT NULL,
    description TEXT,
    trigger_event VARCHAR(255) NOT NULL,
    conditions  JSONB       NOT NULL DEFAULT '{}',
    actions     JSONB       NOT NULL DEFAULT '[]',
    status      VARCHAR(50) NOT NULL DEFAULT 'active',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT chk_flow_definitions_status CHECK (status IN ('active', 'inactive', 'archived'))
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_flow_definitions_name ON event_monitor.flow_definitions (name);
CREATE INDEX IF NOT EXISTS idx_flow_definitions_trigger ON event_monitor.flow_definitions (trigger_event);
CREATE INDEX IF NOT EXISTS idx_flow_definitions_status ON event_monitor.flow_definitions (status);

CREATE TRIGGER trigger_flow_definitions_updated_at
    BEFORE UPDATE ON event_monitor.flow_definitions
    FOR EACH ROW
    EXECUTE FUNCTION event_monitor.update_updated_at();

-- event_records: 受信イベント記録
CREATE TABLE IF NOT EXISTS event_monitor.event_records (
    id          UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type  VARCHAR(255) NOT NULL,
    source      VARCHAR(255) NOT NULL,
    payload     JSONB       NOT NULL DEFAULT '{}',
    metadata    JSONB       NOT NULL DEFAULT '{}',
    status      VARCHAR(50) NOT NULL DEFAULT 'received',
    processed_at TIMESTAMPTZ,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT chk_event_records_status CHECK (status IN ('received', 'processing', 'processed', 'failed'))
);

CREATE INDEX IF NOT EXISTS idx_event_records_event_type ON event_monitor.event_records (event_type);
CREATE INDEX IF NOT EXISTS idx_event_records_source ON event_monitor.event_records (source);
CREATE INDEX IF NOT EXISTS idx_event_records_status ON event_monitor.event_records (status);
CREATE INDEX IF NOT EXISTS idx_event_records_created_at ON event_monitor.event_records (created_at);

-- flow_instances: フロー実行インスタンス
CREATE TABLE IF NOT EXISTS event_monitor.flow_instances (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    flow_definition_id UUID    NOT NULL REFERENCES event_monitor.flow_definitions(id) ON DELETE CASCADE,
    event_record_id UUID       NOT NULL REFERENCES event_monitor.event_records(id) ON DELETE CASCADE,
    status          VARCHAR(50) NOT NULL DEFAULT 'pending',
    result          JSONB,
    error_message   TEXT,
    started_at      TIMESTAMPTZ,
    completed_at    TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT chk_flow_instances_status CHECK (status IN ('pending', 'running', 'completed', 'failed', 'cancelled'))
);

CREATE INDEX IF NOT EXISTS idx_flow_instances_flow_def ON event_monitor.flow_instances (flow_definition_id);
CREATE INDEX IF NOT EXISTS idx_flow_instances_event ON event_monitor.flow_instances (event_record_id);
CREATE INDEX IF NOT EXISTS idx_flow_instances_status ON event_monitor.flow_instances (status);

CREATE TRIGGER trigger_flow_instances_updated_at
    BEFORE UPDATE ON event_monitor.flow_instances
    FOR EACH ROW
    EXECUTE FUNCTION event_monitor.update_updated_at();
