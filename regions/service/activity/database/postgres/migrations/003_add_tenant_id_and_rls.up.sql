-- マルチテナント対応: activities テーブルに tenant_id カラムを追加し、RLS ポリシーを設定する。
SET search_path TO activity_service;

ALTER TABLE activities
    ADD COLUMN IF NOT EXISTS tenant_id TEXT NOT NULL DEFAULT 'system';

CREATE INDEX IF NOT EXISTS idx_activities_tenant_id ON activities (tenant_id);
CREATE INDEX IF NOT EXISTS idx_activities_tenant_task ON activities (tenant_id, task_id);

ALTER TABLE activities ENABLE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS tenant_isolation ON activities;
CREATE POLICY tenant_isolation ON activities
    USING (tenant_id = current_setting('app.current_tenant_id', true)::TEXT);
