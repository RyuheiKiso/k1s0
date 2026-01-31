-- Saga インスタンステーブル
CREATE TABLE IF NOT EXISTS fw_m_saga_instance (
    saga_id       VARCHAR(255) PRIMARY KEY,
    saga_name     VARCHAR(255) NOT NULL,
    status        VARCHAR(50)  NOT NULL DEFAULT 'RUNNING',
    current_step  INTEGER      NOT NULL DEFAULT 0,
    context       JSONB        NOT NULL DEFAULT '{}',
    error_message TEXT,
    created_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_fw_m_saga_instance_status
    ON fw_m_saga_instance (status);

CREATE INDEX IF NOT EXISTS idx_fw_m_saga_instance_name
    ON fw_m_saga_instance (saga_name);
