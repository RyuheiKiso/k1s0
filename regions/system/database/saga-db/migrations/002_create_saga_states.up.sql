-- saga-db: saga_states テーブル作成

CREATE TABLE IF NOT EXISTS saga.saga_states (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    workflow_name   VARCHAR(255) NOT NULL,
    current_step    INT          NOT NULL DEFAULT 0,
    status          VARCHAR(50)  NOT NULL DEFAULT 'STARTED',
    payload         JSONB,
    correlation_id  VARCHAR(255),
    initiated_by    VARCHAR(255),
    error_message   TEXT,
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT chk_saga_states_status CHECK (status IN ('STARTED', 'RUNNING', 'COMPLETED', 'COMPENSATING', 'FAILED', 'CANCELLED'))
);

-- インデックス
CREATE INDEX IF NOT EXISTS idx_saga_states_workflow_name ON saga.saga_states (workflow_name);
CREATE INDEX IF NOT EXISTS idx_saga_states_status ON saga.saga_states (status);
CREATE INDEX IF NOT EXISTS idx_saga_states_correlation_id ON saga.saga_states (correlation_id) WHERE correlation_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_saga_states_created_at ON saga.saga_states (created_at);

-- updated_at トリガー
CREATE TRIGGER update_saga_states_updated_at
    BEFORE UPDATE ON saga.saga_states
    FOR EACH ROW
    EXECUTE FUNCTION saga.update_updated_at();
