-- auth-db: api_keys テーブル作成

CREATE TABLE IF NOT EXISTS auth.api_keys (
    id           UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    name         VARCHAR(255) NOT NULL,
    key_hash     VARCHAR(255) UNIQUE NOT NULL,
    key_prefix   VARCHAR(10)  NOT NULL,
    service_name VARCHAR(255) NOT NULL,
    tier         VARCHAR(20)  NOT NULL,
    permissions  JSONB        NOT NULL DEFAULT '[]',
    expires_at   TIMESTAMPTZ,
    last_used_at TIMESTAMPTZ,
    is_active    BOOLEAN      NOT NULL DEFAULT true,
    created_by   UUID         REFERENCES auth.users(id) ON DELETE SET NULL,
    created_at   TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT chk_api_keys_tier CHECK (tier IN ('system', 'business', 'service'))
);

-- インデックス
CREATE INDEX IF NOT EXISTS idx_api_keys_key_hash ON auth.api_keys (key_hash) WHERE is_active = true;
CREATE INDEX IF NOT EXISTS idx_api_keys_service_name ON auth.api_keys (service_name);
CREATE INDEX IF NOT EXISTS idx_api_keys_key_prefix ON auth.api_keys (key_prefix);
CREATE INDEX IF NOT EXISTS idx_api_keys_expires_at ON auth.api_keys (expires_at) WHERE expires_at IS NOT NULL;

-- updated_at トリガー
CREATE TRIGGER trigger_api_keys_update_updated_at
    BEFORE UPDATE ON auth.api_keys
    FOR EACH ROW
    EXECUTE FUNCTION auth.update_updated_at();
