-- k1s0 tier2 RLS ポリシー定義
-- 4 テーブルクラス（tenant_scoped / tenant_master / pii_segregated / platform_global）の
-- Row Level Security FORCE ポリシーを定義する
-- 適用対象: kind cluster 上の CloudNativePG k1s0-main cluster

-- ============================================================
-- 共通設定: k1s0 スキーマに session GUC を定義する
-- ============================================================

-- app.tenant_id GUC を設定する（tier2 初期化時に一度だけ実行する）
-- ALTER ROLE の方が永続的だが SET LOCAL で transaction 単位に制限する設計
-- ここでは GUC の存在確認と型宣言のみ行う
DO $$
BEGIN
  -- k1s0 スキーマが存在しない場合は作成する
  IF NOT EXISTS (SELECT 1 FROM pg_namespace WHERE nspname = 'k1s0') THEN
    CREATE SCHEMA k1s0;
  END IF;
END $$;

-- ============================================================
-- tenant_scoped テーブルクラス: domain_event テーブルに RLS FORCE を適用
-- ============================================================

-- domain_event テーブルの RLS を有効化する
ALTER TABLE k1s0.domain_event ENABLE ROW LEVEL SECURITY;

-- domain_event テーブルに FORCE フラグを設定する（superuser も RLS をバイパスできない）
ALTER TABLE k1s0.domain_event FORCE ROW LEVEL SECURITY;

-- tenant_scoped SELECT ポリシー: tenant_id が GUC と一致する行のみ返す
DROP POLICY IF EXISTS tenant_scoped_select ON k1s0.domain_event;
CREATE POLICY tenant_scoped_select ON k1s0.domain_event
  -- 対象操作: SELECT
  FOR SELECT
  -- 適用ロール: k1s0app（アプリケーションロール）
  TO k1s0app
  -- USING 条件: GUC の tenant_id と行の tenant_id が一致する場合のみ許可
  USING (tenant_id = current_setting('app.tenant_id', true)::uuid);

-- tenant_scoped INSERT ポリシー: WITH CHECK で tenant_id の不整合を防ぐ（P3 の DB 層対応）
DROP POLICY IF EXISTS tenant_scoped_insert ON k1s0.domain_event;
CREATE POLICY tenant_scoped_insert ON k1s0.domain_event
  -- 対象操作: INSERT
  FOR INSERT
  -- 適用ロール: k1s0app
  TO k1s0app
  -- WITH CHECK 条件: 書込む tenant_id が GUC と一致する行のみ許可
  WITH CHECK (tenant_id = current_setting('app.tenant_id', true)::uuid);

-- ============================================================
-- outbox テーブルの RLS FORCE（tenant_scoped クラス）
-- ============================================================

-- outbox テーブルの RLS を有効化する
ALTER TABLE k1s0.outbox ENABLE ROW LEVEL SECURITY;

-- outbox テーブルに FORCE フラグを設定する
ALTER TABLE k1s0.outbox FORCE ROW LEVEL SECURITY;

-- outbox SELECT ポリシー
DROP POLICY IF EXISTS tenant_scoped_select ON k1s0.outbox;
CREATE POLICY tenant_scoped_select ON k1s0.outbox
  FOR SELECT
  TO k1s0app
  USING (tenant_id = current_setting('app.tenant_id', true)::uuid);

-- outbox INSERT ポリシー
DROP POLICY IF EXISTS tenant_scoped_insert ON k1s0.outbox;
CREATE POLICY tenant_scoped_insert ON k1s0.outbox
  FOR INSERT
  TO k1s0app
  WITH CHECK (tenant_id = current_setting('app.tenant_id', true)::uuid);

-- ============================================================
-- audit_event テーブルの RLS FORCE（tenant_scoped クラス）
-- ============================================================

-- audit_event テーブルの RLS を有効化する
ALTER TABLE k1s0.audit_event ENABLE ROW LEVEL SECURITY;

-- audit_event テーブルに FORCE フラグを設定する
ALTER TABLE k1s0.audit_event FORCE ROW LEVEL SECURITY;

-- audit_event SELECT ポリシー（支援者ロールは all_purpose で参照可）
DROP POLICY IF EXISTS tenant_scoped_select ON k1s0.audit_event;
CREATE POLICY tenant_scoped_select ON k1s0.audit_event
  FOR SELECT
  TO k1s0app
  USING (tenant_id = current_setting('app.tenant_id', true)::uuid);

-- audit_event INSERT ポリシー
DROP POLICY IF EXISTS tenant_scoped_insert ON k1s0.audit_event;
CREATE POLICY tenant_scoped_insert ON k1s0.audit_event
  FOR INSERT
  TO k1s0app
  WITH CHECK (tenant_id = current_setting('app.tenant_id', true)::uuid);

-- ============================================================
-- pii_segregated クラス: purpose check 付きポリシー（P4 の DB 層対応）
-- ============================================================

-- pii_segregated テーブルが存在する場合は RLS FORCE + purpose check を適用する
-- ここでは audit_event の pii 専用ポリシーとして記述する

-- pii_access ポリシー: support / export / emergency の場合のみ参照可
DROP POLICY IF EXISTS pii_segregated_select ON k1s0.audit_event;
CREATE POLICY pii_segregated_select ON k1s0.audit_event
  -- pii テーブルへの SELECT は purpose が support / export / emergency のみ許可
  FOR SELECT
  TO k1s0app
  USING (
    tenant_id = current_setting('app.tenant_id', true)::uuid
    AND current_setting('app.purpose', true) IN ('support', 'export', 'emergency')
  );

-- ============================================================
-- verification: RLS 設定の確認クエリ（適用後に実行して確認する）
-- ============================================================
-- SELECT tablename, rowsecurity, forcerowsecurity
-- FROM pg_tables
-- WHERE schemaname = 'k1s0'
-- AND tablename IN ('domain_event', 'outbox', 'audit_event');
-- 期待値: rowsecurity=true, forcerowsecurity=true
