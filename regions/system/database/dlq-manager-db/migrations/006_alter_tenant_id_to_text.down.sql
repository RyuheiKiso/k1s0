-- dlq_messages の tenant_id を TEXT から VARCHAR(255) に戻す。
-- RLS ポリシーを DROP してから型変更し、AS RESTRICTIVE + WITH CHECK で再作成する。

BEGIN;

SET LOCAL search_path TO dlq, public;

DROP POLICY IF EXISTS tenant_isolation ON dlq.dlq_messages;

ALTER TABLE dlq.dlq_messages
    ALTER COLUMN tenant_id TYPE VARCHAR(255) USING tenant_id::VARCHAR(255);

-- 005 マイグレーション相当のポリシーを復元する（::TEXT キャスト付き）
CREATE POLICY tenant_isolation ON dlq.dlq_messages
    AS RESTRICTIVE
    USING (tenant_id::TEXT = current_setting('app.current_tenant_id', true)::TEXT)
    WITH CHECK (tenant_id::TEXT = current_setting('app.current_tenant_id', true)::TEXT);

COMMIT;
