-- HIGH-005対応: rule-engine-db にテナント分離カラムと RLS ポリシーを追加する
-- rules / rule_sets / rule_set_versions / evaluation_logs は tenant_id カラムを持たないため
-- 追加してテナント間のデータ漏洩を防ぐ

SET LOCAL search_path TO rule_engine, public;

-- rules テーブルに tenant_id カラムを追加する
-- 既存データはシステムテナント 'system' として扱う
ALTER TABLE rule_engine.rules
    ADD COLUMN IF NOT EXISTS tenant_id TEXT NOT NULL DEFAULT 'system';

-- rule_sets テーブルに tenant_id カラムを追加する
ALTER TABLE rule_engine.rule_sets
    ADD COLUMN IF NOT EXISTS tenant_id TEXT NOT NULL DEFAULT 'system';

-- rule_set_versions は rule_sets の子テーブルのため、
-- rule_sets の RLS 経由でテナント分離される。直接テナントIDを持たせる設計とする
ALTER TABLE rule_engine.rule_set_versions
    ADD COLUMN IF NOT EXISTS tenant_id TEXT NOT NULL DEFAULT 'system';

-- evaluation_logs テーブルに tenant_id カラムを追加する
ALTER TABLE rule_engine.evaluation_logs
    ADD COLUMN IF NOT EXISTS tenant_id TEXT NOT NULL DEFAULT 'system';

-- テナントIDによる高速検索のためのインデックスを追加する
CREATE INDEX IF NOT EXISTS idx_rules_tenant_id
    ON rule_engine.rules (tenant_id);

CREATE INDEX IF NOT EXISTS idx_rule_sets_tenant_id
    ON rule_engine.rule_sets (tenant_id);

CREATE INDEX IF NOT EXISTS idx_rule_set_versions_tenant_id
    ON rule_engine.rule_set_versions (tenant_id);

CREATE INDEX IF NOT EXISTS idx_evaluation_logs_tenant_id
    ON rule_engine.evaluation_logs (tenant_id);

-- ADD COLUMN 後は DEFAULT 制約を削除し、今後のINSERTで明示的に指定させる
ALTER TABLE rule_engine.rules ALTER COLUMN tenant_id DROP DEFAULT;
ALTER TABLE rule_engine.rule_sets ALTER COLUMN tenant_id DROP DEFAULT;
ALTER TABLE rule_engine.rule_set_versions ALTER COLUMN tenant_id DROP DEFAULT;
ALTER TABLE rule_engine.evaluation_logs ALTER COLUMN tenant_id DROP DEFAULT;

-- rules テーブルに RLS を有効化する
-- FORCE ROW LEVEL SECURITY によりテーブルオーナーにも RLS を適用する
ALTER TABLE rule_engine.rules ENABLE ROW LEVEL SECURITY;
ALTER TABLE rule_engine.rules FORCE ROW LEVEL SECURITY;

-- rules のテナント分離ポリシー
-- RESTRICTIVE ポリシーにより他の PERMISSIVE ポリシーと AND 結合される
CREATE POLICY tenant_isolation ON rule_engine.rules
    AS RESTRICTIVE
    USING (tenant_id::TEXT = current_setting('app.current_tenant_id', true)::TEXT)
    WITH CHECK (tenant_id::TEXT = current_setting('app.current_tenant_id', true)::TEXT);

-- rule_sets テーブルに RLS を有効化する
ALTER TABLE rule_engine.rule_sets ENABLE ROW LEVEL SECURITY;
ALTER TABLE rule_engine.rule_sets FORCE ROW LEVEL SECURITY;

-- rule_sets のテナント分離ポリシー
CREATE POLICY tenant_isolation ON rule_engine.rule_sets
    AS RESTRICTIVE
    USING (tenant_id::TEXT = current_setting('app.current_tenant_id', true)::TEXT)
    WITH CHECK (tenant_id::TEXT = current_setting('app.current_tenant_id', true)::TEXT);

-- rule_set_versions テーブルに RLS を有効化する
ALTER TABLE rule_engine.rule_set_versions ENABLE ROW LEVEL SECURITY;
ALTER TABLE rule_engine.rule_set_versions FORCE ROW LEVEL SECURITY;

-- rule_set_versions のテナント分離ポリシー
CREATE POLICY tenant_isolation ON rule_engine.rule_set_versions
    AS RESTRICTIVE
    USING (tenant_id::TEXT = current_setting('app.current_tenant_id', true)::TEXT)
    WITH CHECK (tenant_id::TEXT = current_setting('app.current_tenant_id', true)::TEXT);

-- evaluation_logs テーブルに RLS を有効化する
ALTER TABLE rule_engine.evaluation_logs ENABLE ROW LEVEL SECURITY;
ALTER TABLE rule_engine.evaluation_logs FORCE ROW LEVEL SECURITY;

-- evaluation_logs のテナント分離ポリシー
CREATE POLICY tenant_isolation ON rule_engine.evaluation_logs
    AS RESTRICTIVE
    USING (tenant_id::TEXT = current_setting('app.current_tenant_id', true)::TEXT)
    WITH CHECK (tenant_id::TEXT = current_setting('app.current_tenant_id', true)::TEXT);
