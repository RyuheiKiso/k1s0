-- saga-db: saga_step_logs テーブル作成

CREATE TABLE IF NOT EXISTS saga.saga_step_logs (
    id                UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    saga_id           UUID        NOT NULL REFERENCES saga.saga_states(id) ON DELETE CASCADE,
    step_index        INT         NOT NULL,
    step_name         VARCHAR(255) NOT NULL,
    action            VARCHAR(50)  NOT NULL,
    status            VARCHAR(50)  NOT NULL,
    request_payload   JSONB,
    response_payload  JSONB,
    error_message     TEXT,
    started_at        TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    completed_at      TIMESTAMPTZ,

    CONSTRAINT chk_saga_step_logs_action CHECK (action IN ('EXECUTE', 'COMPENSATE')),
    CONSTRAINT chk_saga_step_logs_status CHECK (status IN ('SUCCESS', 'FAILED', 'TIMEOUT', 'SKIPPED'))
);

-- インデックス
CREATE INDEX IF NOT EXISTS idx_saga_step_logs_saga_id_step_index ON saga.saga_step_logs (saga_id, step_index);
