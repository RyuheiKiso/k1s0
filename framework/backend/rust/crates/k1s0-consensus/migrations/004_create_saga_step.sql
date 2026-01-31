-- Saga ステップテーブル
CREATE TABLE IF NOT EXISTS fw_m_saga_step (
    id            BIGSERIAL    PRIMARY KEY,
    saga_id       VARCHAR(255) NOT NULL REFERENCES fw_m_saga_instance(saga_id),
    step_name     VARCHAR(255) NOT NULL,
    step_index    INTEGER      NOT NULL,
    status        VARCHAR(50)  NOT NULL,
    input         JSONB        NOT NULL DEFAULT '{}',
    output        JSONB,
    error_message TEXT,
    executed_at   TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_fw_m_saga_step_saga_id
    ON fw_m_saga_step (saga_id);

CREATE INDEX IF NOT EXISTS idx_fw_m_saga_step_status
    ON fw_m_saga_step (status);
