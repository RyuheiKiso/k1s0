-- k1s0 tier2 migration: 0002_rls_policies
-- テナント RLS ポリシーを全テーブルに適用する
-- 10_テナント分離適合仕様.md の RLS FORCE 要件を物理化する
--
-- 実行方法:
--   sqlx migrate run --database-url $DATABASE_URL --source src/tier2/migrations/

-- k1s0_app ロールを作成する（アプリケーション接続に使用する）
-- GUC app.tenant_id を設定するため、このロールには SET SESSION AUTHORIZATION 権限を与えない
DO $$
BEGIN
    -- k1s0_app ロールが存在しない場合のみ作成する
    IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'k1s0_app') THEN
        -- アプリケーション接続用ロールを作成する（ログインは許可しない）
        CREATE ROLE k1s0_app NOLOGIN;
    END IF;
END;
$$;

-- k1s0_app ロールに k1s0 スキーマへのアクセス権を付与する
GRANT USAGE ON SCHEMA k1s0 TO k1s0_app;

-- k1s0_app ロールに全テーブルへの SELECT / INSERT / UPDATE / DELETE 権限を付与する
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA k1s0 TO k1s0_app;

-- domain_event テーブルに RLS を有効にする
ALTER TABLE k1s0.domain_event ENABLE ROW LEVEL SECURITY;
-- FORCE: スーパーユーザー以外は全て RLS ポリシーを適用する（バイパス禁止）
ALTER TABLE k1s0.domain_event FORCE ROW LEVEL SECURITY;

-- domain_event のテナント分離 RLS ポリシーを作成する
-- app.tenant_id GUC と domain_event.tenant_id が一致する行のみアクセスを許可する
CREATE POLICY IF NOT EXISTS k1s0_domain_event_tenant_isolation
    ON k1s0.domain_event
    -- 全操作（SELECT / INSERT / UPDATE / DELETE）に適用する
    AS RESTRICTIVE
    USING (tenant_id = current_setting('app.tenant_id', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.tenant_id', true)::uuid);

-- outbox_message テーブルに RLS を有効にする
ALTER TABLE k1s0.outbox_message ENABLE ROW LEVEL SECURITY;
-- FORCE: スーパーユーザー以外は全て RLS ポリシーを適用する（バイパス禁止）
ALTER TABLE k1s0.outbox_message FORCE ROW LEVEL SECURITY;

-- outbox_message のテナント分離 RLS ポリシーを作成する
-- app.tenant_id GUC と outbox_message.tenant_id が一致する行のみアクセスを許可する
CREATE POLICY IF NOT EXISTS k1s0_outbox_message_tenant_isolation
    ON k1s0.outbox_message
    -- 全操作に適用する
    AS RESTRICTIVE
    USING (tenant_id = current_setting('app.tenant_id', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.tenant_id', true)::uuid);

-- audit_event テーブルに RLS を有効にする
ALTER TABLE k1s0.audit_event ENABLE ROW LEVEL SECURITY;
-- FORCE: スーパーユーザー以外は全て RLS ポリシーを適用する（バイパス禁止）
ALTER TABLE k1s0.audit_event FORCE ROW LEVEL SECURITY;

-- audit_event のテナント分離 RLS ポリシーを作成する
-- app.tenant_id GUC と audit_event.tenant_id が一致する行のみアクセスを許可する
CREATE POLICY IF NOT EXISTS k1s0_audit_event_tenant_isolation
    ON k1s0.audit_event
    -- 全操作に適用する
    AS RESTRICTIVE
    USING (tenant_id = current_setting('app.tenant_id', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.tenant_id', true)::uuid);
