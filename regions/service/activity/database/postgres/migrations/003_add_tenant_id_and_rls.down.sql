SET search_path TO activity_service;
ALTER TABLE activities DISABLE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS tenant_isolation ON activities;
ALTER TABLE activities DROP COLUMN IF EXISTS tenant_id;
