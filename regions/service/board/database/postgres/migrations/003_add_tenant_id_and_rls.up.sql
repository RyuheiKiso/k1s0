-- マルチテナント対応: board_columns テーブルに tenant_id カラムを追加し、RLS ポリシーを設定する。
SET search_path TO board_service;

ALTER TABLE board_columns
    ADD COLUMN IF NOT EXISTS tenant_id TEXT NOT NULL DEFAULT 'system';

CREATE INDEX IF NOT EXISTS idx_board_columns_tenant_id ON board_columns (tenant_id);
CREATE INDEX IF NOT EXISTS idx_board_columns_tenant_project ON board_columns (tenant_id, project_id);

ALTER TABLE board_columns ENABLE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS tenant_isolation ON board_columns;
CREATE POLICY tenant_isolation ON board_columns
    USING (tenant_id = current_setting('app.current_tenant_id', true)::TEXT);
