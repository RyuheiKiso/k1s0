-- saga_states / saga_step_logs の tenant_id を TEXT から VARCHAR(255) に戻す。

BEGIN;

SET LOCAL search_path TO saga, public;

DROP POLICY IF EXISTS tenant_isolation ON saga.saga_states;
DROP POLICY IF EXISTS tenant_isolation ON saga.saga_step_logs;

ALTER TABLE saga.saga_states
    ALTER COLUMN tenant_id TYPE VARCHAR(255) USING tenant_id::VARCHAR(255);

ALTER TABLE saga.saga_step_logs
    ALTER COLUMN tenant_id TYPE VARCHAR(255) USING tenant_id::VARCHAR(255);

-- 009 マイグレーション相当のポリシーを復元する
CREATE POLICY tenant_isolation ON saga.saga_states
    AS RESTRICTIVE
    USING (tenant_id::TEXT = current_setting('app.current_tenant_id', true)::TEXT)
    WITH CHECK (tenant_id::TEXT = current_setting('app.current_tenant_id', true)::TEXT);

CREATE POLICY tenant_isolation ON saga.saga_step_logs
    AS RESTRICTIVE
    USING (tenant_id::TEXT = current_setting('app.current_tenant_id', true)::TEXT)
    WITH CHECK (tenant_id::TEXT = current_setting('app.current_tenant_id', true)::TEXT);

COMMIT;
