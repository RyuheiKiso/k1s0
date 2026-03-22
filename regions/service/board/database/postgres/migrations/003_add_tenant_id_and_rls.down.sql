SET search_path TO board_service;
ALTER TABLE board_columns DISABLE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS tenant_isolation ON board_columns;
ALTER TABLE board_columns DROP COLUMN IF EXISTS tenant_id;
