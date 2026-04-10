-- HIGH-005対応: rule-engine-db tenant_id カラムと RLS ポリシーのロールバック

SET LOCAL search_path TO rule_engine, public;

-- RLS ポリシーを削除し RLS を無効化する
DROP POLICY IF EXISTS tenant_isolation ON rule_engine.evaluation_logs;
ALTER TABLE rule_engine.evaluation_logs DISABLE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS tenant_isolation ON rule_engine.rule_set_versions;
ALTER TABLE rule_engine.rule_set_versions DISABLE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS tenant_isolation ON rule_engine.rule_sets;
ALTER TABLE rule_engine.rule_sets DISABLE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS tenant_isolation ON rule_engine.rules;
ALTER TABLE rule_engine.rules DISABLE ROW LEVEL SECURITY;

-- インデックスを削除する
DROP INDEX IF EXISTS rule_engine.idx_evaluation_logs_tenant_id;
DROP INDEX IF EXISTS rule_engine.idx_rule_set_versions_tenant_id;
DROP INDEX IF EXISTS rule_engine.idx_rule_sets_tenant_id;
DROP INDEX IF EXISTS rule_engine.idx_rules_tenant_id;

-- tenant_id カラムを削除する
ALTER TABLE rule_engine.evaluation_logs DROP COLUMN IF EXISTS tenant_id;
ALTER TABLE rule_engine.rule_set_versions DROP COLUMN IF EXISTS tenant_id;
ALTER TABLE rule_engine.rule_sets DROP COLUMN IF EXISTS tenant_id;
ALTER TABLE rule_engine.rules DROP COLUMN IF EXISTS tenant_id;
