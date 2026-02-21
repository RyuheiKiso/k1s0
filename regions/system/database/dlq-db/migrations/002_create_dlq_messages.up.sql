-- dlq-db: dlq_messages テーブル作成

CREATE TABLE IF NOT EXISTS dlq.dlq_messages (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    original_topic  VARCHAR(255) NOT NULL,
    error_message   TEXT         NOT NULL,
    retry_count     INT          NOT NULL DEFAULT 0,
    max_retries     INT          NOT NULL DEFAULT 3,
    payload         JSONB,
    status          VARCHAR(50)  NOT NULL DEFAULT 'PENDING',
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    last_retry_at   TIMESTAMPTZ,

    CONSTRAINT chk_dlq_messages_status CHECK (status IN ('PENDING', 'RETRYING', 'RESOLVED', 'DEAD'))
);

-- インデックス
CREATE INDEX IF NOT EXISTS idx_dlq_messages_original_topic ON dlq.dlq_messages (original_topic);
CREATE INDEX IF NOT EXISTS idx_dlq_messages_status ON dlq.dlq_messages (status);
CREATE INDEX IF NOT EXISTS idx_dlq_messages_created_at ON dlq.dlq_messages (created_at);

-- updated_at トリガー
CREATE TRIGGER trg_dlq_messages_updated_at
    BEFORE UPDATE ON dlq.dlq_messages
    FOR EACH ROW
    EXECUTE FUNCTION dlq.update_updated_at();
