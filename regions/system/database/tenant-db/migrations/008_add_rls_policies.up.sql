-- テナント分離のためRow Level Securityを有効化する
-- アプリケーション層でのテナントIDフィルタ失敗時にもデータ保護を保証する（ADR-0054準拠）
-- 既存の saga-db / event-store-db / auth-db / session-db パターンに統一する

BEGIN;

-- tenants テーブルの RLS を有効化する
-- テナントテーブル自体は id が主キーであり、
-- app.current_tenant_id セッション変数と id を照合してテナント分離を実現する
ALTER TABLE tenant.tenants ENABLE ROW LEVEL SECURITY;

-- テナントポリシー: 現在のアプリケーションテナントIDと一致する行のみアクセス可能
-- current_setting の第 2 引数 true = 変数未設定時に NULL を返しエラーを回避する
DROP POLICY IF EXISTS tenant_isolation_policy ON tenant.tenants;
CREATE POLICY tenant_isolation_policy ON tenant.tenants
  USING (id::text = current_setting('app.current_tenant_id', true));

-- スーパーユーザー・テーブルオーナーも RLS の適用対象とする（バイパスを防止）
ALTER TABLE tenant.tenants FORCE ROW LEVEL SECURITY;

-- tenant_members テーブルの RLS を有効化する
-- tenant_id（UUID）と app.current_tenant_id（text）を照合してテナント分離を実現する
ALTER TABLE tenant.tenant_members ENABLE ROW LEVEL SECURITY;

-- テナントメンバーポリシー: テナントIDで絞り込み
-- tenant_id は UUID 型のため text にキャストして比較する
DROP POLICY IF EXISTS tenant_member_isolation_policy ON tenant.tenant_members;
CREATE POLICY tenant_member_isolation_policy ON tenant.tenant_members
  USING (tenant_id::text = current_setting('app.current_tenant_id', true));

-- スーパーユーザー・テーブルオーナーも RLS の適用対象とする（バイパスを防止）
ALTER TABLE tenant.tenant_members FORCE ROW LEVEL SECURITY;

-- サービスアカウント（アプリ）に設定変更権限を付与する
-- PostgreSQL 15 以降で利用可能。アプリが SET app.current_tenant_id を実行できるようにする。
GRANT SET ON PARAMETER app.current_tenant_id TO PUBLIC;

COMMIT;
