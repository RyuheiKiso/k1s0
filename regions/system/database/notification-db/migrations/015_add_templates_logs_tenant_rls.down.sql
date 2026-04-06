-- notification.templates と notification.notification_logs のテナント分離を元に戻す。

BEGIN;

SET LOCAL search_path TO notification, public;

-- notification_logs の RLS 無効化とポリシー削除
DROP POLICY IF EXISTS tenant_isolation ON notification.notification_logs;
ALTER TABLE notification.notification_logs DISABLE ROW LEVEL SECURITY;
DROP INDEX IF EXISTS notification.idx_notification_logs_tenant_id;
ALTER TABLE notification.notification_logs DROP COLUMN IF EXISTS tenant_id;

-- templates の RLS 無効化とポリシー削除
DROP POLICY IF EXISTS tenant_isolation ON notification.templates;
ALTER TABLE notification.templates DISABLE ROW LEVEL SECURITY;
DROP INDEX IF EXISTS notification.idx_templates_tenant_id;
ALTER TABLE notification.templates DROP COLUMN IF EXISTS tenant_id;

COMMIT;
