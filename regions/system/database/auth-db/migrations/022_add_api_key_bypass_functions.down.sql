-- 022_add_api_key_bypass_functions.up.sql の ロールバック
BEGIN;

DROP FUNCTION IF EXISTS auth.api_key_find_by_prefix(TEXT);
DROP FUNCTION IF EXISTS auth.api_key_find_by_id(UUID);
DROP FUNCTION IF EXISTS auth.api_key_revoke(UUID);
DROP FUNCTION IF EXISTS auth.api_key_delete(UUID);

COMMIT;
