DROP TRIGGER IF EXISTS trigger_tenants_update_updated_at ON tenant.tenants;
DROP INDEX IF EXISTS tenant.idx_tenants_status;
DROP INDEX IF EXISTS tenant.idx_tenants_name;
DROP TABLE IF EXISTS tenant.tenants;
