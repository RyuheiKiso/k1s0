-- featureflag-db: feature_flags テーブルの作成

CREATE TABLE IF NOT EXISTS featureflag.feature_flags (
    id          UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    flag_key    VARCHAR(255) NOT NULL UNIQUE,
    description TEXT         NOT NULL DEFAULT '',
    enabled     BOOLEAN      NOT NULL DEFAULT false,
    variants    JSONB        NOT NULL DEFAULT '[]',
    rules       JSONB        NOT NULL DEFAULT '[]',
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_feature_flags_flag_key ON featureflag.feature_flags (flag_key);
CREATE INDEX IF NOT EXISTS idx_feature_flags_enabled ON featureflag.feature_flags (enabled);

CREATE TRIGGER trigger_feature_flags_update_updated_at
    BEFORE UPDATE ON featureflag.feature_flags
    FOR EACH ROW
    EXECUTE FUNCTION featureflag.update_updated_at();
