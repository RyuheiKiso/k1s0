BEGIN;
DROP TRIGGER IF EXISTS trigger_access_policies_update_updated_at ON vault.access_policies;
ALTER TABLE vault.access_policies DROP COLUMN IF EXISTS updated_at;
-- update_updated_at 関数は他のトリガーで使用されている可能性があるため削除しない
COMMIT;
