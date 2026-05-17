-- k1s0 tier2 session GUC 定義
-- PostgreSQL custom GUC（app.tenant_id / app.actor_id / app.purpose / app.delegation_chain）を
-- postgresql.conf または postgresql.auto.conf に追加する設定
-- CloudNativePG の additionalConfig セクションに記述する

-- ============================================================
-- 実 DDL: custom_variable_classes の設定（ALTER SYSTEM SET）
-- CloudNativePG では postgresql.auto.conf に書き込まれる
-- ============================================================

-- custom_variable_classes に 'app' を追加する（既存値がある場合はカンマ区切りで追加する）
-- NOTE: CloudNativePG の additionalConfig を使う場合は ALTER SYSTEM は不要（CNPG が自動設定する）
-- 直接 PostgreSQL を管理する場合のみ以下の ALTER SYSTEM を実行する
ALTER SYSTEM SET custom_variable_classes = 'app';

-- 設定をリロードする（ALTER SYSTEM は RELOAD で有効になる）
SELECT pg_reload_conf();

-- ============================================================
-- 実 DDL: role 作成と GUC アクセス権限の付与
-- ============================================================

-- k1s0app ロールが存在しない場合は作成する（アプリケーション接続ロール）
DO $$
BEGIN
  -- k1s0app ロールが存在しない場合は作成する
  IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'k1s0app') THEN
    -- k1s0app ロールを作成する（LOGIN 不可、NOINHERIT でロール継承を禁止する）
    CREATE ROLE k1s0app NOLOGIN NOINHERIT;
  END IF;
END $$;

-- master_admin ロールが存在しない場合は作成する（テナントマスタ管理者ロール）
DO $$
BEGIN
  -- master_admin ロールが存在しない場合は作成する
  IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'master_admin') THEN
    -- master_admin ロールを作成する（LOGIN 不可）
    CREATE ROLE master_admin NOLOGIN NOINHERIT;
  END IF;
END $$;

-- support_engineer ロールが存在しない場合は作成する（PII 参照専用ロール）
DO $$
BEGIN
  -- support_engineer ロールが存在しない場合は作成する
  IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'support_engineer') THEN
    -- support_engineer ロールを作成する（LOGIN 不可）
    CREATE ROLE support_engineer NOLOGIN NOINHERIT;
  END IF;
END $$;

-- platform_admin ロールが存在しない場合は作成する（プラットフォーム管理者ロール）
DO $$
BEGIN
  -- platform_admin ロールが存在しない場合は作成する
  IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'platform_admin') THEN
    -- platform_admin ロールを作成する（LOGIN 不可）
    CREATE ROLE platform_admin NOLOGIN NOINHERIT;
  END IF;
END $$;

-- k1s0 スキーマへの USAGE 権限を各ロールに付与する
-- k1s0app: アプリケーション操作ロール（テナントスコープテーブルへのアクセス）
GRANT USAGE ON SCHEMA k1s0 TO k1s0app;
-- master_admin: テナントマスタ管理者ロール
GRANT USAGE ON SCHEMA k1s0 TO master_admin;
-- support_engineer: PII 参照専用ロール
GRANT USAGE ON SCHEMA k1s0 TO support_engineer;
-- platform_admin: プラットフォーム設定管理者ロール
GRANT USAGE ON SCHEMA k1s0 TO platform_admin;

-- k1s0app に tenant_scoped テーブル（domain_event / outbox_message / audit_event）への権限を付与する
-- SELECT / INSERT は RLS ポリシーでさらに絞り込まれる
GRANT SELECT, INSERT ON TABLE k1s0.domain_event TO k1s0app;
GRANT SELECT, INSERT ON TABLE k1s0.outbox_message TO k1s0app;
GRANT SELECT, INSERT ON TABLE k1s0.audit_event TO k1s0app;

-- master_admin に tenant_master テーブルへの権限を付与する
GRANT SELECT, INSERT, UPDATE ON TABLE k1s0.tenant_master TO master_admin;

-- platform_admin に platform_config テーブルへの全権限を付与する
GRANT SELECT, INSERT, UPDATE, DELETE ON TABLE k1s0.platform_config TO platform_admin;
-- k1s0app / master_admin / support_engineer には platform_config の SELECT のみを付与する
GRANT SELECT ON TABLE k1s0.platform_config TO k1s0app, master_admin, support_engineer;

-- support_engineer に pii_data テーブルへの SELECT のみを付与する（INSERT/UPDATE は k1s0app 経由）
GRANT SELECT ON TABLE k1s0.pii_data TO support_engineer;
-- k1s0app に pii_data テーブルへの INSERT / UPDATE を付与する（SELECT は purpose check が必要）
GRANT INSERT, UPDATE ON TABLE k1s0.pii_data TO k1s0app;

-- ============================================================
-- 実 DDL: GUC fail-safe デフォルト値の設定（ALTER ROLE）
-- ============================================================

-- k1s0app の接続時デフォルト GUC を fail-safe 値に設定する
-- GUC が SET LOCAL で上書きされない場合は null 相当の値が返り RLS が 0 行を返す（fail-safe）
ALTER ROLE k1s0app SET app.tenant_id = '';
ALTER ROLE k1s0app SET app.actor_id = '';
ALTER ROLE k1s0app SET app.purpose = '';
ALTER ROLE k1s0app SET app.delegation_chain = '{}';

-- support_engineer の接続時デフォルト GUC を fail-safe 値に設定する
ALTER ROLE support_engineer SET app.tenant_id = '';
ALTER ROLE support_engineer SET app.actor_id = '';
ALTER ROLE support_engineer SET app.purpose = '';
ALTER ROLE support_engineer SET app.delegation_chain = '{}';

-- ============================================================
-- GUC 定義（postgresql.conf に追記する形式）
-- ============================================================

-- app.tenant_id: テナント UUID を保持する GUC
-- SET LOCAL app.tenant_id = '<uuid>';
-- custom_variable_classes = 'app'  # postgresql.conf で要設定

-- app.actor_id: Keycloak subject（認証済み actor の識別子）
-- SET LOCAL app.actor_id = '<keycloak-sub>';

-- app.purpose: セッション目的（business_op / support / export / migration / emergency）
-- SET LOCAL app.purpose = 'business_op';

-- app.delegation_chain: 委譲元 user の配列（空配列 '{}' が通常）
-- SET LOCAL app.delegation_chain = '{}';

-- ============================================================
-- GUC の fail-safe デフォルト設定
-- GUC が SET されていない場合に RLS が 0 行を返す動作を確保する
-- ============================================================

-- GUC 未設定時の current_setting('app.tenant_id', true) は null を返す
-- tenant_id = null::uuid は常に false になるため RLS が 0 行を返す（fail-safe）

-- GUC 型検証クエリ（kind cluster 適用後に実行して確認する）
-- SELECT
--   current_setting('app.tenant_id', true) AS tenant_id,
--   current_setting('app.actor_id', true)  AS actor_id,
--   current_setting('app.purpose', true)   AS purpose,
--   current_setting('app.delegation_chain', true) AS delegation_chain;

-- ============================================================
-- CloudNativePG additionalConfig 形式（cnpg cluster.yaml に記述する）
-- ============================================================
-- additionalConfig:
--   custom_variable_classes: "app"

-- ============================================================
-- tier2 テスト用 GUC 設定スクリプト（kind cluster 確認用）
-- ============================================================

-- tenant_a のコンテキストを設定する
-- BEGIN;
-- SET LOCAL app.tenant_id = '00000000-0000-0000-0000-000000000001';
-- SET LOCAL app.actor_id = 'keycloak-sub-tenant-a';
-- SET LOCAL app.purpose = 'business_op';
-- SET LOCAL app.delegation_chain = '{}';
-- SELECT * FROM k1s0.domain_event;  -- tenant_a のデータのみが返ることを確認する
-- ROLLBACK;

-- tenant_b のコンテキストを設定して tenant_a のデータが取得できないことを確認する
-- BEGIN;
-- SET LOCAL app.tenant_id = '00000000-0000-0000-0000-000000000002';
-- SET LOCAL app.actor_id = 'keycloak-sub-tenant-b';
-- SET LOCAL app.purpose = 'business_op';
-- SET LOCAL app.delegation_chain = '{}';
-- SELECT COUNT(*) FROM k1s0.domain_event WHERE tenant_id = '00000000-0000-0000-0000-000000000001';
-- 期待値: 0（tenant_b から tenant_a のデータは見えない）
-- ROLLBACK;
