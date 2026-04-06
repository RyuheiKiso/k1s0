-- HIGH-009 監査対応: outbox_events RLS に FORCE ROW LEVEL SECURITY + AS RESTRICTIVE + WITH CHECK を追加する。
-- 004_add_outbox_rls.up.sql で作成したポリシーは USING 句のみで WITH CHECK が欠落しており、
-- テナント分離設定済みセッションから異テナント ID のアウトボックスイベントを INSERT できる脆弱性がある。
-- 本マイグレーションで INSERT/UPDATE 時のテナント検証を強制する。
--
-- ポリシー設計:
--   USING: バックグラウンドパブリッシャー（set_config 未呼出し）からの READ は全テナント分許可する。
--   WITH CHECK: INSERT/UPDATE は必ず set_config によるテナント設定を要求する。
--
-- lessons.md: マイグレーション内では SET LOCAL search_path TO <schema>, public; を使用する。
SET LOCAL search_path TO board_service, public;

ALTER TABLE outbox_events FORCE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS tenant_isolation ON outbox_events;

CREATE POLICY tenant_isolation ON outbox_events
    AS RESTRICTIVE
    USING (
        current_setting('app.current_tenant_id', true) IS NULL
        OR tenant_id = current_setting('app.current_tenant_id', true)::TEXT
    )
    WITH CHECK (
        tenant_id = current_setting('app.current_tenant_id', true)::TEXT
    );
