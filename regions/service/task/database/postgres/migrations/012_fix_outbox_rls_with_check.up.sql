-- HIGH-009 監査対応: outbox_events RLS に FORCE ROW LEVEL SECURITY + AS RESTRICTIVE + WITH CHECK を追加する。
-- 009_add_outbox_rls.up.sql で作成したポリシーは USING 句のみで WITH CHECK が欠落しており、
-- テナント分離設定済みセッションから異テナント ID のアウトボックスイベントを INSERT できる脆弱性がある。
-- 本マイグレーションで INSERT/UPDATE 時のテナント検証を強制する。
--
-- ポリシー設計:
--   USING: バックグラウンドパブリッシャー（set_config 未呼出し）からの READ は全テナント分許可する。
--          アウトボックスポーラーは tenant_id を設定せずに全テナントのイベントを取得する必要があるため。
--   WITH CHECK: INSERT/UPDATE は必ず set_config によるテナント設定を要求し、
--               現在のセッションテナント ID と一致するレコードのみ書き込みを許可する。
--
-- lessons.md: マイグレーション内では SET LOCAL search_path TO <schema>, public; を使用する。
SET LOCAL search_path TO task_service, public;

-- FORCE ROW LEVEL SECURITY を有効化する（テーブルオーナーにも RLS を適用する）
ALTER TABLE outbox_events FORCE ROW LEVEL SECURITY;

-- 既存ポリシーを削除して再作成する（DROP + CREATE はアトミックに実行される）
DROP POLICY IF EXISTS tenant_isolation ON outbox_events;

-- AS RESTRICTIVE: 他の PERMISSIVE ポリシーが存在しても本ポリシーは必ず適用される
-- USING: NULL 値許可はバックグラウンドパブリッシャーの読み取りアクセスのためにのみ使用する
-- WITH CHECK: INSERT/UPDATE は必ず app.current_tenant_id の設定を要求する
CREATE POLICY tenant_isolation ON outbox_events
    AS RESTRICTIVE
    USING (
        current_setting('app.current_tenant_id', true) IS NULL
        OR tenant_id = current_setting('app.current_tenant_id', true)::TEXT
    )
    WITH CHECK (
        tenant_id = current_setting('app.current_tenant_id', true)::TEXT
    );
