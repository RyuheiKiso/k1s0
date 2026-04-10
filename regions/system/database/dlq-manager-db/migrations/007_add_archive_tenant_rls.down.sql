-- dlq.dlq_messages_archive のテナント分離を元に戻す。

BEGIN;

SET LOCAL search_path TO dlq, public;

DROP POLICY IF EXISTS tenant_isolation ON dlq.dlq_messages_archive;
ALTER TABLE dlq.dlq_messages_archive DISABLE ROW LEVEL SECURITY;
DROP INDEX IF EXISTS dlq.idx_dlq_messages_archive_tenant_id;
ALTER TABLE dlq.dlq_messages_archive DROP COLUMN IF EXISTS tenant_id;

COMMIT;
