-- WITH CHECK 付き RLS ポリシーを USING のみのポリシーに戻す（ロールバック用）
BEGIN;

-- dlq_messages テーブルの tenant_isolation ポリシーを USING のみで再作成する
DROP POLICY IF EXISTS tenant_isolation ON dlq.dlq_messages;
CREATE POLICY tenant_isolation ON dlq.dlq_messages
    USING (tenant_id::TEXT = current_setting('app.current_tenant_id', true)::TEXT);

COMMIT;
