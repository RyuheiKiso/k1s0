-- テナント分離ポリシーとカラムのロールバック
DROP POLICY IF EXISTS tenant_isolation ON file_storage.file_metadata;
ALTER TABLE file_storage.file_metadata DISABLE ROW LEVEL SECURITY;
ALTER TABLE file_storage.file_metadata DROP COLUMN IF EXISTS tenant_id;
