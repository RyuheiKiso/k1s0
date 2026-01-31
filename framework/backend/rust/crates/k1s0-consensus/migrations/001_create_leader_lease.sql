-- リーダーリーステーブル
CREATE TABLE IF NOT EXISTS fw_m_leader_lease (
    lease_key    VARCHAR(255) PRIMARY KEY,
    holder_id    VARCHAR(255) NOT NULL,
    fence_token  BIGINT       NOT NULL DEFAULT 1,
    expires_at   TIMESTAMPTZ  NOT NULL,
    created_at   TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_fw_m_leader_lease_expires
    ON fw_m_leader_lease (expires_at);
