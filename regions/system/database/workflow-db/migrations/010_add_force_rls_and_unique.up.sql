-- workflow テーブル全体に FORCE ROW LEVEL SECURITY を追加する。
-- HIGH-DB-002 監査対応: 008 マイグレーションで ENABLE はされているが FORCE が欠落していた。
-- FORCE により テーブルオーナー（アプリケーションDBユーザー）にも RLS を適用する。
-- HIGH-DB-007: workflow_definitions の UNIQUE(name) を UNIQUE(tenant_id, name) に変更する。
-- RLS ポリシーを AS RESTRICTIVE + WITH CHECK で既に正確に設定済み（009 マイグレーション）のため、
-- FORCE の追加と UNIQUE 制約変更のみ実施する。

BEGIN;

SET LOCAL search_path TO workflow, public;

-- workflow_definitions テーブルに FORCE ROW LEVEL SECURITY を追加する
ALTER TABLE workflow.workflow_definitions FORCE ROW LEVEL SECURITY;

-- workflow_instances テーブルに FORCE ROW LEVEL SECURITY を追加する
ALTER TABLE workflow.workflow_instances FORCE ROW LEVEL SECURITY;

-- workflow_tasks テーブルに FORCE ROW LEVEL SECURITY を追加する
ALTER TABLE workflow.workflow_tasks FORCE ROW LEVEL SECURITY;

-- workflow_definitions テーブルの UNIQUE(name) を UNIQUE(tenant_id, name) に変更する（HIGH-DB-007 対応）
DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'workflow_definitions_name_key' AND conrelid = 'workflow.workflow_definitions'::regclass
    ) THEN
        ALTER TABLE workflow.workflow_definitions DROP CONSTRAINT workflow_definitions_name_key;
    END IF;
END $$;
-- 既存の name 単独インデックスを削除してテナントスコープの UNIQUE INDEX を作成する
DROP INDEX IF EXISTS workflow.idx_workflow_definitions_name;
CREATE UNIQUE INDEX IF NOT EXISTS uq_workflow_definitions_tenant_name
    ON workflow.workflow_definitions (tenant_id, name);

COMMIT;
