-- rules: ルール定義
CREATE TABLE IF NOT EXISTS rule_engine.rules (
    id          UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    name        VARCHAR(255) NOT NULL,
    description TEXT,
    condition   JSONB       NOT NULL,
    action      JSONB       NOT NULL,
    priority    INTEGER     NOT NULL DEFAULT 0,
    status      VARCHAR(50) NOT NULL DEFAULT 'active',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT chk_rules_status CHECK (status IN ('active', 'inactive', 'archived'))
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_rules_name ON rule_engine.rules (name);
CREATE INDEX IF NOT EXISTS idx_rules_status ON rule_engine.rules (status);
CREATE INDEX IF NOT EXISTS idx_rules_priority ON rule_engine.rules (priority);

CREATE TRIGGER trigger_rules_updated_at
    BEFORE UPDATE ON rule_engine.rules
    FOR EACH ROW
    EXECUTE FUNCTION rule_engine.update_updated_at();

-- rule_sets: ルールセット
CREATE TABLE IF NOT EXISTS rule_engine.rule_sets (
    id          UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    name        VARCHAR(255) NOT NULL,
    description TEXT,
    status      VARCHAR(50) NOT NULL DEFAULT 'active',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT chk_rule_sets_status CHECK (status IN ('active', 'inactive', 'archived'))
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_rule_sets_name ON rule_engine.rule_sets (name);

CREATE TRIGGER trigger_rule_sets_updated_at
    BEFORE UPDATE ON rule_engine.rule_sets
    FOR EACH ROW
    EXECUTE FUNCTION rule_engine.update_updated_at();

-- rule_set_versions: ルールセットバージョン
CREATE TABLE IF NOT EXISTS rule_engine.rule_set_versions (
    id          UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    rule_set_id UUID        NOT NULL REFERENCES rule_engine.rule_sets(id) ON DELETE CASCADE,
    version     INTEGER     NOT NULL,
    rules       JSONB       NOT NULL DEFAULT '[]',
    status      VARCHAR(50) NOT NULL DEFAULT 'draft',
    published_at TIMESTAMPTZ,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT chk_rule_set_versions_status CHECK (status IN ('draft', 'published', 'deprecated')),
    CONSTRAINT uq_rule_set_version UNIQUE (rule_set_id, version)
);

CREATE INDEX IF NOT EXISTS idx_rule_set_versions_rule_set ON rule_engine.rule_set_versions (rule_set_id);
CREATE INDEX IF NOT EXISTS idx_rule_set_versions_status ON rule_engine.rule_set_versions (status);

CREATE TRIGGER trigger_rule_set_versions_updated_at
    BEFORE UPDATE ON rule_engine.rule_set_versions
    FOR EACH ROW
    EXECUTE FUNCTION rule_engine.update_updated_at();

-- evaluation_logs: ルール評価ログ
CREATE TABLE IF NOT EXISTS rule_engine.evaluation_logs (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    rule_set_id     UUID        NOT NULL REFERENCES rule_engine.rule_sets(id) ON DELETE CASCADE,
    rule_id         UUID        REFERENCES rule_engine.rules(id) ON DELETE SET NULL,
    input           JSONB       NOT NULL,
    output          JSONB,
    matched         BOOLEAN     NOT NULL DEFAULT false,
    execution_time_ms INTEGER,
    error_message   TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_evaluation_logs_rule_set ON rule_engine.evaluation_logs (rule_set_id);
CREATE INDEX IF NOT EXISTS idx_evaluation_logs_rule ON rule_engine.evaluation_logs (rule_id);
CREATE INDEX IF NOT EXISTS idx_evaluation_logs_matched ON rule_engine.evaluation_logs (matched);
CREATE INDEX IF NOT EXISTS idx_evaluation_logs_created_at ON rule_engine.evaluation_logs (created_at);
