-- 012 のロールバック: RLS ポリシーと tenant_id カラムを削除する
BEGIN;

DROP POLICY IF EXISTS tenant_isolation ON notification.channels;
ALTER TABLE notification.channels DISABLE ROW LEVEL SECURITY;
DROP INDEX IF EXISTS idx_channels_tenant_id;
ALTER TABLE notification.channels DROP COLUMN IF EXISTS tenant_id;

COMMIT;
