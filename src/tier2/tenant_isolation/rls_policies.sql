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
ALTER TABLE k1s0.outbox_message ENABLE ROW LEVEL SECURITY;

-- outbox テーブルに FORCE フラグを設定する
ALTER TABLE k1s0.outbox_message FORCE ROW LEVEL SECURITY;

-- outbox SELECT ポリシー
DROP POLICY IF EXISTS tenant_scoped_select ON k1s0.outbox_message;
CREATE POLICY tenant_scoped_select ON k1s0.outbox_message
  FOR SELECT
  TO k1s0app
  USING (tenant_id = current_setting('app.tenant_id', true)::uuid);

-- outbox INSERT ポリシー
DROP POLICY IF EXISTS tenant_scoped_insert ON k1s0.outbox_message;
CREATE POLICY tenant_scoped_insert ON k1s0.outbox_message
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
-- tenant_master テーブルクラス: テナントマスタ管理テーブルへの RLS FORCE + role 制限
-- 対象: k1s0.tenant_master テーブル（テナントの登録・停止・メタデータ管理）
-- ============================================================

-- tenant_master テーブルの RLS を有効化する
ALTER TABLE k1s0.tenant_master ENABLE ROW LEVEL SECURITY;

-- tenant_master テーブルに FORCE フラグを設定する（superuser も RLS をバイパスできない）
ALTER TABLE k1s0.tenant_master FORCE ROW LEVEL SECURITY;

-- tenant_master SELECT ポリシー: master_admin role のみ参照可（tenant_id フィルタあり）
-- platform_admin は ALL テナントを参照できるが、k1s0app はアクセス禁止
DROP POLICY IF EXISTS tenant_master_select ON k1s0.tenant_master;
CREATE POLICY tenant_master_select ON k1s0.tenant_master
  -- 対象操作: SELECT
  FOR SELECT
  -- 適用ロール: master_admin のみ（テナントマスタ管理者ロール）
  TO master_admin
  -- USING 条件: GUC の tenant_id と行の tenant_id が一致する場合のみ許可
  USING (tenant_id = current_setting('app.tenant_id', true)::uuid);

-- tenant_master INSERT ポリシー: master_admin role のみ書込可（WITH CHECK で tenant_id 整合を保証）
DROP POLICY IF EXISTS tenant_master_insert ON k1s0.tenant_master;
CREATE POLICY tenant_master_insert ON k1s0.tenant_master
  -- 対象操作: INSERT
  FOR INSERT
  -- 適用ロール: master_admin のみ
  TO master_admin
  -- WITH CHECK 条件: 書込む tenant_id が GUC と一致する行のみ許可
  WITH CHECK (tenant_id = current_setting('app.tenant_id', true)::uuid);

-- tenant_master UPDATE ポリシー: master_admin role のみ更新可（USING + WITH CHECK で二重保護）
DROP POLICY IF EXISTS tenant_master_update ON k1s0.tenant_master;
CREATE POLICY tenant_master_update ON k1s0.tenant_master
  -- 対象操作: UPDATE
  FOR UPDATE
  -- 適用ロール: master_admin のみ
  TO master_admin
  -- USING 条件: 更新対象行の tenant_id が GUC と一致する場合のみ許可
  USING (tenant_id = current_setting('app.tenant_id', true)::uuid)
  -- WITH CHECK 条件: 更新後の tenant_id が GUC と一致する場合のみ許可
  WITH CHECK (tenant_id = current_setting('app.tenant_id', true)::uuid);

-- ============================================================
-- platform_global テーブルクラス: 全テナント共通データへのポリシー
-- 対象: k1s0.platform_config テーブル（SaaS プラットフォーム共通設定）
-- ============================================================

-- platform_config テーブルの RLS を有効化する
ALTER TABLE k1s0.platform_config ENABLE ROW LEVEL SECURITY;

-- platform_config テーブルに FORCE フラグを設定する
ALTER TABLE k1s0.platform_config FORCE ROW LEVEL SECURITY;

-- platform_global SELECT ポリシー: 全ロールが参照可（tenant_id 列が存在しない / テナント非依存）
-- k1s0app / master_admin / support_engineer の全ロールが参照可
DROP POLICY IF EXISTS platform_global_select ON k1s0.platform_config;
CREATE POLICY platform_global_select ON k1s0.platform_config
  -- 対象操作: SELECT
  FOR SELECT
  -- 適用ロール: k1s0app, master_admin, support_engineer（全テナントが参照可）
  TO k1s0app, master_admin, support_engineer
  -- USING 条件: テナント非依存のため常に true（全行が参照可）
  USING (true);

-- platform_global INSERT ポリシー: platform_admin role のみ書込可
DROP POLICY IF EXISTS platform_global_insert ON k1s0.platform_config;
CREATE POLICY platform_global_insert ON k1s0.platform_config
  -- 対象操作: INSERT
  FOR INSERT
  -- 適用ロール: platform_admin のみ（プラットフォーム管理者ロール）
  TO platform_admin
  -- WITH CHECK 条件: プラットフォームグローバルデータは tenant_id 列が存在しないため常に true
  WITH CHECK (true);

-- platform_global UPDATE ポリシー: platform_admin role のみ更新可
DROP POLICY IF EXISTS platform_global_update ON k1s0.platform_config;
CREATE POLICY platform_global_update ON k1s0.platform_config
  -- 対象操作: UPDATE
  FOR UPDATE
  -- 適用ロール: platform_admin のみ
  TO platform_admin
  -- USING + WITH CHECK 条件: 常に true（テナント非依存）
  USING (true)
  WITH CHECK (true);

-- ============================================================
-- pii_support_only ポリシー: support_engineer role + purpose check（P4 の細粒度制御）
-- 対象: k1s0.pii_data テーブル（PII 隔離クラスの専用テーブル）
-- ============================================================

-- pii_data テーブルの RLS を有効化する
ALTER TABLE k1s0.pii_data ENABLE ROW LEVEL SECURITY;

-- pii_data テーブルに FORCE フラグを設定する（superuser も RLS をバイパスできない）
ALTER TABLE k1s0.pii_data FORCE ROW LEVEL SECURITY;

-- pii_support_only SELECT ポリシー: support_engineer role + purpose が support / emergency の場合のみ参照可
-- P4: PII へのアクセスは目的（purpose）と役割（role）の両方で制限する
DROP POLICY IF EXISTS pii_support_only_select ON k1s0.pii_data;
CREATE POLICY pii_support_only_select ON k1s0.pii_data
  -- 対象操作: SELECT
  FOR SELECT
  -- 適用ロール: support_engineer のみ（通常業務ロール k1s0app は PII 参照禁止）
  TO support_engineer
  -- USING 条件: tenant_id 一致 + purpose が support / emergency の場合のみ許可
  USING (
    tenant_id = current_setting('app.tenant_id', true)::uuid
    AND current_setting('app.purpose', true) IN ('support', 'emergency')
  );

-- pii_data INSERT ポリシー: k1s0app ロールが自テナントの PII を書込可（purpose チェックあり）
DROP POLICY IF EXISTS pii_support_only_insert ON k1s0.pii_data;
CREATE POLICY pii_support_only_insert ON k1s0.pii_data
  -- 対象操作: INSERT
  FOR INSERT
  -- 適用ロール: k1s0app（PII 書込は通常業務経路のみ許可）
  TO k1s0app
  -- WITH CHECK 条件: tenant_id が GUC と一致する場合のみ許可（purpose 制限なし、書込は許可）
  WITH CHECK (tenant_id = current_setting('app.tenant_id', true)::uuid);

-- pii_data UPDATE ポリシー: k1s0app ロールが自テナントの PII を更新可
DROP POLICY IF EXISTS pii_support_only_update ON k1s0.pii_data;
CREATE POLICY pii_support_only_update ON k1s0.pii_data
  -- 対象操作: UPDATE
  FOR UPDATE
  -- 適用ロール: k1s0app
  TO k1s0app
  -- USING 条件: 更新対象行の tenant_id が GUC と一致する場合のみ許可
  USING (tenant_id = current_setting('app.tenant_id', true)::uuid)
  -- WITH CHECK 条件: 更新後の tenant_id が GUC と一致する場合のみ許可
  WITH CHECK (tenant_id = current_setting('app.tenant_id', true)::uuid);

-- ============================================================
-- verification: RLS 設定の確認クエリ（適用後に実行して確認する）
-- ============================================================
-- SELECT tablename, rowsecurity, forcerowsecurity
-- FROM pg_tables
-- WHERE schemaname = 'k1s0'
-- AND tablename IN ('domain_event', 'outbox', 'audit_event', 'tenant_master', 'platform_config', 'pii_data');
-- 期待値: rowsecurity=true, forcerowsecurity=true（全テーブル）
