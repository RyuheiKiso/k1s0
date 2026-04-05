-- featureflag の tenant_id を UUID から TEXT に変更する
-- CRIT-002 監査対応: RLS ポリシーを先に DROP してから型変更し、WITH CHECK 付きで再作成する
-- PostgreSQL は RLS ポリシーが参照するカラムの型変更を禁止するため、順序が重要
-- 参照: ADR-0093（tenant_id TEXT 統一方針）
SET LOCAL search_path TO featureflag, public;

-- Step 1: 既存の RLS ポリシーを先に削除する（型変更の前提条件）
-- migration 005 で作成したポリシーが tenant_id::TEXT を参照しているため先に DROP が必要
DROP POLICY IF EXISTS tenant_isolation ON featureflag.feature_flags;
DROP POLICY IF EXISTS tenant_isolation ON featureflag.flag_audit_logs;

-- Step 2: tenant_id カラムを UUID から TEXT に変更する
-- USING 句で既存 UUID 値を文字列形式に変換する
-- デフォルト値を 'system' に設定して他サービスと統一する
ALTER TABLE featureflag.feature_flags
    ALTER COLUMN tenant_id TYPE TEXT USING tenant_id::TEXT;
ALTER TABLE featureflag.feature_flags
    ALTER COLUMN tenant_id SET DEFAULT 'system';

ALTER TABLE featureflag.flag_audit_logs
    ALTER COLUMN tenant_id TYPE TEXT USING tenant_id::TEXT;
ALTER TABLE featureflag.flag_audit_logs
    ALTER COLUMN tenant_id SET DEFAULT 'system';

-- Step 3: テナント分離 RLS ポリシーを WITH CHECK 付きで再作成する
-- TEXT 型になったため ::TEXT キャストが不要になった
-- MEDIUM-007 対応: WITH CHECK 句を追加して INSERT 時のテナント検証も行う
CREATE POLICY tenant_isolation ON featureflag.feature_flags
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

CREATE POLICY tenant_isolation ON featureflag.flag_audit_logs
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));
