-- 004_add_tenant_id_rls.up.sql のロールバック: RLS ポリシーと tenant_id カラムを削除する

BEGIN;

-- dlq_messages の RLS を無効化する
ALTER TABLE dlq.dlq_messages DISABLE ROW LEVEL SECURITY;
ALTER TABLE dlq.dlq_messages NO FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS tenant_isolation ON dlq.dlq_messages;

-- インデックスを削除する
DROP INDEX IF EXISTS dlq.idx_dlq_messages_tenant_id;
DROP INDEX IF EXISTS dlq.idx_dlq_messages_tenant_status;

-- tenant_id カラムを削除する
ALTER TABLE dlq.dlq_messages DROP COLUMN IF EXISTS tenant_id;

COMMIT;
