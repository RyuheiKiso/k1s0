-- テナント分離のための tenant_id カラム追加と RLS 設定
ALTER TABLE file_storage.file_metadata ADD COLUMN IF NOT EXISTS tenant_id TEXT NOT NULL DEFAULT 'system';

-- テナント別インデックス
CREATE INDEX IF NOT EXISTS idx_file_metadata_tenant_id ON file_storage.file_metadata(tenant_id);

-- Row Level Security を有効化
ALTER TABLE file_storage.file_metadata ENABLE ROW LEVEL SECURITY;
ALTER TABLE file_storage.file_metadata FORCE ROW LEVEL SECURITY;

-- テナント分離ポリシー（SELECT/UPDATE/DELETE/INSERT 全て）
CREATE POLICY tenant_isolation ON file_storage.file_metadata
    AS RESTRICTIVE
    USING (tenant_id::TEXT = current_setting('app.current_tenant_id', true)::TEXT)
    WITH CHECK (tenant_id::TEXT = current_setting('app.current_tenant_id', true)::TEXT);
