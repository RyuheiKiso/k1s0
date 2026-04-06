-- saga.workflow_definitions テーブルにテナント分離を実装する。
-- HIGH-DB-001 監査対応: saga_states / saga_step_logs は 008/010 で対応済みだが、
-- workflow_definitions テーブルは未対応のためテナント分離を追加する。
-- 既存データは 'system' テナントとしてバックフィルし、新規挿入を強制する。

BEGIN;

SET LOCAL search_path TO saga, public;

-- workflow_definitions テーブルに tenant_id カラムを追加する
ALTER TABLE saga.workflow_definitions
    ADD COLUMN IF NOT EXISTS tenant_id TEXT NOT NULL DEFAULT 'system';

-- バックフィル後はデフォルト値を削除し、新規挿入時の明示指定を強制する
ALTER TABLE saga.workflow_definitions
    ALTER COLUMN tenant_id DROP DEFAULT;

-- tenant_id のインデックスを追加してクエリ性能を確保する
CREATE INDEX IF NOT EXISTS idx_workflow_definitions_tenant_id
    ON saga.workflow_definitions (tenant_id);

-- workflow_definitions テーブルの RLS を有効化する
ALTER TABLE saga.workflow_definitions ENABLE ROW LEVEL SECURITY;
ALTER TABLE saga.workflow_definitions FORCE ROW LEVEL SECURITY;

-- テナント分離ポリシーを設定する
DROP POLICY IF EXISTS tenant_isolation ON saga.workflow_definitions;
CREATE POLICY tenant_isolation ON saga.workflow_definitions
    AS RESTRICTIVE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

COMMIT;
