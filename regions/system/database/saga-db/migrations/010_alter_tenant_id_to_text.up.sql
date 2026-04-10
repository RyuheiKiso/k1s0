-- saga_states / saga_step_logs テーブルの tenant_id を VARCHAR(255) から TEXT 型に変更する。
-- CRITICAL-DB-002 監査対応: 全サービスで tenant_id を TEXT 型に統一する（ADR-0093）。
-- 既存の RLS ポリシーを DROP してから型変更し、AS RESTRICTIVE + WITH CHECK で再作成する。
-- workflow_definitions の tenant_id 追加は別マイグレーション（011）で対応する。

BEGIN;

SET LOCAL search_path TO saga, public;

-- 既存の RLS ポリシーを先に削除する
DROP POLICY IF EXISTS tenant_isolation ON saga.saga_states;
DROP POLICY IF EXISTS tenant_isolation ON saga.saga_step_logs;

-- saga_states テーブルの tenant_id を TEXT 型に変更する
ALTER TABLE saga.saga_states
    ALTER COLUMN tenant_id TYPE TEXT USING tenant_id::TEXT;

-- saga_step_logs テーブルの tenant_id を TEXT 型に変更する
ALTER TABLE saga.saga_step_logs
    ALTER COLUMN tenant_id TYPE TEXT USING tenant_id::TEXT;

-- saga_states テーブルのポリシーを再作成する
CREATE POLICY tenant_isolation ON saga.saga_states
    AS RESTRICTIVE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

-- saga_step_logs テーブルのポリシーを再作成する
CREATE POLICY tenant_isolation ON saga.saga_step_logs
    AS RESTRICTIVE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

COMMIT;
