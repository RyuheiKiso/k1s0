-- 008_add_tenant_id_rls.up.sql のロールバック。
-- tenant_id カラム、インデックス、RLS ポリシーを削除し、マイグレーション前の状態に戻す。
-- 注意: ロールバック時に tenant_id に格納されたデータは失われる。実行前にバックアップを取得すること。

BEGIN;

-- FORCE ROW LEVEL SECURITY を解除する
ALTER TABLE saga.saga_step_logs NO FORCE ROW LEVEL SECURITY;
ALTER TABLE saga.saga_states NO FORCE ROW LEVEL SECURITY;

-- テナント分離ポリシーを削除する
DROP POLICY IF EXISTS tenant_isolation ON saga.saga_step_logs;
DROP POLICY IF EXISTS tenant_isolation ON saga.saga_states;

-- RLS を無効化する
ALTER TABLE saga.saga_step_logs DISABLE ROW LEVEL SECURITY;
ALTER TABLE saga.saga_states DISABLE ROW LEVEL SECURITY;

-- インデックスを削除する
DROP INDEX IF EXISTS saga.idx_saga_step_logs_tenant_id;
DROP INDEX IF EXISTS saga.idx_saga_states_tenant_workflow;
DROP INDEX IF EXISTS saga.idx_saga_states_tenant_id;

-- tenant_id カラムを削除する
ALTER TABLE saga.saga_step_logs DROP COLUMN IF EXISTS tenant_id;
ALTER TABLE saga.saga_states DROP COLUMN IF EXISTS tenant_id;

COMMIT;
