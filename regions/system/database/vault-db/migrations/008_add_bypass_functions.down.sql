-- 008_add_bypass_functions.up.sql のロールバック
BEGIN;

DROP FUNCTION IF EXISTS vault.list_access_logs_all_tenants(UUID, BIGINT);

COMMIT;
