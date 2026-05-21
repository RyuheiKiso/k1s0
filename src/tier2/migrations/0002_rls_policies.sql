-- k1s0 tier2 migration: 0002_rls_policies
-- migration_id: 0002_rls_policies
-- description: テナント RLS ポリシーを全テーブルに適用する（10_テナント分離適合仕様.md RLS FORCE 要件）
-- safety_level: requires_lock
-- table_class: tenant_scoped
-- applied_at: 2026-05-17T00:00:00Z
--
-- 実行方法:
--   sqlx migrate run --database-url $DATABASE_URL --source src/tier2/migrations/

-- k1s0app ロールを作成する（アプリケーション接続に使用する）
-- GUC app.tenant_id を設定するため、このロールには SET SESSION AUTHORIZATION 権限を与えない
DO $$
BEGIN
    -- k1s0app ロールが存在しない場合のみ作成する
    IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'k1s0app') THEN
        -- アプリケーション接続用ロールを作成する（NOLOGIN: 直接ログイン禁止 / NOINHERIT: ロール継承禁止）
        -- spec §162「全 role は NOLOGIN NOINHERIT」に準拠する（Y-tier2-3 修正）
        CREATE ROLE k1s0app NOLOGIN NOINHERIT;
    END IF;
END;
$$;

-- k1s0app ロールに k1s0 スキーマへのアクセス権を付与する
GRANT USAGE ON SCHEMA k1s0 TO k1s0app;

-- k1s0app ロールに全テーブルへの SELECT / INSERT / UPDATE / DELETE 権限を付与する
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA k1s0 TO k1s0app;

-- domain_event テーブルに RLS を有効にする
ALTER TABLE k1s0.domain_event ENABLE ROW LEVEL SECURITY;
-- FORCE: スーパーユーザー以外は全て RLS ポリシーを適用する（バイパス禁止）
ALTER TABLE k1s0.domain_event FORCE ROW LEVEL SECURITY;

-- domain_event のテナント分離 RLS ポリシーを作成する
-- app.tenant_id GUC と domain_event.tenant_id が一致する行のみアクセスを許可する
-- 冪等化: PostgreSQL は CREATE POLICY IF NOT EXISTS を未サポートのため DROP + CREATE パターンを使用する
-- spec §168 冪等化規約準拠（Y-tier2-2 修正）
DROP POLICY IF EXISTS k1s0_domain_event_tenant_isolation ON k1s0.domain_event;
CREATE POLICY k1s0_domain_event_tenant_isolation
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
-- 冪等化: DROP + CREATE パターンで idempotent にする
DROP POLICY IF EXISTS k1s0_outbox_message_tenant_isolation ON k1s0.outbox_message;
CREATE POLICY k1s0_outbox_message_tenant_isolation
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
-- 冪等化: DROP + CREATE パターンで idempotent にする
DROP POLICY IF EXISTS k1s0_audit_event_tenant_isolation ON k1s0.audit_event;
CREATE POLICY k1s0_audit_event_tenant_isolation
    ON k1s0.audit_event
    -- 全操作に適用する
    AS RESTRICTIVE
    USING (tenant_id = current_setting('app.tenant_id', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.tenant_id', true)::uuid);

-- ============================================================
-- 管理者ロールの作成と k1s0 スキーマ権限付与
-- ============================================================

-- master_admin ロールを作成する（テナントマスタ管理者ロール）
DO $$
BEGIN
  -- master_admin ロールが存在しない場合のみ作成する
  IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'master_admin') THEN
    -- master_admin ロールを作成する（LOGIN 不可、NOINHERIT でロール継承を禁止する）
    CREATE ROLE master_admin NOLOGIN NOINHERIT;
  END IF;
END $$;

-- support_engineer ロールを作成する（PII 参照専用ロール）
DO $$
BEGIN
  -- support_engineer ロールが存在しない場合のみ作成する
  IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'support_engineer') THEN
    -- support_engineer ロールを作成する（LOGIN 不可、NOINHERIT でロール継承を禁止する）
    CREATE ROLE support_engineer NOLOGIN NOINHERIT;
  END IF;
END $$;

-- platform_admin ロールを作成する（プラットフォーム管理者ロール）
DO $$
BEGIN
  -- platform_admin ロールが存在しない場合のみ作成する
  IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'platform_admin') THEN
    -- platform_admin ロールを作成する（LOGIN 不可、NOINHERIT でロール継承を禁止する）
    CREATE ROLE platform_admin NOLOGIN NOINHERIT;
  END IF;
END $$;

-- 各ロールに k1s0 スキーマへの USAGE 権限を付与する
-- master_admin: テナントマスタ管理者ロール
GRANT USAGE ON SCHEMA k1s0 TO master_admin;
-- support_engineer: PII 参照専用ロール
GRANT USAGE ON SCHEMA k1s0 TO support_engineer;
-- platform_admin: プラットフォーム設定管理者ロール
GRANT USAGE ON SCHEMA k1s0 TO platform_admin;
