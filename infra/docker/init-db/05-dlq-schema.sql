-- infra/docker/init-db/05-dlq-schema.sql
-- dlq-manager 用スキーマ（dlq-db/migrations/001+002+003 を統合）

\connect dlq_db;

CREATE EXTENSION IF NOT EXISTS "pgcrypto";
CREATE SCHEMA IF NOT EXISTS dlq;

CREATE OR REPLACE FUNCTION dlq.update_updated_at()
RETURNS TRIGGER AS $$ BEGIN NEW.updated_at = NOW(); RETURN NEW; END; $$ LANGUAGE plpgsql;

CREATE TABLE IF NOT EXISTS dlq.dlq_messages (
    id              UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
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
CREATE INDEX idx_dlq_messages_topic      ON dlq.dlq_messages(original_topic);
CREATE INDEX idx_dlq_messages_status     ON dlq.dlq_messages(status);
CREATE INDEX idx_dlq_messages_created_at ON dlq.dlq_messages(created_at);

-- updated_at 自動更新トリガー
CREATE TRIGGER trg_dlq_messages_updated_at
    BEFORE UPDATE ON dlq.dlq_messages
    FOR EACH ROW EXECUTE FUNCTION dlq.update_updated_at();

-- アーカイブテーブル（migrations/003 相当）
CREATE TABLE IF NOT EXISTS dlq.dlq_messages_archive (LIKE dlq.dlq_messages INCLUDING ALL);

CREATE OR REPLACE PROCEDURE dlq.archive_old_dlq_messages()
LANGUAGE plpgsql AS $$
BEGIN
    INSERT INTO dlq.dlq_messages_archive
        SELECT * FROM dlq.dlq_messages
        WHERE status IN ('RESOLVED', 'DEAD')
          AND updated_at < NOW() - INTERVAL '30 days';

    DELETE FROM dlq.dlq_messages
    WHERE status IN ('RESOLVED', 'DEAD')
      AND updated_at < NOW() - INTERVAL '30 days';
END;
$$;
